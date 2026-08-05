<script lang="ts">
  /**
   * A labelled text or password input — the prototype's 32px control with an 11px uppercase
   * caption above it.
   *
   * `mono` is not a style toggle. MASTER.md §3 makes it a **correctness requirement**: every
   * rendered secret uses the mono face with tabular numerals and disambiguated `0/O` and
   * `1/l/I`, because these strings get transcribed by hand. A master password box and a
   * recovery-code box are both places where a misread glyph costs a vault.
   *
   * The label is a real `<label>` and never a placeholder — §9's `placeholder-as-label` is on
   * the binding anti-pattern list.
   */
  import Icon from '../icons/Icon.svelte';

  interface Props {
    label: string;
    value: string;
    type?: 'text' | 'password';
    placeholder?: string;
    /** Renders in `--font-mono`. Required for anything the user may transcribe. */
    mono?: boolean;
    /** Wide letter-spacing, for the dot-run of a masked password. */
    spaced?: boolean;
    autofocus?: boolean;
    disabled?: boolean;
    /** Which surface the control sits on; the prototype uses both. */
    surface?: 'surface' | 'raised';
    /** Shown beneath in `--danger`, with an icon — never colour alone. */
    error?: string;
    /** Shown beneath in `--fg-subtle` when there is no error. */
    hint?: string;
    /** Rendered inside the control, right-aligned — the "Change" button on the path field. */
    trailing?: import('svelte').Snippet;
    oninput?: (value: string) => void;
    onenter?: () => void;
  }

  let {
    label,
    value = $bindable(),
    type = 'text',
    placeholder = '',
    mono = false,
    spaced = false,
    autofocus = false,
    disabled = false,
    surface = 'surface',
    error = '',
    hint = '',
    trailing,
    oninput,
    onenter,
  }: Props = $props();

  /** Whether a password field is currently showing its characters. */
  let shown = $state(false);

  const id = `field-${Math.random().toString(36).slice(2, 9)}`;
  const describedBy = $derived(error || hint ? `${id}-note` : undefined);
  const effectiveType = $derived(type === 'password' && !shown ? 'password' : 'text');

  function handle(event: Event) {
    const next = (event.currentTarget as HTMLInputElement).value;
    value = next;
    oninput?.(next);
  }
</script>

<div class="field">
  <label for={id}>{label}</label>

  <div class="box {surface}" class:invalid={Boolean(error)}>
    <!-- svelte-ignore a11y_autofocus -- a single-purpose screen whose only job is this input;
         moving focus to it is what a keyboard user expects, not a hijack. -->
    <input
      {id}
      type={effectiveType}
      class:mono={mono || (type === 'password' && !shown)}
      class:spaced={spaced || (type === 'password' && !shown)}
      {placeholder}
      {autofocus}
      {disabled}
      aria-invalid={error ? 'true' : undefined}
      aria-describedby={describedBy}
      {value}
      oninput={handle}
      onkeydown={(event) => event.key === 'Enter' && onenter?.()}
    />

    {#if trailing}{@render trailing()}{/if}

    {#if type === 'password'}
      <button
        type="button"
        class="peek"
        onclick={() => (shown = !shown)}
        aria-pressed={shown}
        aria-label={shown ? 'Hide password' : 'Show password'}
      >
        <Icon name={shown ? 'eye-off' : 'eye'} size={15} />
      </button>
    {/if}
  </div>

  {#if error}
    <p class="note error" id="{id}-note">
      <Icon name="alert" size={13} />
      {error}
    </p>
  {:else if hint}
    <p class="note" id="{id}-note">{hint}</p>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
  }

  label {
    margin-bottom: 6px;
    font-size: var(--text-micro);
    line-height: var(--text-micro-lh);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .box {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: 32px;
    padding: 0 var(--space-2) 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    transition:
      border-color var(--dur-instant) var(--ease-out),
      box-shadow var(--dur-instant) var(--ease-out);
  }
  .box.surface {
    background: var(--bg-surface);
  }
  .box.raised {
    background: var(--bg-raised);
  }
  /* The prototype's focus ring: a 2px accent wash outside a brass hairline. It is a
     `box-shadow` rather than an `outline` so it hugs the 4px radius. */
  .box:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }
  .box.invalid {
    border-color: var(--danger);
  }

  input {
    flex: 1;
    min-width: 0;
    height: 100%;
    background: transparent;
    border: none;
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--text-base);
  }
  input:focus {
    /* The ring lives on `.box` so it wraps the reveal button too; removing it here is safe
       only because of that, which is why the two rules sit next to each other. */
    outline: none;
  }
  input::placeholder {
    color: var(--fg-subtle);
  }
  input:disabled {
    color: var(--fg-muted);
  }

  /* MASTER.md §3 — the transcription rule. `ss01`/`ss02` are Geist Mono's slashed zero and
     disambiguated l/I; without them 0/O and 1/l/I are a coin toss on a printed recovery kit. */
  input.mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
  }
  input.spaced {
    letter-spacing: 0.14em;
  }

  .peek {
    display: grid;
    place-items: center;
    flex: none;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    color: var(--fg-muted);
    transition: color var(--dur-instant) var(--ease-out);
  }
  .peek:hover {
    color: var(--fg);
  }
  .peek:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .note {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: 6px;
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
  }
  /* Never colour alone: the icon in the markup is the second channel. */
  .note.error {
    color: var(--danger);
  }
</style>
