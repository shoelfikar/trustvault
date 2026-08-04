<script lang="ts">
  /**
   * A labelled text or password input.
   *
   * `mono` is not a style toggle. MASTER.md §3 makes it a **correctness requirement**: every
   * rendered secret uses the mono face with tabular numerals and disambiguated `0/O` and
   * `1/l/I`, because these strings get transcribed by hand. A master password box and a
   * recovery-code box are both places where a misread glyph costs a vault.
   */
  import Icon from '../icons/Icon.svelte';

  interface Props {
    label: string;
    value: string;
    type?: 'text' | 'password';
    placeholder?: string;
    /** Renders in `--font-mono`. Required for anything the user may transcribe. */
    mono?: boolean;
    autofocus?: boolean;
    /** Shown beneath in `--danger`, with an icon — never colour alone. */
    error?: string;
    /** Shown beneath in `--fg-subtle` when there is no error. */
    hint?: string;
    oninput?: (value: string) => void;
    onenter?: () => void;
  }

  let {
    label,
    value = $bindable(),
    type = 'text',
    placeholder = '',
    mono = false,
    autofocus = false,
    error = '',
    hint = '',
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

  <div class="box" class:invalid={Boolean(error)}>
    <!-- svelte-ignore a11y_autofocus -- a single-purpose screen whose only job is this input;
         moving focus to it is what a keyboard user expects, not a hijack. -->
    <input
      {id}
      type={effectiveType}
      class:mono
      {placeholder}
      {autofocus}
      aria-invalid={error ? 'true' : undefined}
      aria-describedby={describedBy}
      {value}
      oninput={handle}
      onkeydown={(event) => event.key === 'Enter' && onenter?.()}
    />

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
    gap: var(--space-2);
  }

  label {
    font-size: var(--text-micro);
    line-height: var(--text-micro-lh);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-muted);
  }

  .box {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-2) 0 var(--space-3);
    background: var(--bg-base);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    transition: border-color var(--dur-instant) var(--ease-out);
  }
  .box:focus-within {
    border-color: var(--accent);
  }
  .box.invalid {
    border-color: var(--danger);
  }

  input {
    flex: 1;
    height: 30px;
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

  /* MASTER.md §3 — the transcription rule. `ss01`/`ss02` are Geist Mono's slashed zero and
     disambiguated l/I; without them 0/O and 1/l/I are a coin toss on a printed recovery kit. */
  input.mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    letter-spacing: var(--tracking-lg);
  }

  .peek {
    display: flex;
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    color: var(--fg-subtle);
    transition: color var(--dur-instant) var(--ease-out);
  }
  .peek:hover {
    color: var(--fg);
  }
  .peek:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .note {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
  }
  /* Never colour alone: the icon in the markup is the second channel. */
  .note.error {
    color: var(--danger);
  }
</style>
