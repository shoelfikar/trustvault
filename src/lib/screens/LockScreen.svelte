<script lang="ts">
  /**
   * The lock screen — R-07, R-09.
   *
   * Two things here are not obvious and both come from the contract:
   *
   * 1. **The name shown is the file stem, not the vault's name.** The real name lives inside
   *    the sealed body, so until the vault opens there is nothing else to show.
   * 2. **A wrong password and a damaged file give the same message** — R-03. The core makes
   *    them indistinguishable in type and timing; repeating the message verbatim is what keeps
   *    that true at the last place it could leak.
   *
   * The blurred shell behind the card is the prototype's, and it is decoration with a job: it
   * says *this window already has a vault in it* rather than leaving a locked app looking like
   * an empty one. It is drawn from real geometry — 232px sidebar, 300px list — so it lines up
   * with what appears when the card goes away.
   *
   * The prototype also offers "or use Touch ID". There is no biometric path in this build and
   * drawing one would be a promise the app cannot keep, so it is absent rather than disabled.
   */
  import Button from '../components/Button.svelte';
  import TextField from '../components/TextField.svelte';
  import Icon from '../icons/Icon.svelte';
  import Mark from '../icons/Mark.svelte';
  import RecoveryDialog from './RecoveryDialog.svelte';
  import { asIpcError, unlock, type LockReason } from '../ipc';

  interface Props {
    /** Absolute path of the vault to open. */
    path: string;
    /** The file stem — see the note above. */
    displayName: string;
    /** Why the app locked, if it locked rather than started. */
    reason?: LockReason | null;
    /** Drives the "auto-locks after…" line, so it never contradicts the setting. */
    autoLockSeconds?: number;
    onunlocked: () => void;
  }

  const { path, displayName, reason = null, autoLockSeconds = 300, onunlocked }: Props = $props();

  let password = $state('');
  let error = $state('');
  let busy = $state(false);
  let recovering = $state(false);

  /** The prototype's blurred list, nine bars at fixed widths. */
  const ghost = ['70%', '54%', '82%', '48%', '66%', '76%', '58%', '88%', '62%'];

  const idleLine = $derived.by(() => {
    const minutes = Math.round(autoLockSeconds / 60);
    if (autoLockSeconds < 60) return `Auto-locks after ${autoLockSeconds} seconds idle`;
    return `Auto-locks after ${minutes} minute${minutes === 1 ? '' : 's'} idle`;
  });

  const explanation = $derived(
    reason === 'timeout'
      ? 'Locked after a period of inactivity.'
      : reason === 'os_sleep'
        ? 'Locked because this machine went to sleep.'
        : reason === 'manual'
          ? 'Locked.'
          : '',
  );

  async function submit() {
    if (busy || !password) return;
    busy = true;
    error = '';
    try {
      await unlock(path, password);
      password = '';
      onunlocked();
    } catch (thrown) {
      error = asIpcError(thrown).message;
    } finally {
      busy = false;
    }
  }
</script>

<div class="lock">
  <div class="ghost" aria-hidden="true">
    <div class="ghost-sidebar"></div>
    <div class="ghost-list">
      {#each ghost as width, index (index)}
        <span class="bar" style="width:{width}"></span>
      {/each}
    </div>
    <div class="ghost-detail"></div>
  </div>

  {#if !recovering}
    <div class="card">
      <span class="brand"><Mark size={46} /></span>
      <h1>{displayName}</h1>
      <p class="idle">{explanation || idleLine}</p>

      <div class="form">
        <TextField
          label="Master password"
          type="password"
          bind:value={password}
          autofocus
          onenter={submit}
        />

        {#if error}
          <!-- Never colour alone; the glyph is the second channel. -->
          <p class="error"><Icon name="alert" size={13} />{error}</p>
        {/if}
      </div>

      <div class="submit">
        <Button
          variant="primary"
          wide
          tall
          icon="unlock"
          iconSize={15}
          disabled={busy}
          onclick={submit}
        >
          {busy ? 'Unlocking…' : 'Unlock vault'}
        </Button>
      </div>

      <p class="forgot">
        Forgot your master password?
        <button type="button" onclick={() => (recovering = true)}>Use your recovery kit</button>
      </p>
    </div>
  {:else}
    <RecoveryDialog {path} {displayName} onclose={() => (recovering = false)} {onunlocked} />
  {/if}
</div>

<style>
  .lock {
    position: relative;
    display: grid;
    place-items: center;
    height: 100vh;
    background: var(--bg-base);
    overflow: hidden;
  }

  /*
   * MASTER.md §1's ban list forbids glassmorphism *except* on the lock screen, and §5 names the
   * unlock transition as the only expressive moment in the app. This is that one place, and the
   * blur is on a decorative layer rather than on live content — nothing readable is behind it.
   */
  .ghost {
    position: absolute;
    inset: 0;
    display: flex;
    opacity: 0.35;
    filter: blur(9px);
  }
  .ghost-sidebar {
    width: var(--sidebar-w);
    flex: none;
    background: var(--bg-base);
    border-right: 1px solid var(--border);
  }
  .ghost-list {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: var(--list-w);
    flex: none;
    padding: var(--space-4);
    background: var(--bg-surface);
    border-right: 1px solid var(--border);
  }
  .bar {
    height: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
  }
  .ghost-detail {
    flex: 1;
    background: var(--bg-raised);
  }

  .card {
    position: relative;
    width: 352px;
    max-width: calc(100vw - var(--space-7));
    padding: var(--space-7);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    /* The one surface entitled to a shadow: it is a true overlay. §4. */
    box-shadow: var(--shadow-dialog);
    text-align: center;
    animation: rise var(--dur-scrim) var(--ease-out);
  }

  @keyframes rise {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .card {
      animation-duration: var(--dur-instant);
    }
  }

  .brand {
    display: flex;
    justify-content: center;
    color: var(--accent);
  }

  h1 {
    margin-top: var(--space-4);
    font-size: var(--text-xl);
    line-height: var(--text-xl-lh);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-xl);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .idle {
    margin-top: var(--space-1);
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }

  .form {
    margin-top: var(--space-6);
    text-align: left;
  }

  .error {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-3);
    font-size: var(--text-sm);
    color: var(--danger);
  }

  .submit {
    margin-top: var(--space-4);
  }

  .forgot {
    margin-top: 18px;
    padding-top: 14px;
    border-top: 1px solid var(--border);
    font-size: var(--text-sm);
    color: var(--fg-subtle);
  }
  .forgot button {
    color: var(--accent);
    font-size: var(--text-sm);
    transition: color var(--dur-instant) var(--ease-out);
  }
  .forgot button:hover {
    color: var(--accent-hover);
    text-decoration: underline;
  }
</style>
