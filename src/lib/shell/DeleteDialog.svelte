<script lang="ts">
  /**
   * The destructive confirm — `MASTER.md` §7 and the DB rule `confirmation-dialogs`.
   *
   * The rule has two tiers and both are here: deleting an item names the item, and deleting a
   * vault makes you **type the vault name**. The typing gate is not theatre — it is the only
   * thing standing between a mis-aimed click and a file whose contents cannot be reconstructed
   * from anywhere.
   *
   * **Both halves fire as of 2026-08-06.** They differ in one way that is not cosmetic: the
   * item's confirmation is checked *here* and nowhere else, and the vault's is checked **again
   * in Rust**. The contract says why — a wrong item delete costs one entry, and a wrong vault
   * delete costs everything with no undo anywhere in the product, so R-18's typed name is not
   * left to the layer the contract does not trust. The gate below is therefore a courtesy that
   * keeps the button quiet until the name matches; the real check is the host's, and its
   * `confirmation_mismatch` is rendered like any other error.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import { asIpcError, deleteItem, deleteVault } from '../ipc';

  interface Props {
    /** `item` names the item; `vault` demands the name typed back. */
    target: 'item' | 'vault';
    name: string;
    /** Which item to delete. Required when `target` is `item`. */
    itemId?: string | null;
    /** Which vault file to delete. Required when `target` is `vault`. */
    vaultPath?: string | null;
    itemCount?: number;
    onclose: () => void;
    ondeleted?: () => void;
  }

  const {
    target,
    name,
    itemId = null,
    vaultPath = null,
    itemCount = 0,
    onclose,
    ondeleted,
  }: Props = $props();

  let typed = $state('');
  let deleting = $state(false);
  let error = $state('');

  const confirmed = $derived(target === 'item' || typed.trim() === name);

  const canDelete = $derived(target === 'item' ? Boolean(itemId) : Boolean(vaultPath));

  /**
   * The copy says what actually happens.
   *
   * It said "It goes to Trash for 30 days first", which was written against a Trash view that
   * holds nothing and a command that does not have one: `delete_item` removes the item and
   * saves, and the removed value is zeroized on drop. A password manager promising a recovery
   * window it does not have is the worst kind of wrong copy — it is the sentence someone reads
   * right before they click, and it is the reason they click.
   */
  const body = $derived(
    target === 'item'
      ? `“${name}” and every field on it are removed from this vault straight away. There is no Trash and no undo.`
      : `${itemCount} items will be gone for good. The file is deleted from this computer and there is no copy anywhere else.`,
  );

  /**
   * Named `remove`, not `confirm`, since 2026-08-06 — D-59.
   *
   * `tauri-plugin-dialog`'s init script replaces `window.confirm` with an **async** function, so
   * the global that this declaration used to shadow no longer means what its name says: a
   * `if (confirm(…))` anywhere tests a promise and is therefore always true. The local shadow
   * was safe, and that is the problem — it is safe until somebody moves the call, and the thing
   * it guards is the irreversible one. `ipc_audit.rs` now forbids the identifier outright.
   */
  async function remove() {
    if (!confirmed || !canDelete || deleting) return;
    deleting = true;
    error = '';
    try {
      // What the user typed, **never the `name` prop**: sending the prop would have the host
      // compare a string against itself, which is exactly the check being in Rust undone.
      // Trimmed, because the gate above trims — the two must agree, or a trailing space
      // enables the button and then the host says the name does not match.
      if (target === 'vault') await deleteVault(vaultPath as string, typed.trim());
      else await deleteItem(itemId as string);
      ondeleted?.();
    } catch (thrown) {
      error = asIpcError(thrown).message;
      deleting = false;
    }
  }
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

    {#if error}
      <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
    {/if}

    <div class="actions">
      <Button onclick={onclose}>Cancel</Button>
      <Button
        variant="primary"
        disabled={!confirmed || !canDelete || deleting}
        onclick={() => void remove()}
      >
        {#if deleting}
          Deleting…
        {:else}
          {target === 'item' ? 'Delete item' : 'Delete vault'}
        {/if}
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

  /* Status is never colour alone — §2. The icon and the sentence carry it. */
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: var(--space-5);
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
