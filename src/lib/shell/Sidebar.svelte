<script lang="ts">
  /**
   * Vault navigation with counts, tags, Watchtower and the vault footer — `MASTER.md` §6.
   *
   * The counts come from the item list the shell already holds, not from a second command.
   * Asking the host again would be a second source of truth for the same number, and the two
   * would disagree the first time a filter changed.
   *
   * The footer is the prototype's profile row, and **as of D-70 it carries a profile again**.
   * For two phases it carried the vault instead — there was no account and no sync (D-03), so
   * an avatar and an e-mail would have been invented. What changed is not that argument but
   * what is behind the pixels: a name and an e-mail now live in the sealed body, so the row
   * shows a person when the user has named one and falls back to the vault when they have not.
   * Clicking it opens `ProfileMenu` rather than jumping straight to the vault switcher, which
   * is the prototype's own behaviour and the one thing the old footer got wrong.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import type { ItemKind, ItemSummary, Profile } from '../ipc';
  import { initialsOf } from './profile';
  import { isFlagged, sameView, tagColour, type View } from './views';

  interface Props {
    items: ItemSummary[];
    view: View;
    vaultName: string;
    vaultFile: string;
    /** Who the vault belongs to, or `null` — D-70. Never `null` here: the shell is unlocked. */
    profile: Profile | null;
    /** Whether the popover is open, so the row can hold its pressed background. */
    menuOpen: boolean;
    onview: (next: View) => void;
    /** Opens the popover. The element is passed up because the menu is positioned from it. */
    onmenu: (trigger: HTMLElement) => void;
  }

  const { items, view, vaultName, vaultFile, profile, menuOpen, onview, onmenu }: Props = $props();

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

  /**
   * The person if one is named, the vault otherwise — D-70.
   *
   * Three derivations rather than one, because the fallback is not the same shape on both
   * lines: with no profile the row reads exactly as it did before this existed (vault initials,
   * vault name, file and count), and with one it reads as the design drew it. The file and
   * count stay on the second line in both cases — the prototype puts "Synced 2 min ago" there,
   * which D-03 makes a lie, and the vault's own file is the true thing worth that space.
   */
  const initials = $derived(initialsOf(profile?.name ?? '', initialsOf(vaultName, 'TV')));
  const primary = $derived(profile?.name || vaultName);

  /**
   * The three groups as data, so every entry has an index across the whole sidebar.
   *
   * They are rendered as three lists with a rule between them, which is the design; the roving
   * tab stop below has to run across all of them, and an index that resets per list cannot do
   * that. The footer's *Switch vault* is deliberately **not** in here — see `onkeydown`.
   */
  const mainEntries = $derived([
    {
      key: 'all',
      label: 'All Items',
      icon: 'list' as IconName,
      count: items.length,
      to: { kind: 'all' } as View,
    },
    {
      key: 'favourites',
      label: 'Favorites',
      icon: 'star' as IconName,
      count: favourites,
      to: { kind: 'favourites' } as View,
    },
    ...types.map((entry) => ({
      key: `type:${entry.type}`,
      label: entry.label,
      icon: entry.icon,
      count: countOf(entry.type),
      to: { kind: 'type', type: entry.type } as View,
    })),
  ]);

  const tagEntries = $derived(
    tags.map((tag) => ({
      key: `tag:${tag}`,
      label: tag,
      tag,
      count: countTag(tag),
      to: { kind: 'tag', tag } as View,
    })),
  );

  const bottomEntries = $derived([
    {
      key: 'watchtower',
      label: 'Watchtower',
      icon: 'shield' as IconName,
      badge: flagged,
      to: { kind: 'watchtower' } as View,
    },
    {
      key: 'trash',
      label: 'Trash',
      icon: 'trash' as IconName,
      badge: 0,
      to: { kind: 'trash' } as View,
    },
  ]);

  const entries = $derived([...mainEntries, ...tagEntries, ...bottomEntries]);

  /**
   * One tab stop for the whole sidebar — `docs/keyboard-audit.md` row 6, and the implementation
   * catching up to a line that had described it since the row was written.
   *
   * Measured on 2026-08-08 it was **thirteen** consecutive tab stops, which is a legitimate shape
   * for a nav list and is not what the row asks for: reaching Settings from the titlebar took
   * fifteen Tab presses, thirteen of them in here, and from a chair that is indistinguishable
   * from Tab never arriving.
   *
   * **Arrows move, Enter selects** — and that is the difference from `Segmented`, which selects
   * on arrow because a radiogroup's value *is* the focused option. Moving through eleven views
   * would otherwise re-filter the item list ten times on the way to the eleventh. The buttons
   * are left as buttons in a `<nav>` rather than given `listbox`/`option` roles: the roles here
   * were measured under the global rules already, and swapping them is a semantic change no part
   * of row 6 asks for.
   */
  let roving = $state<number | null>(null);

  const selected = $derived(entries.findIndex((entry) => sameView(view, entry.to)));
  /** Where Tab lands: wherever the arrows left off, else the current view, else the first row. */
  const tabStop = $derived(roving ?? Math.max(0, selected));

  $effect(() => {
    // A view chosen anywhere else — the palette, the titlebar — takes the tab stop back to it.
    void view;
    roving = null;
  });

  function onkeydown(event: KeyboardEvent, index: number) {
    const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (!step) return;
    event.preventDefault();
    const next = (index + step + entries.length) % entries.length;
    roving = next;
    const list = (event.currentTarget as HTMLElement).closest('nav');
    list?.querySelector<HTMLElement>(`[data-nav="${next}"]`)?.focus();
  }
</script>

<nav class="sidebar sb" aria-label="Vault">
  <p class="heading">Vault</p>
  <ul>
    {#each mainEntries as entry, index (entry.key)}
      <li>
        <button
          data-nav={index}
          tabindex={index === tabStop ? 0 : -1}
          class:on={sameView(view, entry.to)}
          onclick={() => onview(entry.to)}
          onkeydown={(event) => onkeydown(event, index)}
        >
          <Icon name={entry.icon} size={16} />
          <span class="text">{entry.label}</span>
          <span class="count">{entry.count}</span>
        </button>
      </li>
    {/each}
  </ul>

  {#if tagEntries.length}
    <hr />
    <p class="heading">Tags</p>
    <ul>
      {#each tagEntries as entry, offset (entry.key)}
        {@const index = mainEntries.length + offset}
        <li>
          <button
            data-nav={index}
            tabindex={index === tabStop ? 0 : -1}
            class:on={sameView(view, entry.to)}
            onclick={() => onview(entry.to)}
            onkeydown={(event) => onkeydown(event, index)}
          >
            <span class="dot-slot"
              ><span class="dot" style="background:{tagColour(entry.tag)}"></span></span
            >
            <span class="text">{entry.label}</span>
            <span class="count">{entry.count}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <hr />

  <ul>
    {#each bottomEntries as entry, offset (entry.key)}
      {@const index = mainEntries.length + tagEntries.length + offset}
      <li>
        <button
          data-nav={index}
          tabindex={index === tabStop ? 0 : -1}
          class:on={sameView(view, entry.to)}
          onclick={() => onview(entry.to)}
          onkeydown={(event) => onkeydown(event, index)}
        >
          <Icon name={entry.icon} size={16} />
          <span class="text">{entry.label}</span>
          {#if entry.badge > 0}
            <span class="badge">{entry.badge}</span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>

  <div class="spacer"></div>

  <div class="footer">
    <button
      class="vault"
      class:open={menuOpen}
      aria-haspopup="menu"
      aria-expanded={menuOpen}
      onclick={(event) => onmenu(event.currentTarget)}
    >
      <span class="avatar">{initials}</span>
      <span class="lines">
        <span class="vault-name">{primary}</span>
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
  /* Held while the popover is up, so the row reads as the thing the menu belongs to. It is
     `--bg-hover` and not `--accent-wash`: this is a pressed control, not a selected view. */
  .vault.open,
  .vault.open:hover {
    background: var(--bg-hover);
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    flex: none;
    /* §4: --radius-full is for avatars only, and this is the one. */
    border-radius: var(--radius-full);
    /*
     * **Opaque, unlike the other two avatars, and that is a contrast fix rather than a style.**
     *
     * `--accent-wash` is translucent, so whatever is behind it changes the colour the initials
     * are read against — and this is the one avatar whose backdrop moves: the row tints to
     * `--bg-hover` on hover and while the popover is open. Measured 2026-08-08 by
     * `npm run a11y`: brass over the wash over `--bg-hover` is **4.09:1** in dark, under §9's
     * 4.5. On the sidebar's own `--bg-base` it is comfortably over.
     *
     * The defect predates the popover — the row has tinted on hover since Phase 2 — and it took
     * D-70's `.open` to expose it, because `element.focus()` can reach a state the audit cannot
     * hover into. Mixing against `--bg-base` freezes the avatar at the colour it always had on
     * the untinted row, so the row can tint underneath it without moving it.
     *
     * The token carries the flattened value per theme; `color-mix` was tried first and cannot
     * be used here, because it computes to `color(srgb …)` and the audit's parser reads that as
     * `rgb(1, 1, 1)` — which turned one finding into eighteen false ones.
     */
    background: var(--accent-chip);
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
  /*
   * Rotated to point **up**, which is the prototype's own `rotate(-90deg)` and is also the
   * direction the menu opens. A right-pointing chevron on a control that opens upward reads as
   * "goes to another screen" — which is what this row used to do, and no longer does.
   */
  .chev {
    display: flex;
    flex: none;
    transform: rotate(-90deg);
    color: var(--fg-subtle);
  }
</style>
