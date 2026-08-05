<script lang="ts">
  /**
   * Vault navigation with counts, tags, Watchtower and the vault footer — `MASTER.md` §6.
   *
   * The counts come from the item list the shell already holds, not from a second command.
   * Asking the host again would be a second source of truth for the same number, and the two
   * would disagree the first time a filter changed.
   *
   * The footer is the prototype's profile row with honest content. There is no account and no
   * sync (D-03), so an avatar and an email would be invented; it carries the *vault* instead —
   * initials, name, file and item count — and opens the vault switcher, which is what a footer
   * in that position is for.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import type { ItemKind, ItemSummary } from '../ipc';
  import { isFlagged, sameView, tagColour, type View } from './views';

  interface Props {
    items: ItemSummary[];
    view: View;
    vaultName: string;
    vaultFile: string;
    onview: (next: View) => void;
    onvaults: () => void;
  }

  const { items, view, vaultName, vaultFile, onview, onvaults }: Props = $props();

  /** The three types the design's sidebar names. The other four live under All Items. */
  const types: { type: ItemKind; label: string; icon: IconName }[] = [
    { type: 'login', label: 'Login', icon: 'key' },
    { type: 'card', label: 'Cards', icon: 'card' },
    { type: 'note', label: 'Notes', icon: 'note' },
  ];

  const favourites = $derived(items.filter((item) => item.favourite).length);
  const tags = $derived(
    [...new Set(items.flatMap((item) => item.tags))].sort((a, b) => a.localeCompare(b)),
  );
  const flagged = $derived(items.filter(isFlagged).length);

  const countOf = (type: ItemKind) => items.filter((item) => item.kind === type).length;
  const countTag = (tag: string) => items.filter((item) => item.tags.includes(tag)).length;

  const initials = $derived(
    vaultName
      .trim()
      .split(/\s+/)
      .map((word) => word[0] ?? '')
      .slice(0, 2)
      .join('')
      .toUpperCase() || 'TV',
  );
</script>

<nav class="sidebar sb" aria-label="Vault">
  <p class="heading">Vault</p>
  <ul>
    <li>
      <button class:on={sameView(view, { kind: 'all' })} onclick={() => onview({ kind: 'all' })}>
        <Icon name="list" size={16} />
        <span class="text">All Items</span>
        <span class="count">{items.length}</span>
      </button>
    </li>
    <li>
      <button
        class:on={sameView(view, { kind: 'favourites' })}
        onclick={() => onview({ kind: 'favourites' })}
      >
        <Icon name="star" size={16} />
        <span class="text">Favorites</span>
        <span class="count">{favourites}</span>
      </button>
    </li>
    {#each types as entry (entry.type)}
      <li>
        <button
          class:on={sameView(view, { kind: 'type', type: entry.type })}
          onclick={() => onview({ kind: 'type', type: entry.type })}
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
            class:on={sameView(view, { kind: 'tag', tag })}
            onclick={() => onview({ kind: 'tag', tag })}
          >
            <span class="dot-slot"
              ><span class="dot" style="background:{tagColour(tag)}"></span></span
            >
            <span class="text">{tag}</span>
            <span class="count">{countTag(tag)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <hr />

  <ul>
    <li>
      <button
        class:on={sameView(view, { kind: 'watchtower' })}
        onclick={() => onview({ kind: 'watchtower' })}
      >
        <Icon name="shield" size={16} />
        <span class="text">Watchtower</span>
        {#if flagged > 0}
          <span class="badge">{flagged}</span>
        {/if}
      </button>
    </li>
    <li>
      <button
        class:on={sameView(view, { kind: 'trash' })}
        onclick={() => onview({ kind: 'trash' })}
      >
        <Icon name="trash" size={16} />
        <span class="text">Trash</span>
      </button>
    </li>
  </ul>

  <div class="spacer"></div>

  <div class="footer">
    <button class="vault" onclick={onvaults} title="Switch vault">
      <span class="avatar">{initials}</span>
      <span class="lines">
        <span class="vault-name">{vaultName}</span>
        <span class="vault-meta">{vaultFile} · {items.length} items</span>
      </span>
      <span class="chev"><Icon name="chev" size={13} /></span>
    </button>
  </div>
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--space-3) 0;
    background: var(--bg-base);
    /* §4: hairlines do the work. No shadow on the sidebar. */
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }

  .heading {
    padding: 6px var(--space-5) var(--space-2);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 0 var(--space-3);
  }

  hr {
    height: 1px;
    margin: 10px var(--space-5);
    border: none;
    background: var(--border);
  }

  button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: var(--control-h);
    padding: 0 var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--text-base);
    color: var(--fg);
    text-align: left;
    transition:
      background var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  button:hover {
    background: var(--bg-hover);
  }
  /* §8: the glyph is --accent when active. Brass here is *interaction*, not status. */
  button.on,
  button.on:hover {
    background: var(--accent-wash);
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
    font-size: var(--text-micro);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg-subtle);
  }

  .dot-slot {
    display: grid;
    place-items: center;
    width: 16px;
    flex: none;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 2px;
  }

  /*
   * The badge counts findings, so it is --warn rather than brass: §2 forbids the accent from
   * meaning "attention". It carries a number, which is the second channel beside the colour.
   */
  .badge {
    padding: 0 var(--space-2);
    border: 1px solid var(--warn);
    border-radius: 3px;
    font-size: var(--text-micro);
    line-height: 15px;
    font-family: var(--font-mono);
    font-weight: var(--weight-medium);
    font-variant-numeric: tabular-nums;
    color: var(--warn);
  }

  .spacer {
    flex: 1;
    min-height: var(--space-4);
  }

  .footer {
    border-top: 1px solid var(--border);
  }
  .vault {
    height: auto;
    padding: var(--space-3) var(--space-5);
    border-radius: 0;
    gap: var(--space-3);
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    flex: none;
    /* §4: --radius-full is for avatars only, and this is the one. */
    border-radius: var(--radius-full);
    background: var(--accent-wash);
    font-size: var(--text-micro);
    font-weight: var(--weight-semibold);
    color: var(--accent);
  }
  .lines {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .vault-name {
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .vault-meta {
    font-size: var(--text-micro);
    color: var(--fg-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chev {
    display: flex;
    flex: none;
    color: var(--fg-subtle);
  }
</style>
