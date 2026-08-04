<script lang="ts">
  /**
   * One field row: label, value, reveal, copy — R-12, R-14.
   *
   * This is the component the entire IPC contract was shaped around, so the rules it obeys are
   * worth stating where someone editing it will read them:
   *
   * * **Reveal is never a permanent toggle.** MASTER.md §7 forbids one that survives
   *   navigation, and R-12 caps it at 10 s. The countdown here is *cosmetic* — the host owns
   *   the real timer and pushes `field-remasked`, so a frozen tab or a paused JS timer cannot
   *   extend a reveal.
   * * **Copy never receives the value.** `copy_field` returns only when the clipboard will be
   *   cleared. There is no code path in this file that could put a copied secret into a
   *   variable, because none is offered one.
   * * **The mask is not the secret's length** (D-32). Nothing here derives anything from it.
   */
  import Icon from '../icons/Icon.svelte';
  import { asIpcError, copyField, revealField, type FieldSummary } from '../ipc';

  interface Props {
    itemId: string;
    field: FieldSummary;
    /** Set by the parent when the host says this field was remasked. */
    remaskSignal: number;
  }

  const { itemId, field, remaskSignal }: Props = $props();

  /** The revealed plaintext, held only while it is on screen. */
  let revealed = $state<string | null>(null);
  /** Seconds left on the cosmetic countdown. */
  let secondsLeft = $state(0);
  /** Seconds until the clipboard clears, after a copy. */
  let clipboardLeft = $state(0);
  let error = $state('');

  let countdown: ReturnType<typeof setInterval> | undefined;
  let clipboardTimer: ReturnType<typeof setInterval> | undefined;

  /**
   * The host said this field is remasked. Drop the copy immediately.
   *
   * The frontend is trusted to do this and it cannot be verified — the contract says so
   * plainly. It is the one place the security depends on the untrusted side behaving, which is
   * why the window is 10 seconds rather than 60.
   */
  $effect(() => {
    void remaskSignal;
    revealed = null;
    secondsLeft = 0;
    clearInterval(countdown);
  });

  $effect(() => () => {
    clearInterval(countdown);
    clearInterval(clipboardTimer);
  });

  async function reveal() {
    if (revealed !== null) {
      // Hiding early is allowed and is purely local: the host's timer keeps running and its
      // event will arrive regardless, which is correct — this only shortens the exposure.
      revealed = null;
      secondsLeft = 0;
      clearInterval(countdown);
      return;
    }
    error = '';
    try {
      const result = await revealField(itemId, field.id);
      revealed = result.value;
      clearInterval(countdown);
      countdown = setInterval(() => {
        secondsLeft = Math.max(0, Math.ceil((result.remaskAt - Date.now()) / 1000));
        if (secondsLeft === 0) clearInterval(countdown);
      }, 250);
      secondsLeft = Math.max(0, Math.ceil((result.remaskAt - Date.now()) / 1000));
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  async function copy() {
    error = '';
    try {
      // Note what is *not* assigned here. The response has no value in it to assign.
      const { clearsAt } = await copyField(itemId, field.id);
      clearInterval(clipboardTimer);
      clipboardTimer = setInterval(() => {
        clipboardLeft = Math.max(0, Math.ceil((clearsAt - Date.now()) / 1000));
        if (clipboardLeft === 0) clearInterval(clipboardTimer);
      }, 250);
      clipboardLeft = Math.max(0, Math.ceil((clearsAt - Date.now()) / 1000));
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  const shown = $derived(field.secret ? (revealed ?? field.mask ?? '') : (field.value ?? ''));
</script>

<div class="row">
  <span class="label">{field.label}</span>

  <span class="value" class:mono={field.secret || field.kind === 'otp'}>{shown}</span>

  <div class="actions">
    {#if secondsLeft > 0}
      <!-- Cosmetic. The host owns the real timer; this only says what it is doing. -->
      <span class="chip" aria-live="off">{secondsLeft}s</span>
    {/if}
    {#if clipboardLeft > 0}
      <!--
        Whether this may promise a clear at all is the open question against the Phase 2 gate:
        clipboard managers keep their own copy and the platform hints are advisory. The wording
        is "clears in", not "cleared" -- but it still claims more than the platform guarantees,
        and must be revisited once GPaste and Klipper have actually been tested.
      -->
      <span class="chip">Clears in {clipboardLeft}s</span>
    {/if}

    {#if field.secret}
      <button
        type="button"
        onclick={reveal}
        aria-pressed={revealed !== null}
        aria-label={revealed !== null ? `Hide ${field.label}` : `Reveal ${field.label}`}
      >
        <Icon name={revealed !== null ? 'eye-off' : 'eye'} size={15} />
      </button>
    {/if}

    <button type="button" onclick={copy} aria-label="Copy {field.label}">
      <Icon name="copy" size={15} />
    </button>
  </div>
</div>

{#if error}
  <p class="error"><Icon name="alert" size={12} />{error}</p>
{/if}

<style>
  .row {
    display: grid;
    grid-template-columns: 96px 1fr auto;
    align-items: center;
    gap: var(--space-4);
    min-height: var(--field-row-h);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border);
  }

  .label {
    font-size: var(--text-micro);
    line-height: var(--text-micro-lh);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-muted);
  }

  .value {
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg);
    overflow-wrap: anywhere;
  }

  /*
   * MASTER.md §3, the transcription rule. Every rendered secret uses the mono face with
   * tabular numerals and disambiguated 0/O and 1/l/I. This is a correctness requirement for
   * hand-transcription, not a style preference — a masked field is unmasked to be read aloud
   * or typed somewhere else, and that is when a misread glyph costs something.
   */
  .value.mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    letter-spacing: var(--tracking-lg);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .actions button {
    display: flex;
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    color: var(--fg-subtle);
    transition: color var(--dur-instant) var(--ease-out);
  }
  .actions button:hover {
    color: var(--fg);
  }
  .actions button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .chip {
    padding: 1px var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
    font-size: var(--text-micro);
    font-variant-numeric: tabular-nums;
    color: var(--fg-muted);
    white-space: nowrap;
  }

  .error {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
