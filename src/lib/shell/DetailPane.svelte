<script lang="ts">
  /**
   * The detail pane — title, type icon, status chip, field rows.
   *
   * It fetches through `get_item`, which returns fields with secrets **elided**: masked
   * placeholders and metadata. Nothing on this screen holds a secret until the user reveals
   * one, field by field, through `SecretField`.
   *
   * `history` is not here and cannot be — the contract leaves it out of the shape entirely
   * (§6.1). A user's previous passwords for one site are worse in aggregate than any single
   * one of them, so the reveal for those arrives with its own sanctioned path or not at all.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import SecretField from '../components/SecretField.svelte';
  import StatusChip from '../components/StatusChip.svelte';
  import { asIpcError, getItem, onFieldRemasked, type ItemDetail, type ItemKind } from '../ipc';

  interface Props {
    itemId: string | null;
  }

  const { itemId }: Props = $props();

  let detail = $state<ItemDetail | null>(null);
  let error = $state('');
  /** Bumped when the host remasks a field, which tells every row to drop its copy. */
  let remaskSignal = $state(0);

  const glyphs: Record<ItemKind, IconName> = {
    login: 'key',
    api_key: 'terminal',
    card: 'card',
    note: 'note',
    wifi: 'wifi',
    ssh_key: 'terminal',
    identity: 'user',
  };

  $effect(() => {
    const id = itemId;
    if (!id) {
      detail = null;
      return;
    }
    error = '';
    void getItem(id)
      .then((loaded) => {
        if (itemId === id) detail = loaded;
      })
      .catch((thrown) => {
        if (itemId === id) error = asIpcError(thrown).message;
      });
  });

  $effect(() => {
    // Broadcast rather than targeted: the signal is a "something was remasked" tick and every
    // row drops its copy. Simpler than routing by id, and erring towards masking is the right
    // direction for this particular error to fall.
    const unlisten = onFieldRemasked(() => (remaskSignal += 1));
    return () => void unlisten.then((stop) => stop());
  });

  const updated = $derived(
    detail ? new Date(detail.updatedAt).toLocaleString(undefined, { dateStyle: 'medium' }) : '',
  );
</script>

<section class="detail" aria-label="Item detail">
  {#if error}
    <EmptyState icon="alert" message={error} />
  {:else if !detail}
    <EmptyState icon="vault" message="Select an item to see its details." />
  {:else}
    <header>
      <span class="glyph"><Icon name={glyphs[detail.kind]} size={20} /></span>
      <h1>{detail.title}</h1>
      <StatusChip status={detail.status} />
    </header>

    <div class="fields">
      {#each detail.fields as field (field.id)}
        <SecretField itemId={detail.id} {field} {remaskSignal} />
      {/each}
    </div>

    <footer>
      {#if detail.tags.length}
        <div class="tags">
          {#each detail.tags as tag (tag)}
            <span class="tag"><Icon name="tag" size={11} />{tag}</span>
          {/each}
        </div>
      {/if}
      <p class="updated">Updated {updated}</p>
    </footer>
  {/if}
</section>

<style>
  .detail {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    padding: var(--space-5);
    background: var(--bg-raised);
    overflow-y: auto;
  }

  header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--border);
  }
  .glyph {
    display: flex;
    color: var(--fg-muted);
  }
  h1 {
    flex: 1;
    font-size: var(--text-md);
    line-height: var(--text-md-lh);
    font-weight: var(--weight-semibold);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fields {
    display: flex;
    flex-direction: column;
    padding: var(--space-2) 0;
  }

  footer {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin-top: auto;
    padding-top: var(--space-4);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 1px var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: var(--text-micro);
    color: var(--fg-muted);
  }
  .updated {
    margin-left: auto;
    font-size: var(--text-sm);
    color: var(--fg-subtle);
  }
</style>
