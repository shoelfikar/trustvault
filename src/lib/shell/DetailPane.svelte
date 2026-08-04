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
  import {
    asIpcError,
    getItem,
    onClipboardCleared,
    onFieldRemasked,
    type ItemDetail,
    type ItemStatus,
    type Strength,
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
    ondelete: () => void;
  }

  const { itemId, listEmpty = false, clipboardUntil = 0, oncopied, ondelete }: Props = $props();

  let detail = $state<ItemDetail | null>(null);
  let error = $state('');
  /** Bumped when the host remasks a field, which tells every row to drop its copy. */
  let remaskSignal = $state(0);
  /** Seconds until the clipboard clears, after a copy — the chip §5 asks for. */
  let clipboardLeft = $state(0);
  let clipboardTimer: ReturnType<typeof setInterval> | undefined;

  $effect(() => {
    const id = itemId;
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
      <!-- `update_item` is a Phase 3 command. The control is drawn because the prototype's
           toolbar is part of the shape, and disabled because nothing behind it exists yet. -->
      <Button disabled title="Editing arrives with the mutation commands">Edit</Button>
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
      </div>

      {#if clipboardLeft > 0 && !cleared}
        <!--
          Whether this may promise a clear at all is the open question against the Phase 2 gate:
          clipboard managers keep their own copy and the platform hints are advisory. The
          wording is "clears in", not "cleared" -- but it still claims more than the platform
          guarantees, and must be revisited once GPaste and Klipper have actually been tested.
        -->
        <p class="clip"><Icon name="clock" size={13} />Clipboard clears in {clipboardLeft}s</p>
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
