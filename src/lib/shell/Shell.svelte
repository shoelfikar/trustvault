<script lang="ts">
  /**
   * The three-pane shell — `MASTER.md` §6. Titlebar, sidebar, item list, detail, plus the two
   * full-width surfaces (Watchtower, Settings) and every overlay.
   *
   * Pane widths are persisted through the settings file (D-33), because §10 asks for them to
   * be restored and they are no more secret than the theme. Native window decorations, per §6:
   * custom chrome on Windows and Linux costs more bugs than it buys — so the prototype's three
   * traffic lights are absent here. They are the operating system's, drawn above this strip.
   *
   * The titlebar's search is a **button, not an input**. §7 makes the command palette the
   * primary navigation surface, and two search affordances that behave differently is how a
   * user learns to trust neither. Clicking it opens ⌘K.
   */
  import { untrack } from 'svelte';
  import Icon from '../icons/Icon.svelte';
  import IconButton from '../components/IconButton.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import DeleteDialog from './DeleteDialog.svelte';
  import DetailPane from './DetailPane.svelte';
  import EditItemDialog from './EditItemDialog.svelte';
  import GeneratorDialog from './GeneratorDialog.svelte';
  import ImportDialog from './ImportDialog.svelte';
  import ItemList from './ItemList.svelte';
  import NewItemDialog from './NewItemDialog.svelte';
  import SettingsPane from './SettingsPane.svelte';
  import Sidebar from './Sidebar.svelte';
  import VaultSwitcher from './VaultSwitcher.svelte';
  import Watchtower from './Watchtower.svelte';
  import {
    asIpcError,
    listItems,
    lock,
    setSettings,
    type ItemSummary,
    type Settings,
    type VaultStatus,
  } from '../ipc';
  import { isFullWidth, matches, viewTitle, type View } from './views';

  interface Props {
    status: VaultStatus;
    settings: Settings;
    onsettings: (next: Settings) => void;
    /**
     * The open vault was switched away from or deleted — R-22, R-18.
     *
     * The host has already locked and zeroized by the time this fires, so what is left is for
     * the router to re-read `vault_status` and stop drawing this shell. It re-reads rather than
     * being told which screen to show: the host owns lock state, and a frontend that decided
     * "so we go to the lock screen now" would be the second source of truth §9 check 6 exists
     * to keep from existing.
     */
    onvaultchanged: () => void;
    /**
     * The user asked for a second vault from the switcher — R-22, D-62.
     *
     * Routed up to the router rather than handled here, for the reason above one line: this
     * shell is what gets replaced by the onboarding flow, and a screen that swaps itself out
     * would be deciding what is on screen instead of the state that owns it.
     */
    oncreatevault: () => void;
  }

  const { status, settings, onsettings, onvaultchanged, oncreatevault }: Props = $props();

  let items = $state<ItemSummary[]>([]);
  let view = $state<View>({ kind: 'all' });
  let selectedId = $state<string | null>(null);
  let error = $state('');
  /** A settings write the host refused. Only `launch_at_login` can produce one. */
  let settingsError = $state('');

  type Overlay =
    | 'none'
    | 'palette'
    | 'generator'
    | 'add'
    | 'edit'
    | 'import'
    | 'vaults'
    | 'deleteItem'
    | 'deleteVault';

  /** Which overlay is up. One at a time — the prototype never stacks two. */
  let overlay = $state<Overlay>('none');

  /**
   * Close an overlay only if it is still the one on screen.
   *
   * One state for every overlay means "run the command, then close me" is two synchronous writes
   * to the same variable, and the close was winning: the palette's *New item* row set
   * `overlay = 'add'` and had it overwritten by `'none'` in the next statement, so the dialog
   * never rendered. *Generate password* was broken identically, while *Lock vault*, *Watchtower*
   * and *Settings* worked because they route through `onlock`/`onview` and never touch `overlay`
   * — three of five working is why it survived, and why the palette looked alive.
   *
   * Reordering the two calls would have fixed the same two rows and left the next one to be
   * written broken, because it would still be ordering that decided. Guarding on identity makes
   * the rule structural: a handler that navigated somewhere keeps where it went, and a close that
   * is only a close still closes. Found on 2026-08-08 in the manual keyboard pass, though the
   * pointer path was identically broken — `docs/keyboard-audit.md`, finding 5.
   */
  function closeOverlay(which: Overlay) {
    if (overlay === which) overlay = 'none';
  }

  /** Epoch-ms the clipboard is scheduled to clear at. Owned here; see DetailPane's note. */
  let clipboardUntil = $state(0);

  /**
   * Bumped after any mutation, which is what makes both panes re-read.
   *
   * The mutation commands return the identifier at most — `update_item` and `delete_item`
   * return nothing at all — because `get_item` is deliberately the one path that decides what
   * may cross (§6.4). The cost of that is exactly this: the frontend never learns the new state
   * from a response, so it asks again.
   */
  let mutations = $state(0);

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
    void mutations;
    void listItems()
      .then((loaded) => (items = loaded))
      .catch((thrown) => (error = asIpcError(thrown).message));
  });

  const visible = $derived(items.filter((item) => matches(view, item)));

  const vaultFile = $derived((status.path ?? '').split(/[/\\]/).pop() || 'vault.tvault');
  const tags = $derived(
    [...new Set(items.flatMap((item) => item.tags))].sort((a, b) => a.localeCompare(b)),
  );

  const selectedTitle = $derived(items.find((item) => item.id === selectedId)?.title ?? '');

  /**
   * Keep the selection inside the current view, and land on the first row when it falls out.
   *
   * The prototype opens with an item already selected, and it is the right default: a detail
   * pane that says "select an item" on every launch spends the app's most valuable pixels on an
   * instruction. Switching views re-selects rather than blanking, for the same reason.
   */
  $effect(() => {
    if (isFullWidth(view)) return;
    if (selectedId && visible.some((item) => item.id === selectedId)) return;
    selectedId = visible[0]?.id ?? null;
  });

  function goto(next: View) {
    view = next;
    if (isFullWidth(next)) selectedId = null;
  }

  function openItem(id: string) {
    if (isFullWidth(view)) view = { kind: 'all' };
    selectedId = id;
  }

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

  /**
   * Saves a settings change, and shows the one that can be refused.
   *
   * `launch_at_login` writes outside this process — a desktop entry, a `LaunchAgent`, a
   * registry value — so the host rejects with `io` and **stores nothing** when the platform
   * will not take it. This side therefore does not update its own copy on failure either: the
   * toggle snaps back to what the host still holds, which is the truth about the machine, and
   * the sentence beside it says why it moved. Optimistically keeping the new value would leave
   * a screen promising the app starts at login when nothing registered it.
   */
  function saveSettings(next: Settings) {
    settingsError = '';
    void setSettings(next)
      .then(onsettings)
      .catch((thrown) => (settingsError = asIpcError(thrown).message));
  }

  /**
   * A new item landed. Select it, and leave a filtered view if it would hide it.
   *
   * The view change is not a nicety: adding a login while the sidebar is on the Cards filter
   * would otherwise save the item and show nothing, which is indistinguishable from the save
   * having failed.
   */
  function itemSaved(itemId: string) {
    overlay = 'none';
    mutations += 1;
    view = { kind: 'all' };
    selectedId = itemId;
  }

  /** An edit landed. Both panes re-read; the selection is already right. */
  function itemChanged() {
    overlay = 'none';
    mutations += 1;
  }

  /**
   * A delete landed. The selection is dropped first, so the detail pane stops asking for an
   * item the vault no longer has — the `$effect` below then lands it on the next row.
   */
  function itemDeleted() {
    overlay = 'none';
    selectedId = null;
    mutations += 1;
  }

  /**
   * §7's shortcuts.
   *
   * Every one of them re-checks nothing about the lock state, because it cannot need to: this
   * component only exists while the host says the vault is unlocked, and the host re-checks on
   * every command regardless of what the frontend believes.
   */
  function onkeydown(event: KeyboardEvent) {
    const meta = event.metaKey || event.ctrlKey;
    const key = event.key.toLowerCase();
    if (meta && key === 'k') {
      event.preventDefault();
      overlay = 'palette';
    } else if (meta && key === 'n') {
      event.preventDefault();
      overlay = 'add';
    } else if (meta && key === 'g') {
      event.preventDefault();
      overlay = 'generator';
    } else if (meta && key === 'l') {
      event.preventDefault();
      void lock();
    } else if (event.key === 'Escape' && overlay !== 'none') {
      overlay = 'none';
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="shell">
  <!-- §6: native decorations, so this is a toolbar inside the window rather than window chrome. -->
  <header class="titlebar">
    <div class="vault">
      <Icon name="vault" size={14} />
      <span class="vault-name">{status.displayName}</span>
    </div>

    <button class="search" onclick={() => (overlay = 'palette')}>
      <Icon name="search" size={13} />
      <span>Search items, tags, or commands</span>
      <span class="grow"></span>
      <span class="kbd">⌘K</span>
    </button>

    <div class="grow"></div>

    <IconButton
      icon="settings"
      label="Settings"
      title="Settings"
      active={view.kind === 'settings'}
      onclick={() => goto({ kind: 'settings' })}
    />
    <button class="lock" onclick={() => void lock()}>
      <Icon name="lock" size={14} />Lock
    </button>
  </header>

  <div class="panes">
    <div class="pane" style="width: {sidebarWidth}px">
      <Sidebar
        {items}
        {view}
        vaultName={status.displayName}
        {vaultFile}
        onview={goto}
        onvaults={() => (overlay = 'vaults')}
      />
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="splitter"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize sidebar"
      onpointerdown={(event) => drag('sidebar', event)}
    ></div>

    {#if view.kind === 'watchtower'}
      <Watchtower {items} onopen={openItem} />
    {:else if view.kind === 'settings'}
      <SettingsPane
        {settings}
        {status}
        itemCount={items.length}
        error={settingsError}
        onchange={saveSettings}
        ondeletevault={() => (overlay = 'deleteVault')}
        onimport={() => (overlay = 'import')}
      />
    {:else}
      <div class="pane" style="width: {listWidth}px">
        <ItemList
          items={visible}
          {view}
          {selectedId}
          {error}
          onselect={(id) => (selectedId = id)}
          ongenerate={() => (overlay = 'generator')}
          onadd={() => (overlay = 'add')}
          onview={goto}
        />
      </div>

      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="splitter"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize item list"
        onpointerdown={(event) => drag('list', event)}
      ></div>

      <DetailPane
        itemId={selectedId}
        listEmpty={visible.length === 0}
        {clipboardUntil}
        reloadSignal={mutations}
        oncopied={(clearsAt) => (clipboardUntil = clearsAt)}
        onedit={() => (overlay = 'edit')}
        ondelete={() => (overlay = 'deleteItem')}
      />
    {/if}
  </div>

  {#if overlay === 'palette'}
    <CommandPalette
      onclose={() => closeOverlay('palette')}
      onopen={openItem}
      onview={goto}
      onlock={() => void lock()}
      ongenerate={() => (overlay = 'generator')}
      onadd={() => (overlay = 'add')}
      oncopied={(clearsAt) => (clipboardUntil = clearsAt)}
    />
  {:else if overlay === 'generator'}
    <GeneratorDialog onclose={() => closeOverlay('generator')} />
  {:else if overlay === 'add'}
    <NewItemDialog
      vaultName={status.displayName}
      {vaultFile}
      {tags}
      onclose={() => closeOverlay('add')}
      onsaved={itemSaved}
    />
  {:else if overlay === 'edit' && selectedId}
    <EditItemDialog
      itemId={selectedId}
      vaultName={status.displayName}
      {vaultFile}
      {tags}
      onclose={() => closeOverlay('edit')}
      onsaved={itemChanged}
    />
  {:else if overlay === 'import'}
    <!-- The dialog stays open after a successful import and shows the commit's report; only
         `mutations` moves here, so the list and the tag sidebar re-read behind it. Closing on
         success would take the refusal list off screen at the moment it becomes permanent —
         it is the one record of what did *not* come across. -->
    <ImportDialog onclose={() => closeOverlay('import')} onimported={() => (mutations += 1)} />
  {:else if overlay === 'vaults'}
    <VaultSwitcher
      openPath={status.path}
      itemCount={items.length}
      onclose={() => closeOverlay('vaults')}
      onswitched={onvaultchanged}
      oncreate={() => {
        closeOverlay('vaults');
        oncreatevault();
      }}
    />
  {:else if overlay === 'deleteItem'}
    <DeleteDialog
      target="item"
      name={selectedTitle}
      itemId={selectedId}
      onclose={() => closeOverlay('deleteItem')}
      ondeleted={itemDeleted}
    />
  {:else if overlay === 'deleteVault'}
    <DeleteDialog
      target="vault"
      name={status.displayName}
      vaultPath={status.path}
      itemCount={items.length}
      onclose={() => closeOverlay('deleteVault')}
      ondeleted={onvaultchanged}
    />
  {/if}

  <!-- The view name is announced when it changes; the panes themselves are static landmarks. -->
  <p class="sr" aria-live="polite">{viewTitle(view)}</p>
</div>

<style>
  .shell {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-surface);
    overflow: hidden;
  }

  .titlebar {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    height: var(--titlebar-h);
    flex: none;
    padding: 0 var(--space-4);
    background: var(--bg-base);
    border-bottom: 1px solid var(--border);
  }

  .vault {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    color: var(--fg-muted);
  }
  .vault-name {
    font-size: var(--text-sm);
    color: var(--fg);
    white-space: nowrap;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: 1;
    max-width: 440px;
    height: 24px;
    margin: 0 auto;
    padding: 0 var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--fg-subtle);
    font-size: var(--text-sm);
    transition: border-color var(--dur-instant) var(--ease-out);
  }
  .search:hover {
    border-color: var(--border-strong);
  }
  .search:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }
  .kbd {
    padding: 0 var(--space-2);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: var(--text-micro);
    line-height: 15px;
  }

  .grow {
    flex: 1;
  }

  .lock {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    height: var(--control-h);
    padding: 0 9px;
    border-radius: var(--radius-sm);
    color: var(--fg-muted);
    font-size: var(--text-sm);
    transition:
      background var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  .lock:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .lock:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
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

  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
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
