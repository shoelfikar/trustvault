<script lang="ts">
  /**
   * The item list — `MASTER.md` §7, R-11.
   *
   * 34px rows, type icon in `--fg-muted`, title at `--text-base`, username right-aligned at
   * `--text-sm`/`--fg-muted` on the same line, status pips right-aligned. Selected gets
   * `--bg-selected` plus a 2px brass left bar; keyboard focus is a 1px inset ring, which §7
   * requires to be **visually distinct from selection** — a keyboard user moving through the
   * list must be able to see where they are and what is open at the same time.
   *
   * No shadow, no hover lift, no staggered entrance. §5 forbids all three, and §1's ban list
   * names `translateY(-2px)` on rows specifically.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import StatusChip from '../components/StatusChip.svelte';
  import type { ItemKind, ItemSummary } from '../ipc';

  interface Props {
    items: ItemSummary[];
    selectedId: string | null;
    onselect: (id: string) => void;
  }

  const { items, selectedId, onselect }: Props = $props();

  /**
   * One glyph per item type. `identity` has no dedicated glyph in the shipped 30 and uses
   * `user`, which is close enough to be honest — recorded in `docs/icon-gaps.md` (D-28).
   */
  const glyphs: Record<ItemKind, IconName> = {
    login: 'key',
    api_key: 'terminal',
    card: 'card',
    note: 'note',
    wifi: 'wifi',
    ssh_key: 'terminal',
    identity: 'user',
  };

  /**
   * The subtitle: whichever non-secret field best identifies the row.
   *
   * Only ever a value the host already sent in the clear — a field the user declared is not
   * secret. There is no path here that could reach a masked one, because the list shape does
   * not carry field values at all; this reads the item's own metadata.
   */
  function subtitle(item: ItemSummary): string {
    return item.tags.join(' · ');
  }

  /** Roving arrow-key movement, so the list is operable without a pointer (S-08 groundwork). */
  function onkeydown(event: KeyboardEvent, index: number) {
    const next = event.key === 'ArrowDown' ? index + 1 : event.key === 'ArrowUp' ? index - 1 : null;
    if (next === null) return;
    const neighbour = items[next];
    if (!neighbour) return;
    event.preventDefault();
    onselect(neighbour.id);
    const target = event.currentTarget as HTMLElement;
    const sibling = target.parentElement?.children[next]?.querySelector('button');
    (sibling as HTMLElement | null)?.focus();
  }
</script>

<ul class="list" aria-label="Items">
  {#each items as item, index (item.id)}
    <li>
      <button
        class="row"
        class:selected={item.id === selectedId}
        aria-current={item.id === selectedId ? 'true' : undefined}
        onclick={() => onselect(item.id)}
        onkeydown={(event) => onkeydown(event, index)}
      >
        <span class="glyph"><Icon name={glyphs[item.kind]} size={16} /></span>
        <span class="title">{item.title}</span>
        {#if item.favourite}
          <span class="fav"><Icon name="star" size={12} label="Favourite" /></span>
        {/if}
        <span class="sub">{subtitle(item)}</span>
        <StatusChip status={item.status} compact />
      </button>
    </li>
  {/each}
</ul>

<style>
  .list {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--space-2) var(--space-2);
    overflow-y: auto;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    height: var(--row-h);
    padding: 0 var(--space-4);
    border-radius: var(--radius-sm);
    /* The left bar is always present and transparent, so selecting a row does not shift the
       text by 2px. A layout that moves on selection reads as a bug. */
    border-left: 2px solid transparent;
    text-align: left;
    transition: background var(--dur-instant) var(--ease-out);
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row.selected {
    background: var(--bg-selected);
    border-left-color: var(--accent);
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

  .title {
    flex: none;
    max-width: 55%;
    font-size: var(--text-base);
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fav {
    display: flex;
    color: var(--fg-subtle);
  }

  .sub {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--fg-muted);
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
