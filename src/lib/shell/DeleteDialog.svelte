<script lang="ts">
  /**
   * The destructive confirm — `MASTER.md` §7 and the DB rule `confirmation-dialogs`.
   *
   * The rule has two tiers and both are here: deleting an item names the item, and deleting a
   * vault makes you **type the vault name**. The typing gate is not theatre — it is the only
   * thing standing between a mis-aimed click and a file whose contents cannot be reconstructed
   * from anywhere.
   *
   * Neither action fires yet. `delete_item` and vault deletion are Phase 3 commands, so the
   * confirm button carries the reason rather than a handler.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';

  interface Props {
    /** `item` names the item; `vault` demands the name typed back. */
    target: 'item' | 'vault';
    name: string;
    itemCount?: number;
    onclose: () => void;
  }

  const { target, name, itemCount = 0, onclose }: Props = $props();

  let typed = $state('');

  const confirmed = $derived(target === 'item' || typed.trim() === name);

  /**
   * No command exists behind either action yet, so the button never enables.
   *
   * Written as a named constant rather than a hard-coded `disabled` so that landing
   * `delete_item` is a one-line change here and the naming gate above is already wired.
   */
  const CAN_DELETE = false;

  const body = $derived(
    target === 'item'
      ? `“${name}” and every field on it are removed from this vault. It goes to Trash for 30 days first.`
      : `${itemCount} items will be gone for good. The file is deleted from this computer and there is no copy anywhere else.`,
  );
</script>

<Dialog width={420} bare {onclose}>
  <div class="pane">
    <div class="head">
      <span class="glyph"><Icon name="alert" size={20} /></span>
      <div>
        <h2>{target === 'item' ? `Delete “${name}”?` : `Delete ${name}?`}</h2>
        <p class="body">{body}</p>
      </div>
    </div>

    {#if target === 'vault'}
      <div class="confirm">
        <label for="confirm-name">Type “{name}” to confirm</label>
        <input id="confirm-name" class="mono" bind:value={typed} spellcheck="false" />
      </div>
    {/if}

    <div class="actions">
      <Button onclick={onclose}>Cancel</Button>
      <Button
        variant="primary"
        disabled={!confirmed || !CAN_DELETE}
        title="Deletion arrives with the mutation commands"
      >
        {target === 'item' ? 'Delete item' : 'Delete vault'}
      </Button>
    </div>
  </div>
</Dialog>

<style>
  .pane {
    padding: var(--space-7);
  }

  .head {
    display: flex;
    gap: var(--space-4);
  }
  .glyph {
    display: flex;
    flex: none;
    margin-top: 2px;
    color: var(--danger);
  }

  h2 {
    font-size: var(--text-md);
    line-height: var(--text-md-lh);
    font-weight: var(--weight-medium);
  }
  .body {
    margin-top: 6px;
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .confirm {
    display: flex;
    flex-direction: column;
    margin-top: var(--space-5);
  }
  .confirm label {
    margin-bottom: 6px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }
  .confirm input {
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--text-base);
    outline: none;
  }
  .confirm input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }

  .actions {
    display: flex;
    gap: var(--space-3);
    justify-content: flex-end;
    margin-top: var(--space-6);
  }
</style>
