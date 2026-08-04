<script lang="ts">
  /**
   * The password generator — drawn as the prototype draws it, with one deliberate difference.
   *
   * **The generated value cannot be copied from here, and that is the architecture, not an
   * omission.** `docs/ipc-contract.md` names `generate_password` as a *fourth* sanctioned
   * command that does not exist yet, and says adding one is a decision rather than a patch. The
   * reason it belongs in the host: a password the webview generates lives in a heap that cannot
   * be wiped, and the clipboard clear that makes a copy safe is scheduled by Rust in
   * `copy_field`. Copying from here would produce a secret in the clipboard that nothing ever
   * clears — worse than the app's own promise.
   *
   * So the preview is real (`crypto.getRandomValues`, never `Math.random`) and readable, the
   * controls work, and the two copy paths are disabled with the reason on them.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import IconButton from '../components/IconButton.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import { generatePassword, ALL_SETS, type CharacterSet } from '../passwords';
  import { scorePassword, type Strength } from '../ipc';

  interface Props {
    onclose: () => void;
  }

  const { onclose }: Props = $props();

  const OPTIONS: { key: CharacterSet; label: string }[] = [
    { key: 'lower', label: 'a–z' },
    { key: 'upper', label: 'A–Z' },
    { key: 'digits', label: '2–9' },
    { key: 'symbols', label: '!@#$' },
  ];

  let length = $state(20);
  let enabled = $state<Record<CharacterSet, boolean>>({ ...ALL_SETS });
  let value = $state('');
  let strength = $state<Strength | null>(null);

  const generate = () => (value = generatePassword(length, enabled));

  $effect(() => {
    // Re-run whenever the shape of the request changes.
    void length;
    void enabled.lower;
    void enabled.upper;
    void enabled.digits;
    void enabled.symbols;
    generate();
  });

  $effect(() => {
    const current = value;
    if (!current) return;
    void scorePassword(current).then((result) => {
      if (value === current) strength = result;
    });
  });
</script>

<Dialog title="Password generator" icon="refresh" width={480} {onclose}>
  <div class="preview">
    <p class="value">{value}</p>
    <IconButton icon="refresh" label="Regenerate" title="Regenerate" size={16} onclick={generate} />
    <IconButton
      icon="copy"
      label="Copy generated password"
      title="Copying arrives with the host-side generator — it is what schedules the clipboard clear"
      size={16}
      disabled
    />
  </div>

  <div class="strength"><StrengthMeter {strength} width="150px" /></div>

  <div class="length">
    <span class="label">Length</span>
    <input type="range" min="8" max="64" bind:value={length} aria-label="Password length" />
    <span class="number">{length}</span>
  </div>

  <div class="options">
    {#each OPTIONS as option (option.key)}
      <button
        type="button"
        class="chip"
        class:on={enabled[option.key]}
        aria-pressed={enabled[option.key]}
        onclick={() => (enabled = { ...enabled, [option.key]: !enabled[option.key] })}
      >
        <span class="tick" class:off={!enabled[option.key]}><Icon name="check" size={12} /></span>
        {option.label}
      </button>
    {/each}
  </div>

  <p class="note">
    <Icon name="alert" size={13} />
    <span>
      This preview is generated on this device with the OS random source. It cannot be copied yet:
      the clipboard auto-clear is scheduled by the host, and a copy that skips it would leave the
      password in the clipboard for good.
    </span>
  </p>

  {#snippet footer()}
    <span class="grow"></span>
    <Button onclick={onclose}>Close</Button>
    <Button
      variant="primary"
      disabled
      title="Copying arrives with the host-side generator — it is what schedules the clipboard clear"
    >
      Copy &amp; autofill
    </Button>
  {/snippet}
</Dialog>

<style>
  .preview {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
  }
  /* MASTER.md §3: a generated password is transcribed by hand more often than any other string
     in the app, so mono + tabular + disambiguated glyphs is a correctness requirement here. */
  .value {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: var(--text-md);
    line-height: 22px;
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    word-break: break-all;
    user-select: text;
  }

  .strength {
    margin-top: var(--space-4);
  }

  .length {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin-top: 18px;
  }
  .label {
    width: 96px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }
  .length input {
    flex: 1;
    height: 4px;
    accent-color: var(--accent);
  }
  .number {
    width: 28px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: var(--text-base);
    font-variant-numeric: tabular-nums;
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 14px;
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
    transition:
      background var(--dur-instant) var(--ease-out),
      border-color var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  /* Brass marks which options are engaged — interaction, not a verdict about the character
     set. §2. */
  .chip.on {
    border-color: var(--accent);
    background: var(--accent-wash);
    color: var(--accent);
  }
  .chip:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .tick {
    display: flex;
  }
  .tick.off {
    opacity: 0.18;
  }

  .note {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-5);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
    text-wrap: pretty;
  }
  .note :global(.icon) {
    margin-top: 2px;
  }

  .grow {
    flex: 1;
  }
</style>
