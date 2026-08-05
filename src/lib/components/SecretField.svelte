<script lang="ts">
  /**
   * One field row inside the detail pane's bordered box: label, value, reveal, copy — R-12, R-14.
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
   *   variable, because none is offered one. The countdown it hands to `oncopied` is a
   *   timestamp, and the parent draws the chip §5 asks for.
   * * **The mask is not the secret's length** (D-32). Nothing here derives anything from it.
   */
  import Icon from '../icons/Icon.svelte';
  import IconButton from './IconButton.svelte';
  import { asIpcError, copyField, revealField, type FieldSummary } from '../ipc';

  interface Props {
    itemId: string;
    field: FieldSummary;
    /** Draws the 1px separator; the first row in the box has none. */
    first?: boolean;
    /** Set by the parent when the host says this field was remasked. */
    remaskSignal: number;
    /** Fires with the epoch-ms the clipboard clears at, so the parent can draw the chip. */
    oncopied?: (clearsAt: number) => void;
  }

  const { itemId, field, first = false, remaskSignal, oncopied }: Props = $props();

  /** The revealed plaintext, held only while it is on screen. */
  let revealed = $state<string | null>(null);
  /** Seconds left on the cosmetic countdown. */
  let secondsLeft = $state(0);
  /** Drives the check-mark swap §5 asks for, 1.2s. */
  let justCopied = $state(false);
  let error = $state('');

  let countdown: ReturnType<typeof setInterval> | undefined;
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

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
    clearTimeout(copiedTimer);
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
      const tick = () => {
        secondsLeft = Math.max(0, Math.ceil((result.remaskAt - Date.now()) / 1000));
        if (secondsLeft === 0) clearInterval(countdown);
      };
      clearInterval(countdown);
      countdown = setInterval(tick, 250);
      tick();
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  async function copy() {
    error = '';
    try {
      // Note what is *not* assigned here. The response has no value in it to assign.
      const { clearsAt } = await copyField(itemId, field.id);
      oncopied?.(clearsAt);
      justCopied = true;
      clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => (justCopied = false), 1200);
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  const shown = $derived(field.secret ? (revealed ?? field.mask ?? '') : (field.value ?? ''));
  const isLink = $derived(field.kind === 'url');
  const isMono = $derived(field.secret || field.kind === 'otp' || isLink);
</script>

<div class="row" class:first>
  <span class="label">{field.label}</span>

  <span class="value" class:mono={isMono} class:link={isLink}>{shown}</span>

  {#if secondsLeft > 0}
    <!-- Cosmetic. The host owns the real timer; this only says what it is doing. -->
    <span class="countdown" aria-live="off">{secondsLeft}s</span>
  {/if}

  {#if field.secret}
    <IconButton
      icon={revealed !== null ? 'eye-off' : 'eye'}
      label={revealed !== null ? `Hide ${field.label}` : `Reveal ${field.label}`}
      title="Reveal · hides again automatically"
      active={revealed !== null}
      onclick={reveal}
    />
  {/if}

  <IconButton
    icon={justCopied ? 'check' : 'copy'}
    tone={justCopied ? 'ok' : 'default'}
    label="Copy {field.label}"
    title="Copy"
    onclick={copy}
  />
</div>

{#if error}
  <p class="error"><Icon name="alert" size={12} />{error}</p>
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    min-height: 34px;
    padding: 6px var(--space-3) 6px 14px;
    border-top: 1px solid var(--border);
  }
  .row.first {
    border-top: none;
  }

  .label {
    width: 124px;
    flex: none;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .value {
    flex: 1;
    min-width: 0;
    font-size: var(--text-base);
    font-variant-numeric: tabular-nums;
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .value.link {
    color: var(--accent);
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
    user-select: text;
  }

  .countdown {
    flex: none;
    font-size: var(--text-micro);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg-subtle);
  }

  .error {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 14px;
    border-top: 1px solid var(--border);
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
