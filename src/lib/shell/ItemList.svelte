<script lang="ts">
  /**
   * The item list pane — `MASTER.md` §7, R-11. Header strip, rows, empty state.
   *
   * 34px rows, type icon in `--fg-muted`, title at `--text-base`, subtitle right-aligned at
   * `--text-sm`/`--fg-muted` on the same line, status pip right-aligned. Selected gets
   * `--bg-selected` plus a 2px brass left bar; keyboard focus is a 1px inset ring, which §7
   * requires to be **visually distinct from selection** — a keyboard user moving through the
   * list must be able to see where they are and what is open at the same time.
   *
   * The left bar is an inset `box-shadow` rather than a border, so selecting a row does not
   * shift its text by 2px. A layout that moves on selection reads as a bug.
   *
   * No shadow, no hover lift, no staggered entrance. §5 forbids all three, and §1's ban list
   * names `translateY(-2px)` on rows specifically.
   */
  import Icon from '../icons/Icon.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import IconButton from '../components/IconButton.svelte';
  import StatusChip from '../components/StatusChip.svelte';
  import type { ItemSummary } from '../ipc';
  import { TYPE_GLYPHS, typeLabel, viewTitle, type View } from './views';

  interface Props {
    items: ItemSummary[];
    view: View;
    selectedId: string | null;
    error?: string;
    onselect: (id: string) => void;
    ongenerate: () => void;
    onadd: () => void;
    /** The empty states' way out: every pane that cannot be filled from here offers All Items. */
    onview: (next: View) => void;
  }

  const {
    items,
    view,
    selectedId,
    error = '',
    onselect,
    ongenerate,
    onadd,
    onview,
  }: Props = $props();

  /**
   * The subtitle: the item's tags.
   *
   * Only ever a value the host already sent in the clear. There is no path here that could
   * reach a masked field, because the list shape does not carry field values at all — this
   * reads the item's own metadata.
   */
  const subtitle = (item: ItemSummary) => item.tags.join(' · ');

  /**
   * The empty state, per view — R-19, `MASTER.md` §7.
   *
   * One message for all of them is what this file carried until now, and it stated two things
   * that were not true. Standing in Favorites in a vault holding fifty items, it read *"No items
   * in this vault yet"*; standing in Trash it read *"Deleted items sit here for 30 days"*,
   * against a `delete_item` that removes the item and a Trash that can never hold anything —
   * the same sentence D-49 took out of the delete dialog, still here one screen over.
   *
   * So each branch names **what is actually empty**, and the action is what the person standing
   * in that pane can do about it: add an item where adding fills the pane, and All Items where
   * it does not, because Trash and an unused tag are dead ends you leave rather than fill.
   */
  const empty = $derived.by(() => {
    const toAll = { label: 'Show all items', run: () => onview({ kind: 'all' }) };
    const add = { label: 'Add item', run: onadd };
    switch (view.kind) {
      case 'trash':
        return {
          icon: 'trash' as const,
          text: 'Deleting an item removes it straight away, so nothing collects here.',
          ...toAll,
        };
      case 'favourites':
        return {
          icon: 'star' as const,
          text: 'No favorites yet — star an item to keep it here.',
          ...toAll,
        };
      case 'tag':
        return {
          icon: 'tag' as const,
          text: `Nothing is tagged “${view.tag}” yet.`,
          ...toAll,
        };
      case 'type':
        return {
          icon: TYPE_GLYPHS[view.type],
          text: `No ${typeLabel(view.type)} items in this vault yet.`,
          ...add,
        };
      // `watchtower` and `settings` replace this pane entirely (`isFullWidth`), so they never
      // reach here. They share `all`'s copy rather than a placeholder, because a placeholder is
      // what gets shipped the day one of them stops being full-width.
      default:
        return { icon: 'list' as const, text: 'No items in this vault yet.', ...add };
    }
  });

  /** Roving arrow-key movement, so the list is operable without a pointer (S-08 groundwork). */
  function onkeydown(event: KeyboardEvent, index: number) {
    const next = event.key === 'ArrowDown' ? index + 1 : event.key === 'ArrowUp' ? index - 1 : null;
    if (next === null) return;
    const neighbour = items[next];
    if (!neighbour) return;
    event.preventDefault();
    onselect(neighbour.id);
    const target = event.currentTarget as HTMLElement;
    const sibling = target.parentElement?.parentElement?.children[next]?.querySelector('button');
    (sibling as HTMLElement | null)?.focus();
  }
</script>

<div class="pane">
  <header>
    <span class="title">{viewTitle(view)}</span>
    <span class="count">{items.length}</span>
    <IconButton
      icon="refresh"
      label="Generate password"
      title="Generate password"
      onclick={ongenerate}
    />
    <IconButton icon="plus" label="New item" title="New item" size={16} onclick={onadd} />
  </header>

  {#if error}
    <EmptyState icon="alert" message={error} />
  {:else if items.length === 0}
    <!-- §7: never a blank pane. Every list has an empty state, and every one has an action. -->
    <EmptyState
      icon={empty.icon}
      message={empty.text}
      actionLabel={empty.label}
      onaction={empty.run}
    />
  {:else}
    <ul class="list sb" aria-label="Items">
      {#each items as item, index (item.id)}
        <li>
          <button
            class="row"
            class:selected={item.id === selectedId}
            aria-current={item.id === selectedId ? 'true' : undefined}
            onclick={() => onselect(item.id)}
            onkeydown={(event) => onkeydown(event, index)}
          >
            <span class="glyph"><Icon name={TYPE_GLYPHS[item.kind]} size={16} /></span>
            <span class="row-title">{item.title}</span>
            {#if item.favourite}
              <span class="fav"><Icon name="star" size={12} label="Favourite" /></span>
            {/if}
            <span class="sub">{subtitle(item)}</span>
            <StatusChip status={item.status} pip />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--bg-surface);
    border-right: 1px solid var(--border);
  }

  header {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    height: var(--toolbar-h);
    padding: 0 var(--space-3) 0 var(--space-5);
    border-bottom: 1px solid var(--border);
  }
  .title {
    flex: 1;
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    font-size: var(--text-micro);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg-subtle);
  }

  .list {
    flex: 1;
    min-height: 0;
    padding: var(--space-2) 0;
    overflow-y: auto;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: var(--row-h);
    padding: 0 var(--space-4);
    text-align: left;
    transition: background var(--dur-instant) var(--ease-out);
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row.selected,
  .row.selected:hover {
    background: var(--bg-selected);
    box-shadow: inset 2px 0 0 0 var(--accent);
  }
  /* §7: focus must be distinguishable from selection, so it is a ring and not a fill. */
  .row:focus-visible {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }

  .glyph {
    display: flex;
    color: var(--fg-muted);
  }
  .row.selected .glyph {
    color: var(--accent);
  }

  .row-title {
    flex: 1;
    min-width: 0;
    font-size: var(--text-base);
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fav {
    display: flex;
    flex: none;
    color: var(--fg-subtle);
  }

  .sub {
    max-width: 106px;
    font-size: var(--text-sm);
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
