<script lang="ts">
  /**
   * The three-pane shell — `MASTER.md` §6. Titlebar, sidebar, item list, detail.
   *
   * Pane widths are persisted through the settings file (D-33), because §10 asks for them to
   * be restored and they are no more secret than the theme. Native window decorations, per §6:
   * custom chrome on Windows and Linux costs more bugs than it buys.
   */
  import { untrack } from 'svelte';
  import Icon from '../icons/Icon.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import DetailPane from './DetailPane.svelte';
  import ItemList from './ItemList.svelte';
  import Sidebar, { type Filter } from './Sidebar.svelte';
  import {
    asIpcError,
    listItems,
    lock,
    setSettings,
    type ItemSummary,
    type Settings,
    type VaultStatus,
  } from '../ipc';

  interface Props {
    status: VaultStatus;
    settings: Settings;
    onsettings: (next: Settings) => void;
  }

  const { status, settings, onsettings }: Props = $props();

  let items = $state<ItemSummary[]>([]);
  let filter = $state<Filter>({ kind: 'all' });
  let selectedId = $state<string | null>(null);
  let query = $state('');
  let error = $state('');

  /**
   * Live pane widths, committed to the settings file when a drag ends.
   *
   * Seeded from `settings` once by design — during a drag this is the authority and the
   * settings object is stale until the drag commits. Reading it reactively would fight the
   * pointer, so the initial-value capture is the intent, not an oversight.
   */
  let sidebarWidth = $state(untrack(() => settings.sidebarWidth));
  let listWidth = $state(untrack(() => settings.listWidth));

  $effect(() => {
    void listItems()
      .then((loaded) => (items = loaded))
      .catch((thrown) => (error = asIpcError(thrown).message));
  });

  const visible = $derived.by(() => {
    const matchesFilter = (item: ItemSummary) => {
      switch (filter.kind) {
        case 'all':
          return true;
        case 'favourites':
          return item.favourite;
        case 'type':
          return item.kind === filter.type;
        case 'tag':
          return item.tags.includes(filter.tag);
        case 'watchtower':
          return ['weak', 'reused', 'breached', 'expired'].includes(item.status);
      }
    };
    const needle = query.trim().toLowerCase();
    return items.filter(
      (item) =>
        matchesFilter(item) &&
        (!needle ||
          item.title.toLowerCase().includes(needle) ||
          item.tags.some((tag) => tag.toLowerCase().includes(needle))),
    );
  });

  /** Clamped to `MASTER.md` §4's ranges, which the tokens also name. */
  const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

  /**
   * Pointer-driven resize.
   *
   * Written against pointer events with capture rather than a library: a splitter is twenty
   * lines, and D-09 already rejected pulling in a component set whose defaults `MASTER.md`
   * bans. Keyboard resizing is deliberately **not** here — it is a Phase 3 keyboard-audit
   * item, and claiming it now would put a tick against S-08 that nothing earned.
   */
  function drag(which: 'sidebar' | 'list', event: PointerEvent) {
    event.preventDefault();
    const start = event.clientX;
    const from = which === 'sidebar' ? sidebarWidth : listWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);

    const move = (moved: PointerEvent) => {
      const next = from + (moved.clientX - start);
      if (which === 'sidebar') sidebarWidth = clamp(next, 180, 320);
      else listWidth = clamp(next, 240, 460);
    };
    const done = () => {
      target.releasePointerCapture(event.pointerId);
      target.removeEventListener('pointermove', move);
      target.removeEventListener('pointerup', done);
      // Written once, at the end of the drag. Persisting on every pointermove would be a file
      // write per frame.
      const next = { ...settings, sidebarWidth, listWidth };
      void setSettings(next).then(onsettings);
    };
    target.addEventListener('pointermove', move);
    target.addEventListener('pointerup', done);
  }

  const emptyMessage = $derived(
    items.length === 0
      ? 'No items in this vault yet.'
      : query.trim()
        ? `Nothing matches “${query.trim()}”.`
        : filter.kind === 'watchtower'
          ? 'Watchtower has nothing to report.'
          : 'Nothing here yet.',
  );
</script>

<div class="shell">
  <!-- §6: native decorations, so this is a toolbar inside the window rather than window chrome. -->
  <header class="titlebar">
    <span class="mark"><Icon name="vault" size={15} /></span>
    <span class="name">{status.displayName}</span>

    <div class="search">
      <Icon name="search" size={14} />
      <input
        type="search"
        placeholder="Search items"
        aria-label="Search items"
        bind:value={query}
      />
    </div>

    <button type="button" class="tool" aria-label="Settings" disabled title="Settings — Phase 3">
      <Icon name="settings" size={15} />
    </button>
    <button type="button" class="tool" aria-label="Lock vault" onclick={() => void lock()}>
      <Icon name="lock" size={15} />
    </button>
  </header>

  <div class="panes">
    <div class="pane" style="width: {sidebarWidth}px">
      <Sidebar {items} {filter} onfilter={(next) => (filter = next)} />
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="splitter"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize sidebar"
      onpointerdown={(event) => drag('sidebar', event)}
    ></div>

    <div class="pane list" style="width: {listWidth}px">
      {#if error}
        <EmptyState icon="alert" message={error} />
      {:else if visible.length === 0}
        <!-- §7: never a blank pane. Every list has an empty state. -->
        <EmptyState icon="list" message={emptyMessage} />
      {:else}
        <ItemList items={visible} {selectedId} onselect={(id) => (selectedId = id)} />
      {/if}
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="splitter"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize item list"
      onpointerdown={(event) => drag('list', event)}
    ></div>

    <DetailPane itemId={selectedId} />
  </div>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-surface);
  }

  .titlebar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: var(--titlebar-h);
    flex: none;
    padding: 0 var(--space-3);
    background: var(--bg-base);
    border-bottom: 1px solid var(--border);
  }
  .mark {
    display: flex;
    color: var(--accent);
  }
  .name {
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
    white-space: nowrap;
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: 1;
    max-width: 320px;
    margin: 0 auto;
    padding: 0 var(--space-3);
    height: 24px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg-subtle);
    transition: border-color var(--dur-instant) var(--ease-out);
  }
  .search:focus-within {
    border-color: var(--accent);
    color: var(--fg-muted);
  }
  .search input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .search input:focus {
    outline: none;
  }

  .tool {
    display: flex;
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    color: var(--fg-subtle);
    transition: color var(--dur-instant) var(--ease-out);
  }
  .tool:hover:not(:disabled) {
    color: var(--fg);
  }
  .tool:disabled {
    opacity: 0.4;
  }
  .tool:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .panes {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .pane {
    flex: none;
    min-width: 0;
  }
  .pane.list {
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border-right: 1px solid var(--border);
  }

  /* A 5px grab area over a 1px hairline: the border stays the visual, the target is usable. */
  .splitter {
    flex: none;
    width: 5px;
    margin: 0 -2px;
    cursor: col-resize;
    background: transparent;
    z-index: 1;
  }
  .splitter:hover {
    background: color-mix(in srgb, var(--accent) 30%, transparent);
  }

  /* Below 900px the detail becomes an overlay sheet — §6. The list keeps the space until
     then, which is why this collapses the two fixed panes rather than the flexible one. */
  @media (max-width: 900px) {
    .pane {
      width: auto !important;
      flex: 1;
    }
    .splitter {
      display: none;
    }
  }
</style>
