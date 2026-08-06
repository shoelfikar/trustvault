<script lang="ts">
  /**
   * The password generator — drawn as the prototype draws it, with two deliberate differences.
   *
   * **The value is minted by the host**, through `generate_password` (D-44), which is the
   * fourth and last sanctioned command. It used to be minted here with
   * `crypto.getRandomValues`, and that was the right call for a dialog with no command behind
   * it; it stopped being right the moment the value could be saved. Randomness protecting a
   * stored credential belongs on the one path R-06 and D-23 constrain.
   *
   * **Copying goes through `copy_generated`**, never `navigator.clipboard`. That is D-37's
   * constraint honoured rather than repealed: what D-37 objected to was a copy nothing would
   * ever clear, and the host schedules the same clear `copy_field` does.
   *
   * The footer says *Copy password*, not the prototype's *Copy & autofill*: autofill is a
   * browser extension, and `trustvault-project.md` puts it out of scope for v1. A button
   * promising it would be the same class of mistake as D-49's Trash copy.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import IconButton from '../components/IconButton.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import {
    ALL_SETS,
    asIpcError,
    copyGenerated,
    generatePassword,
    type CharSets,
    type Strength,
  } from '../ipc';

  interface Props {
    onclose: () => void;
  }

  const { onclose }: Props = $props();

  /**
   * The chips, and their labels are load-bearing.
   *
   * "2–9" rather than "0–9" because the generator excludes the ambiguous glyphs `0 O 1 l I`
   * permanently (`MASTER.md` §3): a generated password is transcribed by hand more often than
   * anything else in the app. There is no toggle for it — the prototype draws four chips and
   * its own digit label already says 2–9, so the exclusion is a property of the sets rather
   * than an option, and the labels stay true.
   */
  const OPTIONS: { key: keyof CharSets; label: string }[] = [
    { key: 'lowercase', label: 'a–z' },
    { key: 'uppercase', label: 'A–Z' },
    { key: 'digits', label: '2–9' },
    { key: 'symbols', label: '!@#$' },
  ];

  let length = $state(20);
  let enabled = $state<CharSets>({ ...ALL_SETS });
  let value = $state('');
  let strength = $state<Strength | null>(null);
  let error = $state('');
  /** Seconds until the clipboard clears, after a copy — the chip §5 asks for. */
  let clipboardLeft = $state(0);
  let clipboardTimer: ReturnType<typeof setInterval> | undefined;

  /** The request currently in flight, so a slow answer cannot overwrite a newer one. */
  let inFlight = 0;

  async function generate() {
    const request = ++inFlight;
    try {
      const generated = await generatePassword(length, { ...enabled });
      if (request !== inFlight) return;
      value = generated.password;
      strength = { score: generated.score, label: generated.label, crackTime: generated.crackTime };
      error = '';
    } catch (thrown) {
      if (request !== inFlight) return;
      error = asIpcError(thrown).message;
    }
  }

  /**
   * Turns a set on or off, refusing to turn the last one off.
   *
   * The host answers a request with no character class at all with `internal`, because such a
   * request is a bug in this file rather than something a user did. Keeping the last chip on
   * is what makes that true.
   */
  function toggle(key: keyof CharSets) {
    const next = { ...enabled, [key]: !enabled[key] };
    if (!next.lowercase && !next.uppercase && !next.digits && !next.symbols) return;
    enabled = next;
  }

  $effect(() => {
    // Re-run whenever the shape of the request changes.
    void length;
    void enabled;
    void generate();
  });

  async function copy() {
    if (!value) return;
    try {
      const { clearsAt } = await copyGenerated(value);
      clearInterval(clipboardTimer);
      const tick = () => {
        clipboardLeft = Math.max(0, Math.ceil((clearsAt - Date.now()) / 1000));
        if (clipboardLeft === 0) clearInterval(clipboardTimer);
      };
      clipboardTimer = setInterval(tick, 250);
      tick();
      error = '';
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  $effect(() => () => clearInterval(clipboardTimer));
</script>

<Dialog title="Password generator" icon="refresh" width={480} {onclose}>
  <div class="preview">
    <p class="value">{value}</p>
    <IconButton
      icon="refresh"
      label="Regenerate"
      title="Regenerate"
      size={16}
      onclick={() => void generate()}
    />
    <IconButton
      icon="copy"
      label="Copy generated password"
      title="Copy to the clipboard; TrustVault clears its own copy"
      size={16}
      onclick={() => void copy()}
    />
  </div>

  <div class="strength"><StrengthMeter {strength} width="150px" /></div>

  {#if clipboardLeft > 0}
    <!-- The wording is D-41's, measured rather than assumed: TrustVault clears its own copy and
         cannot reach a clipboard manager's history. The title carries the rest. -->
    <p
      class="clip"
      title="TrustVault clears its own copy. A clipboard manager (GPaste, Klipper, CopyQ) may keep its own — measured against GPaste 45.3, which does."
    >
      <Icon name="clock" size={13} />TrustVault clears its copy in {clipboardLeft}s
    </p>
  {/if}

  {#if error}
    <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
  {/if}

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
        onclick={() => toggle(option.key)}
      >
        <span class="tick" class:off={!enabled[option.key]}><Icon name="check" size={12} /></span>
        {option.label}
      </button>
    {/each}
  </div>

  <p class="note">
    <Icon name="alert" size={13} />
    <span>
      Generated by TrustVault itself with the operating system's random source, never in this
      window. Ambiguous glyphs (0/O, 1/l/I) are left out so the password survives being written down
      and typed back in.
    </span>
  </p>

  {#snippet footer()}
    <span class="grow"></span>
    <Button onclick={onclose}>Close</Button>
    <Button variant="primary" icon="copy" onclick={() => void copy()}>Copy password</Button>
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

  /* Brass because it reports an interaction the user just performed, not a verdict. §2. */
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
    color: var(--accent);
  }

  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 10px;
    font-size: var(--text-sm);
    color: var(--danger);
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
