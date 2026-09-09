<script lang="ts">
  /**
   * The Edit-item dialog — R-17, `docs/ipc-contract.md` §6.4.
   *
   * **The prototype does not draw this surface.** Its detail-pane Edit button has no handler at
   * all, so there are no pixel values to follow and `MASTER.md` is the only authority here; the
   * dialog is built out of the New-item dialog's own vocabulary so the two do not become two
   * different opinions about what an item looks like.
   *
   * It is **field-driven, not type-driven**, and that is the load-bearing difference from the
   * New-item dialog. An item in the vault is a list of fields, not an instance of a template:
   * an imported one carries custom fields with arbitrary labels (D-43), and the contract's
   * third rule is that **omission deletes**. A form built from `ITEM_FIELDS[kind]` would
   * therefore submit a list missing every field the template does not know about, and delete
   * them — in silence, on a rename. So the form is built from what `get_item` actually returned.
   *
   * The second rule is the one this surface exists to express: **`value: null` means
   * unchanged.** The form never received the secret values — §6.1 elides them — so a secret row
   * shows its mask and sends `null` until the user explicitly replaces it. The mask is a fixed
   * twelve characters and is not the secret's length (D-32); nothing here may treat it as one.
   */
  import { tick } from 'svelte';
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import IconButton from '../components/IconButton.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import Toggle from '../components/Toggle.svelte';
  import TotpPreview from '../components/TotpPreview.svelte';
  import {
    ALL_SETS,
    asIpcError,
    generatePassword,
    getItem,
    scorePassword,
    updateItem,
    type EditField,
    type FieldKind,
    type ItemDetail,
    type Strength,
  } from '../ipc';
  import { tagColour, typeLabel } from './views';

  interface Props {
    itemId: string;
    vaultName: string;
    vaultFile: string;
    /** Tags already in the vault, offered as chips. */
    tags: string[];
    onclose: () => void;
    onsaved: () => void;
  }

  const { itemId, vaultName, vaultFile, tags, onclose, onsaved }: Props = $props();

  /**
   * One row of the form.
   *
   * `value: null` is the wire meaning — *keep what is stored* — carried through the UI state
   * unchanged rather than translated at the last moment, so there is one representation of the
   * rule instead of two.
   */
  interface Row {
    /** `null` for a field being added here. */
    id: string | null;
    label: string;
    kind: FieldKind;
    secret: boolean;
    custom: boolean;
    value: string | null;
    mask: string;
    revealed: boolean;
  }

  let detail = $state<ItemDetail | null>(null);
  let title = $state('');
  let favourite = $state(false);
  let chosen = $state<string[]>([]);
  let rows = $state<Row[]>([]);
  let strength = $state<Strength | null>(null);
  let saving = $state(false);
  let error = $state('');
  let loadError = $state('');

  $effect(() => {
    const id = itemId;
    void getItem(id)
      .then((loaded) => {
        if (itemId !== id) return;
        detail = loaded;
        title = loaded.title;
        favourite = loaded.favourite;
        chosen = [...loaded.tags];
        rows = loaded.fields.map((field) => ({
          id: field.id,
          label: field.label,
          kind: field.kind,
          secret: field.secret,
          custom: field.custom,
          // The elided secret starts as `null` — unchanged. The public value starts as itself,
          // which is safe because §6.1 already sent it: it is not a secret by the user's own
          // declaration, and it is what the detail pane is drawing right now.
          value: field.secret ? null : (field.value ?? ''),
          mask: field.mask ?? '',
          revealed: false,
        }));
      })
      .catch((thrown) => {
        if (itemId === id) loadError = asIpcError(thrown).message;
      });
  });

  /** The password being replaced, if one is — the only value here worth scoring. */
  const typedPassword = $derived(
    rows.find((row) => row.kind === 'password' && row.value !== null)?.value ?? '',
  );

  let scoreTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const current = typedPassword;
    clearTimeout(scoreTimer);
    if (!current) {
      strength = null;
      return;
    }
    scoreTimer = setTimeout(() => {
      void scorePassword(current, [title, vaultName]).then((result) => {
        if (typedPassword === current) strength = result;
      });
    }, 180);
  });

  const toggleTag = (tag: string) =>
    (chosen = chosen.includes(tag) ? chosen.filter((one) => one !== tag) : [...chosen, tag]);

  /** Tags invented here, offered beside the vault's own — see `NewItemDialog`. */
  let created = $state<string[]>([]);
  let fresh = $state('');

  const offered = $derived([...tags, ...created]);

  /** Adds the typed tag and selects it, folding a case-only difference onto the existing one. */
  function addTag() {
    const tag = fresh.trim();
    if (!tag) return;
    const existing = offered.find((one) => one.toLowerCase() === tag.toLowerCase());
    if (existing) {
      if (!chosen.includes(existing)) chosen = [...chosen, existing];
    } else {
      created = [...created, tag];
      chosen = [...chosen, tag];
    }
    fresh = '';
  }

  /**
   * Opens a secret row for replacement. Until this is pressed the row sends `null`.
   *
   * Focus is moved into the input this creates, and that is a keyboard-audit row (18) rather
   * than a nicety: Replace swaps a disabled input for an editable one in the same position, and
   * nothing about that moves focus on its own — a keyboard user would press Replace and then
   * have to Tab backwards to reach what they just asked for.
   */
  async function replace(index: number) {
    const row = rows[index];
    if (!row) return;
    row.value = '';
    row.revealed = true;
    await tick();
    document.getElementById(`edit-secret-${index}`)?.focus();
  }

  /**
   * Replaces a secret row's value with one minted by the host — D-44.
   *
   * Writes into `row.value`, which means the row stops being *unchanged*: `null` is the only
   * value that keeps the stored secret, so generating here is a replacement the user has to
   * mean. `Keep` puts it back.
   */
  async function fillGenerated(index: number) {
    const row = rows[index];
    if (!row) return;
    try {
      const generated = await generatePassword(20, ALL_SETS);
      row.value = generated.password;
      row.revealed = true;
      error = '';
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  /** Puts a secret row back to unchanged, discarding whatever was typed into it. */
  function keep(index: number) {
    const row = rows[index];
    if (!row) return;
    row.value = null;
    row.revealed = false;
  }

  /** Removes the row. Omission is what deletes the field — there is no `remove_field`. */
  const drop = (index: number) => (rows = rows.filter((_, at) => at !== index));

  /**
   * Appends a field the user is adding by hand.
   *
   * `custom: true` and `id: null`, both stored rather than inferred — D-43. A field added here
   * is not one of the type's own, and `set_field` on the host deliberately cannot see it, so a
   * custom "Username" can never overwrite the login's real one.
   */
  const addField = () =>
    (rows = [
      ...rows,
      {
        id: null,
        label: '',
        kind: 'text',
        secret: false,
        custom: true,
        // Not `null`: `id: null` with `value: null` reads as *create a field whose value is
        // unchanged*, which the contract refuses outright rather than inventing a meaning for.
        value: '',
        mask: '',
        revealed: false,
      },
    ]);

  /** A field being created needs a label; nothing else on this form is required. */
  const unlabelled = $derived(rows.some((row) => row.id === null && row.label.trim() === ''));
  const canSave = $derived(Boolean(detail) && title.trim().length > 0 && !unlabelled && !saving);

  async function save() {
    if (!canSave) return;
    saving = true;
    error = '';

    const fields: EditField[] = rows.map((row) => ({
      id: row.id,
      label: row.label.trim(),
      kind: row.kind,
      value: row.value,
      secret: row.secret,
      custom: row.custom,
    }));

    try {
      await updateItem(itemId, title.trim(), chosen, favourite, fields);
      onsaved();
    } catch (thrown) {
      error = asIpcError(thrown).message;
      saving = false;
    }
  }
</script>

<Dialog
  title="Edit item"
  icon="key"
  meta={detail ? typeLabel(detail.kind) : vaultName}
  width={560}
  maxHeight={720}
  {onclose}
>
  {#if loadError}
    <p class="error" role="alert"><Icon name="alert" size={13} />{loadError}</p>
  {:else if !detail}
    <p class="hint">Loading…</p>
  {:else}
    <div class="form">
      <div class="control">
        <label for="edit-title">Title</label>
        <!-- svelte-ignore a11y_autofocus -- the dialog opens on the field most edits change. -->
        <input id="edit-title" bind:value={title} autofocus />
      </div>

      <div class="switch">
        <Toggle label="Favourite" checked={favourite} onchange={(next) => (favourite = next)} />
        <div class="switch-text">
          <p class="switch-label">Favourite</p>
          <p class="hint">Pinned to the top of the sidebar's Favorites view.</p>
        </div>
      </div>

      <p class="section">Fields</p>

      {#each rows as row, index (row.id ?? `new-${index}`)}
        <div class="control">
          <div class="row-head">
            {#if row.custom}
              <input
                class="label-input"
                bind:value={row.label}
                placeholder="Field name"
                aria-label="Field name"
              />
            {:else}
              <span class="label-as-text">{row.label}</span>
            {/if}
            <span class="grow"></span>
            {#if row.custom}<span class="badge">Custom</span>{/if}
            <IconButton
              icon="trash"
              label="Remove {row.label || 'field'}"
              title="Remove this field from the item"
              tone="danger"
              onclick={() => drop(index)}
            />
          </div>

          {#if row.secret && row.value === null}
            <!-- The mask, not the value, and it is a fixed twelve characters — D-32. It is
                 shown in a disabled input so the row reads as "there is something here" while
                 being visibly not editable, which is what `value: null` means on the wire. -->
            <div class="composite">
              <input class="mono" value={row.mask} disabled aria-label="{row.label}, unchanged" />
              <Button variant="ghost" onclick={() => void replace(index)}>Replace</Button>
            </div>
            <p class="hint">Unchanged. The stored value never leaves the vault to be edited.</p>
          {:else if row.secret}
            <div class="composite">
              <input
                id="edit-secret-{index}"
                class="mono"
                type={row.revealed ? 'text' : 'password'}
                bind:value={rows[index]!.value}
                aria-label="New value for {row.label}"
              />
              <IconButton
                icon={row.revealed ? 'eye-off' : 'eye'}
                label={row.revealed ? 'Hide value' : 'Reveal value'}
                onclick={() => (row.revealed = !row.revealed)}
              />
              {#if row.kind === 'password'}
                <Button
                  variant="ghost"
                  icon="refresh"
                  title="Generate a new password with the operating system's random source"
                  onclick={() => void fillGenerated(index)}
                >
                  Generate
                </Button>
              {/if}
              {#if row.id}
                <Button variant="ghost" onclick={() => keep(index)}>Keep</Button>
              {/if}
            </div>
            {#if row.kind === 'password'}
              <div class="meter"><StrengthMeter {strength} width="150px" /></div>
            {:else if row.kind === 'otp'}
              <!--
                The same validator the Add dialog has, and it is here because this is the other
                way a seed gets into the vault. A preview on only one of the two paths would be
                reported as "sometimes it checks" — and the seed replaced here is the more
                dangerous one, because the item already worked before the edit.
              -->
              <TotpPreview seed={row.value ?? ''} />
            {/if}
          {:else if row.kind === 'note'}
            <textarea rows="4" bind:value={rows[index]!.value} aria-label={row.label}></textarea>
          {:else}
            <input
              class:mono={row.kind !== 'text' && row.kind !== 'username'}
              bind:value={rows[index]!.value}
              spellcheck="false"
              aria-label={row.label}
            />
          {/if}
        </div>
      {/each}

      <div>
        <Button icon="plus" onclick={addField}>Add field</Button>
      </div>

      <div class="control">
        <label class="label-as-text" for="edit-item-tag">Tags</label>
        <div class="chips">
          {#each offered as tag (tag)}
            <button
              type="button"
              class="chip tag"
              class:on={chosen.includes(tag)}
              aria-pressed={chosen.includes(tag)}
              onclick={() => toggleTag(tag)}
            >
              <span class="dot" style="background:{tagColour(tag)}"></span>{tag}
            </button>
          {/each}
          <input
            id="edit-item-tag"
            class="new-tag"
            bind:value={fresh}
            placeholder="New tag…"
            spellcheck="false"
            onkeydown={(event) => {
              if (event.key !== 'Enter') return;
              event.preventDefault();
              event.stopPropagation();
              addTag();
            }}
            onblur={addTag}
          />
        </div>
      </div>

      {#if error}
        <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
      {/if}
    </div>
  {/if}

  {#snippet footer()}
    <span class="stored">Stored encrypted in {vaultFile}</span>
    <Button onclick={onclose}>Cancel</Button>
    <Button
      variant="primary"
      disabled={!canSave}
      title={unlabelled ? 'A new field needs a name' : undefined}
      onclick={() => void save()}
    >
      {saving ? 'Saving…' : 'Save changes'}
    </Button>
  {/snippet}
</Dialog>

<style>
  .section {
    margin: var(--space-2) 0 0;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .control {
    display: flex;
    flex-direction: column;
  }
  .control label,
  .label-as-text {
    margin-bottom: 6px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .row-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: 6px;
  }
  .row-head .label-as-text {
    margin-bottom: 0;
  }
  .grow {
    flex: 1;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    height: 18px;
    padding: 0 6px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    font-size: var(--text-micro);
    color: var(--fg-subtle);
  }

  input,
  textarea {
    width: 100%;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    outline: none;
    transition:
      border-color var(--dur-instant) var(--ease-out),
      box-shadow var(--dur-instant) var(--ease-out);
  }
  input {
    height: 32px;
  }
  textarea {
    padding: var(--space-3) 10px;
    line-height: 18px;
    resize: none;
  }
  input:focus,
  textarea:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }
  input:disabled {
    color: var(--fg-subtle);
  }
  input::placeholder {
    color: var(--fg-subtle);
  }

  .label-input {
    height: 24px;
    width: 200px;
    padding: 0 var(--space-2);
    font-size: var(--text-sm);
  }

  /* MASTER.md §3 — anything the user may transcribe renders in the mono face. */
  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
  }

  .composite {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 32px;
    padding: 0 var(--space-2) 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    transition:
      border-color var(--dur-instant) var(--ease-out),
      box-shadow var(--dur-instant) var(--ease-out);
  }
  .composite:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }
  .composite input {
    height: 100%;
    padding: 0;
    border: none;
    background: none;
  }
  .composite input:focus {
    box-shadow: none;
  }

  .meter {
    margin-top: var(--space-3);
  }

  .switch {
    display: flex;
    align-items: flex-start;
    gap: 9px;
  }
  .switch-text {
    flex: 1;
    min-width: 0;
  }
  .switch-label {
    font-size: var(--text-base);
  }

  .hint {
    margin-top: 6px;
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
    text-wrap: pretty;
  }
  .switch-text .hint {
    margin-top: 0;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: flex;
    align-items: center;
    gap: 7px;
    height: var(--control-h);
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    color: var(--fg-muted);
    white-space: nowrap;
    transition:
      background var(--dur-instant) var(--ease-out),
      border-color var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  .chip:hover:not(.on) {
    color: var(--fg);
  }
  /* Brass marks the engaged chip — interaction, not a verdict. §2. */
  .chip.on {
    border-color: var(--accent);
    background: var(--accent-wash);
    color: var(--accent);
  }
  .chip:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .chip.tag {
    height: 24px;
    padding: 0 9px;
    gap: 6px;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 2px;
  }
  /* Chip-shaped, for the reason `NewItemDialog` gives: it belongs to the row it adds to. */
  .new-tag {
    height: 24px;
    min-width: 92px;
    max-width: 140px;
    padding: 0 9px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: none;
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .new-tag::placeholder {
    color: var(--fg-subtle);
  }
  .new-tag:focus-visible {
    outline: 1px solid var(--accent);
    outline-offset: 1px;
  }

  /* Status is never colour alone — §2. The icon and the sentence carry it. */
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-sm);
    color: var(--danger);
  }

  .stored {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--fg-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
