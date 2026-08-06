<script lang="ts">
  /**
   * The New-item dialog — every field the prototype draws, per type, and now saving.
   *
   * The form is built from `ITEM_FIELDS` rather than from a block of markup per type. That
   * changed when `add_item` landed: markup describing an item and code building the payload
   * are two statements of the same thing, and the first edit that touches only one of them is
   * a field saved under the wrong label or, worse, a secret saved unmasked.
   *
   * The typed password is scored through `score_password` in the host, the same path onboarding
   * uses: zxcvbn's dictionaries are a few hundred kilobytes and S-01 budgets 800ms for cold
   * start, so they never enter the bundle.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import IconButton from '../components/IconButton.svelte';
  import Segmented from '../components/Segmented.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import Toggle from '../components/Toggle.svelte';
  import { ALL_SETS, generatePassword } from '../passwords';
  import {
    addItem,
    asIpcError,
    scorePassword,
    type ItemKind,
    type NewField,
    type Strength,
  } from '../ipc';
  import { ITEM_FIELDS, TITLE_PLACEHOLDERS, passwordIndex, type FieldTemplate } from './itemFields';
  import { TYPE_GLYPHS, tagColour, typeLabel } from './views';

  interface Props {
    vaultName: string;
    vaultFile: string;
    /** Tags already in the vault, offered as chips. */
    tags: string[];
    onclose: () => void;
    /** The item landed in the vault; the shell reloads the list and selects it. */
    onsaved: (itemId: string) => void;
  }

  const { vaultName, vaultFile, tags, onclose, onsaved }: Props = $props();

  const KINDS: ItemKind[] = ['login', 'api_key', 'card', 'note', 'wifi', 'ssh_key', 'identity'];

  /** A fixed choice starts on its first option; everything else starts empty. */
  const blank = (of: ItemKind) => ITEM_FIELDS[of].map((field) => field.options?.[0] ?? '');

  let kind = $state<ItemKind>('login');
  let title = $state('');
  let values = $state<string[]>(blank('login'));
  let revealed = $state(false);
  let totpOn = $state(false);
  let chosen = $state<string[]>([]);
  let strength = $state<Strength | null>(null);
  let saving = $state(false);
  let error = $state('');

  function pickKind(next: ItemKind) {
    if (next === kind) return;
    kind = next;
    // Cleared rather than carried across. The types share labels ("Password" is on three of
    // them) and a value that survived a type change would be a secret the user believes they
    // discarded, sitting in a field they are no longer looking at.
    values = blank(next);
    revealed = false;
    totpOn = false;
    strength = null;
    error = '';
  }

  const templates = $derived(ITEM_FIELDS[kind]);
  const passwordAt = $derived(passwordIndex(kind));
  const password = $derived(passwordAt >= 0 ? (values[passwordAt] ?? '') : '');

  /**
   * The visible fields, grouped into rows so an `inline` template shares a row with the one
   * before it — the card's Expiry and CVV.
   */
  const rows = $derived.by(() => {
    const out: { template: FieldTemplate; index: number }[][] = [];
    templates.forEach((template, index) => {
      if (template.behindTotpSwitch) return;
      const last = out[out.length - 1];
      if (template.inline && last) last.push({ template, index });
      else out.push([{ template, index }]);
    });
    return out;
  });

  const totpField = $derived.by(() => {
    const index = templates.findIndex((template) => template.behindTotpSwitch);
    return index < 0 ? null : { template: templates[index]!, index };
  });

  let scoreTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const current = password;
    clearTimeout(scoreTimer);
    if (!current) {
      strength = null;
      return;
    }
    scoreTimer = setTimeout(() => {
      void scorePassword(current, [title, vaultName]).then((result) => {
        if (password === current) strength = result;
      });
    }, 180);
  });

  const toggleTag = (tag: string) =>
    (chosen = chosen.includes(tag) ? chosen.filter((one) => one !== tag) : [...chosen, tag]);

  const glyph = (candidate: ItemKind): IconName => TYPE_GLYPHS[candidate];

  const canSave = $derived(title.trim().length > 0 && !saving);

  async function save() {
    if (!canSave) return;
    saving = true;
    error = '';

    /*
     * Empty fields are not sent.
     *
     * The alternative is an item whose detail pane is mostly blank rows, and a blank row is
     * not information — it is a field the user declined to fill in, which the absence says
     * better. Editing the item adds it back the moment there is something to put in it.
     */
    const fields: NewField[] = templates
      .map((template, index) => ({ template, value: (values[index] ?? '').trim() }))
      .filter(({ template, value }) => value !== '' && (!template.behindTotpSwitch || totpOn))
      .map(({ template, value }) => ({
        label: template.label,
        kind: template.kind,
        value,
        secret: template.secret,
        // Everything a template produces is one of the type's own — D-43. `custom` is stored,
        // never inferred, and this is the one place the false is asserted rather than guessed.
        custom: false,
      }));

    try {
      const { itemId } = await addItem(kind, title.trim(), chosen, fields);
      onsaved(itemId);
    } catch (thrown) {
      error = asIpcError(thrown).message;
      saving = false;
    }
  }
</script>

<Dialog title="New item" icon="plus" meta={vaultName} width={560} maxHeight={720} {onclose}>
  <p class="section">Item type</p>
  <div class="chips">
    {#each KINDS as candidate (candidate)}
      <button
        type="button"
        class="chip"
        class:on={kind === candidate}
        aria-pressed={kind === candidate}
        onclick={() => pickKind(candidate)}
      >
        <Icon name={glyph(candidate)} size={14} />
        {typeLabel(candidate)}
      </button>
    {/each}
  </div>

  <div class="form">
    <div class="control">
      <label for="new-title">Title</label>
      <!-- svelte-ignore a11y_autofocus -- the dialog exists to be filled in from the top. -->
      <input id="new-title" bind:value={title} placeholder={TITLE_PLACEHOLDERS[kind]} autofocus />
    </div>

    {#each rows as row, rowIndex (rowIndex)}
      <div class:split={row.length > 1}>
        {#each row as { template, index } (template.label)}
          <div class="control" class:grow={row.length > 1 && index !== row[row.length - 1]?.index}>
            {#if template.options}
              <span class="label-as-text">{template.label}</span>
              <Segmented
                label={template.label}
                options={template.options.map((option) => ({ value: option, label: option }))}
                value={values[index] ?? ''}
                onchange={(next) => (values[index] = next as string)}
              />
            {:else}
              <label for="new-{index}">{template.label}</label>

              {#if template.multiline}
                <textarea
                  id="new-{index}"
                  class:mono={template.mono}
                  rows={template.kind === 'note' ? 5 : 3}
                  spellcheck="false"
                  placeholder={template.placeholder}
                  bind:value={values[index]}></textarea>
              {:else if index === passwordAt}
                <div class="composite">
                  <input
                    id="new-{index}"
                    class="mono"
                    type={revealed ? 'text' : 'password'}
                    bind:value={values[index]}
                  />
                  <IconButton
                    icon={revealed ? 'eye-off' : 'eye'}
                    label={revealed ? 'Hide password' : 'Reveal password'}
                    onclick={() => (revealed = !revealed)}
                  />
                  <Button
                    variant="ghost"
                    icon="refresh"
                    title="Generate a new password on this device"
                    onclick={() => {
                      values[index] = generatePassword(20, ALL_SETS);
                      revealed = true;
                    }}
                  >
                    Generate
                  </Button>
                </div>
                <div class="meter"><StrengthMeter {strength} width="150px" /></div>
              {:else}
                <input
                  id="new-{index}"
                  class:mono={template.mono}
                  type={template.secret ? 'password' : 'text'}
                  placeholder={template.placeholder}
                  spellcheck="false"
                  bind:value={values[index]}
                />
              {/if}
            {/if}
          </div>
        {/each}
      </div>
    {/each}

    {#if kind === 'api_key'}
      <p class="hint">Stored encrypted. Only the last 4 characters show in the list.</p>
    {/if}

    {#if totpField}
      <div class="switch">
        <Toggle label="Save 2FA secret" checked={totpOn} onchange={(next) => (totpOn = next)} />
        <div class="switch-text">
          <p class="switch-label">Save 2FA secret (TOTP)</p>
          <p class="hint">
            When you turn on 2FA, the site shows a QR code and a secret key. Save it here —
            TrustVault generates the codes locally.
          </p>
        </div>
      </div>

      {#if totpOn}
        <div class="nested">
          <div class="control">
            <label for="new-totp">Secret key</label>
            <div class="composite">
              <input
                id="new-totp"
                class="mono"
                bind:value={values[totpField.index]}
                placeholder={totpField.template.placeholder}
                spellcheck="false"
              />
            </div>
          </div>
          <p class="hint">
            Paste the secret first — the live 6-digit preview arrives with TOTP support.
          </p>
        </div>
      {/if}
    {/if}

    {#if tags.length}
      <div class="control">
        <span class="label-as-text">Tags</span>
        <div class="chips">
          {#each tags as tag (tag)}
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
        </div>
      </div>
    {/if}

    {#if error}
      <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
    {/if}
  </div>

  {#snippet footer()}
    <span class="stored">Stored encrypted in {vaultFile}</span>
    <Button onclick={onclose}>Cancel</Button>
    <Button
      variant="primary"
      disabled={!canSave}
      title={title.trim() ? undefined : 'An item needs a title'}
      onclick={() => void save()}
    >
      {saving ? 'Saving…' : 'Save item'}
    </Button>
  {/snippet}
</Dialog>

<style>
  .section {
    margin-bottom: var(--space-3);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
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

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-top: 18px;
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
  input::placeholder,
  textarea::placeholder {
    color: var(--fg-subtle);
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

  .split {
    display: flex;
    gap: var(--space-4);
  }
  .split .control {
    width: 150px;
  }
  .split .control.grow {
    flex: 1;
    width: auto;
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

  .nested {
    padding: var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
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
