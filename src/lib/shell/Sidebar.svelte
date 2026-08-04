<script lang="ts">
  /**
   * Vault navigation with counts, tags, and the Watchtower entry — `MASTER.md` §6.
   *
   * The counts come from the item list the shell already holds, not from a second command.
   * Asking the host again would be a second source of truth for the same number, and the two
   * would disagree the first time a filter changed.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import type { ItemKind, ItemSummary } from '../ipc';

  export type Filter =
    | { kind: 'all' }
    | { kind: 'favourites' }
    | { kind: 'type'; type: ItemKind }
    | { kind: 'tag'; tag: string }
    | { kind: 'watchtower' };

  interface Props {
    items: ItemSummary[];
    filter: Filter;
    onfilter: (next: Filter) => void;
  }

  const { items, filter, onfilter }: Props = $props();

  /** The four types the design's sidebar names. The other three live under All. */
  const types: { type: ItemKind; label: string; icon: IconName }[] = [
    { type: 'login', label: 'Logins', icon: 'key' },
    { type: 'card', label: 'Cards', icon: 'card' },
    { type: 'note', label: 'Notes', icon: 'note' },
    { type: 'wifi', label: 'Wi-Fi', icon: 'wifi' },
  ];

  const favourites = $derived(items.filter((item) => item.favourite).length);
  const tags = $derived(
    [...new Set(items.flatMap((item) => item.tags))].sort((a, b) => a.localeCompare(b)),
  );

  /**
   * Items Watchtower has something to say about.
   *
   * `unknown` and `strong` are not findings — a badge counting them would show a number on a
   * vault where nothing is wrong, which is how a badge stops meaning anything.
   */
  const flagged = $derived(
    items.filter((item) => ['weak', 'reused', 'breached', 'expired'].includes(item.status)).length,
  );

  const countOf = (type: ItemKind) => items.filter((item) => item.kind === type).length;

  const isActive = (candidate: Filter) =>
    filter.kind === candidate.kind &&
    (candidate.kind !== 'type' || (filter.kind === 'type' && filter.type === candidate.type)) &&
    (candidate.kind !== 'tag' || (filter.kind === 'tag' && filter.tag === candidate.tag));
</script>

<nav class="sidebar" aria-label="Vault">
  <ul>
    <li>
      <button class:on={isActive({ kind: 'all' })} onclick={() => onfilter({ kind: 'all' })}>
        <Icon name="list" size={16} />
        <span class="text">All items</span>
        <span class="count">{items.length}</span>
      </button>
    </li>
    <li>
      <button
        class:on={isActive({ kind: 'favourites' })}
        onclick={() => onfilter({ kind: 'favourites' })}
      >
        <Icon name="star" size={16} />
        <span class="text">Favourites</span>
        <span class="count">{favourites}</span>
      </button>
    </li>
  </ul>

  <hr />

  <ul>
    {#each types as entry (entry.type)}
      <li>
        <button
          class:on={isActive({ kind: 'type', type: entry.type })}
          onclick={() => onfilter({ kind: 'type', type: entry.type })}
        >
          <Icon name={entry.icon} size={16} />
          <span class="text">{entry.label}</span>
          <span class="count">{countOf(entry.type)}</span>
        </button>
      </li>
    {/each}
  </ul>

  {#if tags.length}
    <hr />
    <p class="heading">Tags</p>
    <ul>
      {#each tags as tag (tag)}
        <li>
          <button
            class:on={isActive({ kind: 'tag', tag })}
            onclick={() => onfilter({ kind: 'tag', tag })}
          >
            <Icon name="tag" size={16} />
            <span class="text">{tag}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <hr />

  <ul>
    <li>
      <button
        class:on={isActive({ kind: 'watchtower' })}
        onclick={() => onfilter({ kind: 'watchtower' })}
      >
        <Icon name="shield" size={16} />
        <span class="text">Watchtower</span>
        {#if flagged > 0}
          <span class="badge">{flagged}</span>
        {/if}
      </button>
    </li>
  </ul>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    height: 100%;
    padding: var(--space-3) var(--space-2);
    background: var(--bg-base);
    /* §4: hairlines do the work. No shadow on the sidebar. */
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  hr {
    height: 1px;
    margin: var(--space-2) var(--space-2);
    border: none;
    background: var(--border);
  }

  .heading {
    padding: 0 var(--space-3);
    font-size: var(--text-micro);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  button {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    min-height: var(--target-min);
    padding: 0 var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--text-base);
    color: var(--fg-muted);
    text-align: left;
    transition:
      background var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  button:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  /* §8: the glyph is --accent when active. Brass here is *interaction*, not status. */
  button.on {
    background: var(--bg-selected);
    color: var(--accent);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    color: var(--fg-subtle);
  }

  /*
   * The badge counts findings, so it is --warn rather than brass: §2 forbids the accent from
   * meaning "attention". It carries a number, which is the second channel beside the colour.
   */
  .badge {
    min-width: 18px;
    padding: 0 var(--space-2);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--warn) 18%, transparent);
    font-size: var(--text-micro);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-medium);
    color: var(--warn);
    text-align: center;
  }
</style>
