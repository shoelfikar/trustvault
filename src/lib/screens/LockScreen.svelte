<script lang="ts">
  /**
   * The lock screen and the recovery flow — R-07, R-09.
   *
   * Two things here are not obvious and both come from the contract:
   *
   * 1. **The name shown is the file stem, not the vault's name.** The real name lives inside
   *    the sealed body, so until the vault opens there is nothing else to show. The copy says
   *    "file" rather than "vault" so it does not claim more than it knows.
   * 2. **A wrong password and a damaged file give the same message** — R-03. The core makes
   *    them indistinguishable in type and timing; repeating the message verbatim is what keeps
   *    that true at the last place it could leak.
   */
  import Button from '../components/Button.svelte';
  import TextField from '../components/TextField.svelte';
  import Icon from '../icons/Icon.svelte';
  import { asIpcError, unlock, unlockRecoveryKit, type LockReason } from '../ipc';

  interface Props {
    /** Absolute path of the vault to open. */
    path: string;
    /** The file stem — see the note above. */
    displayName: string;
    /** Why the app locked, if it locked rather than started. */
    reason?: LockReason | null;
    onunlocked: () => void;
  }

  const { path, displayName, reason = null, onunlocked }: Props = $props();

  let mode = $state<'password' | 'recovery'>('password');
  let password = $state('');
  let code = $state('');
  let error = $state('');
  let busy = $state(false);
  /** A freshly issued kit, when unlocking spent the old one. */
  let reissued = $state('');

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
    if (busy) return;
    busy = true;
    error = '';
    try {
      if (mode === 'password') {
        await unlock(path, password);
        password = '';
        onunlocked();
      } else {
        const kit = await unlockRecoveryKit(path, code);
        code = '';
        // The old kit is spent, so a new one was issued. It must be shown before moving on,
        // or the user leaves recovery with no way back in next time.
        reissued = kit.recoveryCode;
      }
    } catch (thrown) {
      error = asIpcError(thrown).message;
    } finally {
      busy = false;
    }
  }

  function acknowledgeNewKit() {
    reissued = '';
    onunlocked();
  }
</script>

<div class="scrim">
  <div class="panel">
    {#if reissued}
      <header>
        <span class="mark"><Icon name="key" size={20} /></span>
        <h1>A new recovery kit</h1>
      </header>
      <p class="lede">
        The kit you just used is spent. This one replaces it — shown once, and not stored anywhere.
      </p>
      <div class="kit">
        {#each reissued.split('-') as group, index (index)}
          <span class="group">{group}</span>
        {/each}
      </div>
      <Button variant="primary" wide onclick={acknowledgeNewKit}>I have written it down</Button>
    {:else}
      <header>
        <span class="mark"><Icon name="lock" size={20} /></span>
        <div class="titles">
          <h1>{displayName}</h1>
          <!-- "file", not "vault": the vault's own name is sealed until this screen succeeds. -->
          <p class="path">{path}</p>
        </div>
      </header>

      {#if explanation}
        <p class="reason"><Icon name="clock" size={13} />{explanation}</p>
      {/if}

      {#if mode === 'password'}
        <TextField
          label="Master password"
          type="password"
          bind:value={password}
          autofocus
          onenter={submit}
        />
      {:else}
        <TextField
          label="Recovery code"
          bind:value={code}
          mono
          placeholder="XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"
          hint="Six groups of four, from the kit you printed."
          onenter={submit}
        />
      {/if}

      {#if error}
        <p class="reason danger"><Icon name="alert" size={13} />{error}</p>
      {/if}

      <Button variant="primary" wide disabled={busy} onclick={submit}>
        {busy ? 'Unlocking…' : 'Unlock'}
      </Button>

      <button
        type="button"
        class="switch"
        onclick={() => {
          mode = mode === 'password' ? 'recovery' : 'password';
          error = '';
        }}
      >
        {mode === 'password' ? 'Use a recovery code instead' : 'Use the master password instead'}
      </button>
    {/if}
  </div>
</div>

<style>
  /*
   * MASTER.md §1's ban list forbids glassmorphism *except* on the lock-screen scrim, and §5
   * names the unlock transition as the only expressive moment in the app. This is that one
   * place, and it is the only backdrop-filter in the codebase.
   */
  .scrim {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: color-mix(in srgb, var(--bg-base) 88%, transparent);
    backdrop-filter: blur(8px);
    animation: clear var(--dur-scrim) var(--ease-out);
  }

  @keyframes clear {
    from {
      opacity: 0;
      backdrop-filter: blur(0);
    }
    to {
      opacity: 1;
    }
  }

  /* N-08: reduced motion drops to opacity-only. The blur is the motion here, so it goes. */
  @media (prefers-reduced-motion: reduce) {
    .scrim {
      animation: none;
    }
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    width: 100%;
    max-width: 340px;
    padding: var(--space-6);
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    /* The one surface entitled to a shadow: it is a true overlay. */
    box-shadow: var(--shadow-dialog);
  }

  header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .mark {
    display: flex;
    color: var(--accent);
  }
  .titles {
    min-width: 0;
  }
  h1 {
    font-size: var(--text-md);
    line-height: var(--text-md-lh);
    font-weight: var(--weight-semibold);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .path {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }

  .lede {
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
  }

  .reason {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
  }
  .reason.danger {
    color: var(--danger);
  }

  .switch {
    align-self: center;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    color: var(--fg-muted);
    transition: color var(--dur-instant) var(--ease-out);
  }
  .switch:hover {
    color: var(--fg);
  }
  .switch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .kit {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .group {
    font-family: var(--font-mono);
    font-size: var(--text-base);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    letter-spacing: var(--tracking-lg);
    text-align: center;
  }
</style>
