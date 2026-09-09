<script lang="ts">
  /**
   * The detail pane — toolbar, title block, field box, strength, footer facts.
   *
   * It fetches through `get_item`, which returns fields with secrets **elided**: masked
   * placeholders and metadata. Nothing on this screen holds a secret until the user reveals
   * one, field by field, through `SecretField`.
   *
   * `history` is not here and cannot be — the contract leaves it out of the shape entirely
   * (§6.1). A user's previous passwords for one site are worse in aggregate than any single one
   * of them, so the footer says the values stay sealed rather than counting versions the way
   * the prototype does. That is the honest version of the same slot.
   *
   * The strength readout is derived from the item's Watchtower `status`, which D-26 stores as a
   * *cache of the last scan*. It is not a live score and cannot be: scoring needs the plaintext,
   * and the plaintext is exactly what does not cross this boundary to draw a bar. Where the
   * status is `unknown` the meter says so instead of guessing.
   */
  import Icon from '../icons/Icon.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import Button from '../components/Button.svelte';
  import IconButton from '../components/IconButton.svelte';
  import SecretField from '../components/SecretField.svelte';
  import StatusChip from '../components/StatusChip.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import TotpRing from '../components/TotpRing.svelte';
  import {
    asIpcError,
    copyGenerated,
    getItem,
    onClipboardCleared,
    onFieldRemasked,
    totpCode,
    type ItemDetail,
    type ItemStatus,
    type Strength,
    type TotpCode,
  } from '../ipc';
  import { TYPE_GLYPHS, typeLabel } from './views';

  interface Props {
    itemId: string | null;
    /** Whether the list has anything in it, so the empty copy can say the right thing. */
    listEmpty?: boolean;
    /**
     * Epoch-ms the clipboard is scheduled to clear at, owned by the shell.
     *
     * It lives up there rather than here because the command palette copies too, and two
     * countdowns for one clipboard would eventually disagree about the one thing they describe.
     */
    clipboardUntil?: number;
    oncopied: (clearsAt: number) => void;
    onedit: () => void;
    ondelete: () => void;
    /**
     * Bumped by the shell after a mutation lands, so the pane re-reads.
     *
     * The pane cannot notice on its own: `update_item` returns nothing, deliberately, so that
     * `get_item` stays the single elision path. Re-reading is therefore the only way this
     * screen learns what it now holds.
     */
    reloadSignal?: number;
  }

  const {
    itemId,
    listEmpty = false,
    clipboardUntil = 0,
    oncopied,
    onedit,
    ondelete,
    reloadSignal = 0,
  }: Props = $props();

  let detail = $state<ItemDetail | null>(null);
  let error = $state('');
  /** Bumped when the host remasks a field, which tells every row to drop its copy. */
  let remaskSignal = $state(0);
  /** Seconds until the clipboard clears, after a copy — the chip §5 asks for. */
  let clipboardLeft = $state(0);
  let clipboardTimer: ReturnType<typeof setInterval> | undefined;

  $effect(() => {
    const id = itemId;
    // Read so the effect re-runs when the shell says the item changed underneath.
    void reloadSignal;
    if (!id) {
      detail = null;
      return;
    }
    error = '';
    void getItem(id)
      .then((loaded) => {
        if (itemId === id) detail = loaded;
      })
      .catch((thrown) => {
        if (itemId === id) error = asIpcError(thrown).message;
      });
  });

  $effect(() => {
    // Broadcast rather than targeted: the signal is a "something was remasked" tick and every
    // row drops its copy. Simpler than routing by id, and erring towards masking is the right
    // direction for this particular error to fall.
    const unlisten = onFieldRemasked(() => (remaskSignal += 1));
    return () => void unlisten.then((stop) => stop());
  });

  $effect(() => {
    // The host is the authority on when the clipboard actually cleared; the countdown below is
    // only a prediction of it.
    const unlisten = onClipboardCleared(() => {
      cleared = true;
      clipboardLeft = 0;
      clearInterval(clipboardTimer);
    });
    return () => void unlisten.then((stop) => stop());
  });

  /** Set by the host's own event, which outranks the local prediction. */
  let cleared = $state(false);

  $effect(() => {
    const until = clipboardUntil;
    clearInterval(clipboardTimer);
    if (!until) {
      clipboardLeft = 0;
      return;
    }
    cleared = false;
    const tick = () => {
      clipboardLeft = Math.max(0, Math.ceil((until - Date.now()) / 1000));
      if (clipboardLeft === 0) clearInterval(clipboardTimer);
    };
    clipboardTimer = setInterval(tick, 250);
    tick();
    return () => clearInterval(clipboardTimer);
  });

  /**
   * The Watchtower status, expressed as the meter's four segments.
   *
   * Deliberately not a `Strength` from `score_password`: that needs the password, and the
   * password does not cross the boundary to draw a bar.
   */
  const STRENGTH: Record<ItemStatus, Strength | null> = {
    unknown: null,
    strong: { score: 4, label: 'Very strong', crackTime: '' },
    weak: { score: 1, label: 'Weak', crackTime: '' },
    reused: { score: 2, label: 'Fair', crackTime: '' },
    breached: { score: 1, label: 'Weak', crackTime: '' },
    expired: { score: 2, label: 'Fair', crackTime: '' },
  };

  const hasPassword = $derived(detail?.fields.some((field) => field.kind === 'password') ?? false);
  const strength = $derived(detail ? STRENGTH[detail.status] : null);
  const strengthNote = $derived(
    !hasPassword
      ? 'this kind is not scored'
      : detail?.status === 'unknown'
        ? 'not scanned yet — run Watchtower'
        : 'from the last Watchtower scan',
  );

  /* ---- One-time code — R-20, §6.6, D-45 ---------------------------------- */

  /**
   * The code for the selected item, refreshed a step at a time.
   *
   * Held here rather than fetched per render, and dropped the moment the selection changes:
   * D-45 exempts a code from `Secret`, and the exemption is narrow enough to be worth honouring
   * literally — one item, the selected one, for as long as it is on screen.
   */
  let totp = $state<TotpCode | null>(null);
  /**
   * Why there is no code, when there should be one.
   *
   * Separate from the pane's `error`, which replaces the whole screen: a seed that will not
   * parse is a broken row, not a broken item, and blanking the item's own fields to say so
   * would hide the very thing the user needs in order to fix it.
   */
  let totpError = $state('');
  let totpTimer: ReturnType<typeof setTimeout> | undefined;
  /** Swaps the copy icon to a check for a moment, the way every other copy in the app does. */
  let codeCopied = $state(false);

  /**
   * Whether this item has a seed at all, read off the elided detail.
   *
   * `kind`, not the label: an imported item's seed field can be called anything, and it is the
   * kind that both write paths set explicitly. The value itself is elided to a mask here, which
   * is exactly right — this pane never sees the seed, only whether there is one.
   */
  const hasTotp = $derived(detail?.fields.some((field) => field.kind === 'otp') ?? false);

  $effect(() => {
    const id = itemId;
    void reloadSignal;
    clearTimeout(totpTimer);
    totp = null;
    codeCopied = false;
    if (!id || !hasTotp) return;

    let live = true;
    const refresh = async () => {
      try {
        const next = await totpCode(id);
        if (!live || itemId !== id) return;
        totp = next;
        // Re-fetched at the step boundary rather than every second, and from the host each
        // time rather than recomputed here — the seed is what generates a code, and the seed
        // does not cross. The extra 250 ms keeps a fast clock from asking for the step it is
        // still in and getting the same code back with a ring already at zero.
        totpTimer = setTimeout(
          () => void refresh(),
          Math.max(250, next.expiresAt - Date.now() + 250),
        );
      } catch (thrown) {
        // `no_such_field` is the ordinary answer for an item whose seed field is empty, so it
        // hides the row rather than raising anything. A malformed seed is different: it is a
        // stored value that will never generate a code, and saying so here is the only place
        // the user finds out.
        if (!live || itemId !== id) return;
        const failure = asIpcError(thrown);
        totp = null;
        totpError = failure.kind === 'no_such_field' ? '' : failure.message;
      }
    };
    void refresh();

    return () => {
      live = false;
      clearTimeout(totpTimer);
    };
  });

  /**
   * Copies the code and schedules the clear in Rust — D-37, §5.
   *
   * Through `copy_generated` rather than `navigator.clipboard`, for the same reason the
   * generator goes that way: what D-37 objected to was a copy nothing would ever clear. The
   * command's own argument covers this caller unchanged — it takes a value this side already
   * holds, reads nothing, and returns only the instant of the clear.
   */
  async function copyCode() {
    if (!totp) return;
    try {
      const { clearsAt } = await copyGenerated(totp.code);
      codeCopied = true;
      setTimeout(() => (codeCopied = false), 1200);
      oncopied(clearsAt);
    } catch (thrown) {
      totpError = asIpcError(thrown).message;
    }
  }

  /**
   * `418209` → `418 209`, and an eight-digit code into two fours.
   *
   * Grouped for transcription, which is the same reason §3 puts every secret in the mono face:
   * the user is reading this off the screen and typing it somewhere else, against a clock.
   */
  const grouped = $derived.by(() => {
    const code = totp?.code ?? '';
    const half = Math.ceil(code.length / 2);
    return `${code.slice(0, half)} ${code.slice(half)}`.trim();
  });

  const subtitle = $derived(detail ? detail.tags.join(' · ') : '');
  const updated = $derived(
    detail ? new Date(detail.updatedAt).toLocaleDateString(undefined, { dateStyle: 'medium' }) : '',
  );
</script>

<section class="detail" aria-label="Item detail">
  {#if error}
    <EmptyState icon="alert" message={error} quiet />
  {:else if !detail}
    <EmptyState
      icon="list"
      message={listEmpty ? 'Nothing to show yet.' : 'Select an item to see its details'}
      quiet
    />
  {:else}
    <header class="toolbar">
      <span class="grow"></span>
      <Button title="Edit this item" onclick={onedit}>Edit</Button>
      <IconButton
        icon="trash"
        label="Delete item"
        title="Delete item"
        tone="danger"
        onclick={ondelete}
      />
    </header>

    <div class="body sb">
      <div class="head">
        <span class="glyph"><Icon name={TYPE_GLYPHS[detail.kind]} size={19} /></span>
        <div class="titles">
          <h1>{detail.title}</h1>
          <p class="kind">
            {typeLabel(detail.kind)}{subtitle ? ` · ${subtitle}` : ''}
          </p>
        </div>
        <StatusChip status={detail.status} />
      </div>

      <div class="fields">
        {#each detail.fields as field, index (field.id)}
          <SecretField itemId={detail.id} {field} first={index === 0} {remaskSignal} {oncopied} />
        {/each}

        <!--
          The one-time code sits inside the field box, below the fields, as its own row — the
          prototype's placement, and it is the right one: the code is not a stored field and
          must not look like one, but it belongs to this item and not to the pane.
        -->
        {#if hasTotp}
          <div class="totp">
            <span class="totp-label">
              <span class="totp-name">One-time code</span>
              <span class="totp-note">computed locally from your 2FA secret</span>
            </span>

            {#if totp}
              <span class="code">{grouped}</span>
              <TotpRing expiresAt={totp.expiresAt} period={totp.period} />
              <IconButton
                icon={codeCopied ? 'check' : 'copy'}
                label="Copy one-time code"
                title="Copy code"
                onclick={() => void copyCode()}
              />
            {:else if totpError}
              <!-- Icon and words, never colour alone — §2. -->
              <span class="totp-broken"><Icon name="alert" size={13} />{totpError}</span>
            {:else}
              <span class="totp-note">Generating…</span>
            {/if}
          </div>
        {/if}
      </div>

      {#if clipboardLeft > 0 && !cleared}
        <!--
          Measured 2026-08-05 against GPaste 45.3 on GNOME/Wayland with track-changes on
          (`src-tauri/tests/clipboard_manager.rs`): GPaste recorded the copied value **despite**
          the `x-kde-passwordManagerHint` the copy carries, and still held it after TrustVault
          cleared the clipboard. So the wording says what is true -- TrustVault clears its own
          copy, and cannot reach a manager's history. The title carries the rest, because the
          chip has room for one line and the honest sentence is two.
        -->
        <p
          class="clip"
          title="TrustVault clears its own copy. A clipboard manager (GPaste, Klipper, CopyQ) may keep its own — measured against GPaste 45.3, which does."
        >
          <Icon name="clock" size={13} />TrustVault clears its copy in {clipboardLeft}s
        </p>
      {/if}

      {#if hasPassword || detail.status !== 'unknown'}
        <p class="section">Password strength</p>
        <StrengthMeter {strength} width="180px" note={strengthNote} />
      {/if}

      <footer>
        <div class="fact">
          <p class="section">Tags</p>
          {#if detail.tags.length}
            <div class="tags">
              {#each detail.tags as tag (tag)}
                <span class="tag">{tag}</span>
              {/each}
            </div>
          {:else}
            <p class="value">None</p>
          {/if}
        </div>
        <div class="fact">
          <p class="section">History</p>
          <p class="value">
            <Icon name="history" size={13} />Previous values stay sealed
          </p>
        </div>
        <div class="fact">
          <p class="section">Updated</p>
          <p class="value nums">{updated}</p>
        </div>
      </footer>
    </div>
  {/if}
</section>

<style>
  .detail {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    background: var(--bg-raised);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: none;
    height: var(--toolbar-h);
    padding: 0 var(--space-4) 0 var(--space-5);
    border-bottom: 1px solid var(--border);
  }
  .grow {
    flex: 1;
  }

  .body {
    flex: 1;
    min-height: 0;
    padding: var(--space-6) var(--space-7) var(--space-7);
    overflow-y: auto;
  }

  .head {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    margin-bottom: 20px;
  }
  .glyph {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    flex: none;
    border-radius: var(--radius-md);
    background: var(--accent-wash);
    color: var(--accent);
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  h1 {
    font-size: var(--text-lg);
    line-height: var(--text-lg-lh);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-lg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .kind {
    margin-top: 1px;
    font-size: var(--text-sm);
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* §4: hairline borders do the work. One box, rows separated by a 1px rule. */
  .fields {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  /* The prototype's geometry for this row, kept: 34px tall, 12px gaps, the label column at
     124px so the code starts on the same x as every field value above it. */
  .totp {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 34px;
    padding: var(--space-2) var(--space-2) var(--space-2) 14px;
    border-top: 1px solid var(--border);
  }
  .totp-label {
    width: 124px;
    flex: none;
  }
  .totp-name {
    display: block;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }
  .totp-note {
    display: block;
    font-size: var(--text-micro);
    color: var(--fg-subtle);
  }
  /* §3: a rendered secret — and a code being transcribed under a clock is exactly that — is
     mono, tabular, with 0/O and 1/l/I disambiguated. The tracking is the prototype's. */
  .code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--text-md);
    font-weight: var(--weight-medium);
    letter-spacing: 0.14em;
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
  }
  .totp-broken {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    font-size: var(--text-sm);
    color: var(--danger);
  }

  .clip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 24px;
    margin-top: 10px;
    padding: 0 9px;
    border-radius: var(--radius-sm);
    background: var(--accent-wash);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    font-variant-numeric: tabular-nums;
    /* Brass because it reports an interaction the user just performed, not a verdict. §2. */
    color: var(--accent);
  }

  .section {
    margin: var(--space-6) 0 var(--space-3);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  footer {
    display: flex;
    gap: 36px;
    margin-top: var(--space-6);
    padding-top: var(--space-5);
    border-top: 1px solid var(--border);
  }
  .fact .section {
    margin: 0 0 6px;
  }
  .value {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }
  .value.nums {
    font-variant-numeric: tabular-nums;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 7px;
    border-radius: var(--radius-sm);
    background: var(--accent-wash);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    color: var(--accent);
  }
</style>
