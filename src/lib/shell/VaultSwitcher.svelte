<script lang="ts">
  /**
   * The vault switcher — R-22.
   *
   * Wired 2026-08-06. It listed exactly one row until then, which was the honest state of a
   * feature with no commands under it; it now reads `list_vaults` and every row switches.
   *
   * **Every row but the open one shows a file stem, and the meta line says so.** A vault's real
   * name is inside its sealed body, so with no key there is nothing to read it with — §6.7 asks
   * this surface not to imply otherwise, and the way it does not is by showing the file name
   * itself rather than a name-shaped string the app guessed.
   *
   * Two actions and two different verbs, deliberately far apart. **Leave** removes the row and
   * leaves the file where it is; deleting a vault is not here at all — it is in Settings, behind
   * a dialog that makes you type the name. Putting an irreversible action in the same row as a
   * reversible one is how the wrong one gets clicked.
   *
   * **Open vault file…** is still disabled, and for the reason it always was: a picker means
   * `tauri-plugin-dialog`, and the manifest's standing rule is that a plugin arrives when a
   * requirement needs one and not before, because a plugin is widened attack surface in a
   * process holding decrypted secrets. **New vault** is disabled with a Phase 3 reason of its
   * own — onboarding is the only surface that creates a vault, and reaching it from here means
   * closing the one that is open.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import { asIpcError, forgetVault, listVaults, switchVault, type VaultRef } from '../ipc';

  interface Props {
    /** The open vault's path, so the row for it can be marked without a second lookup. */
    openPath: string | null;
    itemCount: number;
    onclose: () => void;
    /** A switch landed: the host is now locked and pointing elsewhere. */
    onswitched: () => void;
  }

  const { openPath, itemCount, onclose, onswitched }: Props = $props();

  let vaults = $state<VaultRef[]>([]);
  let error = $state('');
  let busy = $state(false);

  /** Bumped after a forget, which is what makes the list re-read. */
  let reloads = $state(0);

  $effect(() => {
    void reloads;
    void listVaults()
      .then((loaded) => (vaults = loaded))
      .catch((thrown) => (error = asIpcError(thrown).message));
  });

  const fileOf = (path: string) => path.split(/[/\\]/).pop() || path;

  /**
   * "3 days ago" is worse than a date here.
   *
   * This is the line a user reads to work out which of two similarly named vaults is the one
   * they meant, and a relative time answers a question they did not ask while hiding the one
   * they did. Never opened is stated rather than left blank.
   */
  function opened(at: number | null): string {
    if (at === null) return 'never opened here';
    return `opened ${new Date(at).toLocaleDateString()}`;
  }

  async function choose(vault: VaultRef) {
    if (busy || vault.path === openPath) return;
    busy = true;
    error = '';
    try {
      await switchVault(vault.path);
      onswitched();
    } catch (thrown) {
      error = asIpcError(thrown).message;
      busy = false;
    }
  }

  /**
   * Leave: the row goes, the file stays.
   *
   * No confirmation, and that is the point of the split — this is the reversible one. The vault
   * comes back the moment it is opened again, so a dialog here would train the user to click
   * through the one that matters.
   */
  async function leave(vault: VaultRef) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      await forgetVault(vault.path);
      reloads += 1;
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
    busy = false;
  }
</script>

<Dialog title="Switch vault" icon="vault" width={440} bare {onclose}>
  <div class="list">
    {#each vaults as vault (vault.path)}
      {@const isOpen = vault.path === openPath}
      <div class="vault" class:open={isOpen}>
        <button
          class="pick"
          disabled={isOpen || busy}
          title={isOpen ? 'This vault is already open' : `Switch to ${vault.displayName}`}
          onclick={() => void choose(vault)}
        >
          <span class="glyph"><Icon name="vault" size={16} /></span>
          <span class="lines">
            <span class="name">{vault.displayName}</span>
            <!-- The file name, not a second copy of the display name: for every row but the
                 open one they are the same string, and showing it as the file is what tells
                 the user this is a stem rather than the name they typed at onboarding. -->
            <span class="meta">
              {fileOf(vault.path)} · {isOpen ? `${itemCount} items · unlocked` : opened(vault.lastOpenedAt)}
            </span>
          </span>
        </button>
        {#if isOpen}
          <span class="badge">Open</span>
        {:else}
          <button class="leave" disabled={busy} onclick={() => void leave(vault)}>Leave</button>
        {/if}
      </div>
    {/each}

    {#if vaults.length === 0 && !error}
      <!-- Unreachable in practice — any vault the app has opened is listed, and a session that
           reaches this dialog has opened one. Drawn anyway: an empty box with no words in it
           reads as a failed load. -->
      <p class="empty">No vaults yet. The one you create at setup appears here.</p>
    {/if}

    {#if error}
      <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
    {/if}
  </div>

  {#snippet footer()}
    <Button icon="note" disabled title="Opening another vault file needs the native file picker">
      Open vault file…
    </Button>
    <Button
      variant="primary"
      icon="plus"
      disabled
      title="Creating a second vault means closing this one first"
    >
      New vault
    </Button>
  {/snippet}
</Dialog>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
  }

  .vault {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding-right: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  /* Brass outline marks the vault you are in — interaction state, not a verdict. §2. */
  .vault.open {
    border-color: var(--accent);
    background: var(--accent-wash);
  }

  /* The whole row is the switch target: a 40px strip is easier to hit than a word, and there
     is nothing else in the row to click by accident. */
  .pick {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex: 1;
    min-width: 0;
    padding: 9px 0 9px var(--space-4);
    text-align: left;
    border-radius: var(--radius-md) 0 0 var(--radius-md);
  }
  .pick:not(:disabled):hover {
    background: var(--bg-hover);
  }
  .pick:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .glyph {
    display: flex;
    flex: none;
    color: var(--accent);
  }

  .lines {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .name {
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    font-size: var(--text-sm);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 7px;
    border-radius: var(--radius-sm);
    background: var(--accent-wash);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    color: var(--accent);
    white-space: nowrap;
  }

  /* Quiet by default. It is the reversible action and it should not compete with the row. */
  .leave {
    flex: none;
    height: var(--control-h);
    padding: 0 var(--space-3);
    border-radius: var(--radius-sm);
    color: var(--fg-subtle);
    font-size: var(--text-sm);
    white-space: nowrap;
    transition: color var(--dur-instant) var(--ease-out);
  }
  .leave:hover {
    color: var(--fg);
  }
  .leave:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .empty {
    padding: var(--space-4);
    font-size: var(--text-sm);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  /* Status is never colour alone — §2. */
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-sm);
    color: var(--danger);
  }

  :global(.list + footer) {
    justify-content: stretch;
  }
</style>
