<script lang="ts">
  /**
   * The vault switcher.
   *
   * Multi-vault is Phase 3, so the list has exactly one row: the vault that is open. Drawing
   * the surface with one row is the honest state of the feature — an empty or absent switcher
   * would say "TrustVault has one vault", which is a different and wrong promise.
   *
   * Both footer actions are disabled and both for reasons worth keeping. **Open vault file…**
   * needs a native file picker, and a picker means `tauri-plugin-dialog`; the manifest's
   * standing rule is that a plugin arrives when a requirement needs one and not before, because
   * a plugin is widened attack surface in a process holding decrypted secrets. **New vault**
   * needs somewhere to put the vault that is currently open, which is the multi-vault question
   * itself.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';

  interface Props {
    vaultName: string;
    vaultFile: string;
    itemCount: number;
    onclose: () => void;
  }

  const { vaultName, vaultFile, itemCount, onclose }: Props = $props();
</script>

<Dialog title="Switch vault" icon="vault" width={440} bare {onclose}>
  <div class="list">
    <div class="vault open">
      <span class="glyph"><Icon name="vault" size={16} /></span>
      <span class="lines">
        <span class="name">{vaultName}</span>
        <span class="meta">{vaultFile} · {itemCount} items · unlocked</span>
      </span>
      <span class="badge">Open</span>
    </div>
  </div>

  {#snippet footer()}
    <Button icon="note" disabled title="Opening another vault file needs the native file picker">
      Open vault file…
    </Button>
    <Button
      variant="primary"
      icon="plus"
      disabled
      title="A second vault arrives with multi-vault support"
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
    gap: var(--space-4);
    padding: 9px var(--space-3) 9px var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  /* Brass outline marks the vault you are in — interaction state, not a verdict. §2. */
  .vault.open {
    border-color: var(--accent);
    background: var(--accent-wash);
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

  :global(.list + footer) {
    justify-content: stretch;
  }
</style>
