<script lang="ts">
  /**
   * The command palette — `MASTER.md` §7 calls it "the primary navigation surface".
   *
   * §7 also fixes the two keys, and they are not the obvious way round: **Enter copies the
   * password**, ⇧Enter opens the item. That is the whole point of the surface — the common act
   * is "give me the password for X", and making it two keystrokes is what stops the app from
   * being a place you visit.
   *
   * Copy goes through `copy_field`, so the secret is written to the clipboard by Rust and the
   * clear is scheduled by Rust. This file never sees a value; it looks up which field to copy
   * and hands the host two ids. That round trip is the reason `run` is async.
   */
  import { tick } from 'svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import { asIpcError, copyField, getItem, searchItems, type ItemSummary } from '../ipc';
  import { TYPE_GLYPHS, type View } from './views';

  interface Props {
    onclose: () => void;
    onopen: (id: string) => void;
    onview: (next: View) => void;
    onlock: () => void;
    ongenerate: () => void;
    onadd: () => void;
    /** Reports a scheduled clipboard clear so the shell can draw the countdown. */
    oncopied: (clearsAt: number) => void;
  }

  const { onclose, onopen, onview, onlock, ongenerate, onadd, oncopied }: Props = $props();

  /** How many item rows the palette draws, and therefore how many it asks the host for. */
  const ROWS = 6;

  let query = $state('');
  let cursor = $state(0);
  let error = $state('');
  let matched = $state<ItemSummary[]>([]);

  interface Row {
    key: string;
    title: string;
    sub: string;
    icon: IconName;
    hint: string;
    accent: boolean;
    run: () => void | Promise<void>;
  }

  /**
   * Copy the item's password without ever holding it.
   *
   * The field is chosen by kind, then by "first secret" — an item with no password at all (a
   * secure note) copies its first secret field, which is what somebody reaching for ⌘K on a
   * note actually wants.
   */
  async function copyPassword(id: string) {
    try {
      const detail = await getItem(id);
      const field =
        detail.fields.find((candidate) => candidate.kind === 'password') ??
        detail.fields.find((candidate) => candidate.secret);
      if (!field) {
        error = 'This item has nothing secret to copy.';
        return;
      }
      const { clearsAt } = await copyField(id, field.id);
      oncopied(clearsAt);
      onclose();
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

  const needle = $derived(query.trim().toLowerCase());

  /**
   * Which query the rows on screen belong to.
   *
   * A round trip per keystroke means two answers can be in flight at once, and the slower one
   * must not win: typing `gi` then `git` would otherwise leave `gi`'s rows under a cursor bound
   * to Enter, which copies a password. The counter is the guard — a reply for anything but the
   * latest request is dropped.
   */
  let issued = 0;

  /**
   * Ask the host to rank the vault — R-16, D-46.
   *
   * Every keystroke crosses IPC and nothing is filtered here, because nothing here has the
   * usernames and URLs R-16 asks to search. The `performance` marks are S-04's measurement
   * instrument: the criterion is *keystroke to filtered results rendered*, so the mark is set
   * before the call and the measure is taken after `tick()`, once Svelte has flushed the rows.
   * Read them from the devtools console with
   * `performance.getEntriesByName('palette-keystroke-to-render')`. The host half of the same
   * path is measured by `cargo bench --bench search`.
   */
  $effect(() => {
    const asked = ++issued;
    const text = query;
    performance.mark('palette-keystroke');
    void searchItems(text, ROWS)
      .then(async (hits) => {
        if (asked !== issued) return;
        matched = hits;
        error = '';
        await tick();
        performance.measure('palette-keystroke-to-render', 'palette-keystroke');
      })
      .catch((thrown) => {
        if (asked !== issued) return;
        matched = [];
        error = asIpcError(thrown).message;
      });
  });

  const commands = $derived(
    (
      [
        { title: 'Lock vault', icon: 'lock', hint: '⌘L', run: onlock },
        { title: 'Generate password', icon: 'refresh', hint: '⌘G', run: ongenerate },
        {
          title: 'Open Watchtower',
          icon: 'shield',
          hint: '',
          run: () => onview({ kind: 'watchtower' }),
        },
        { title: 'New item', icon: 'plus', hint: '⌘N', run: onadd },
        {
          title: 'Settings',
          icon: 'settings',
          hint: '',
          run: () => onview({ kind: 'settings' }),
        },
      ] satisfies { title: string; icon: IconName; hint: string; run: () => void }[]
    ).filter((command) => !needle || command.title.toLowerCase().includes(needle)),
  );

  const itemRows = $derived<Row[]>(
    matched.map((item) => ({
      key: `item:${item.id}`,
      title: item.title,
      sub: item.tags.join(' · '),
      icon: TYPE_GLYPHS[item.kind],
      hint: '↵ copy password',
      accent: false,
      run: () => copyPassword(item.id),
    })),
  );

  const commandRows = $derived<Row[]>(
    commands.map((command) => ({
      key: `cmd:${command.title}`,
      title: command.title,
      sub: '',
      icon: command.icon,
      hint: command.hint,
      accent: true,
      run: () => {
        command.run();
        onclose();
      },
    })),
  );

  const rows = $derived([...itemRows, ...commandRows]);

  $effect(() => {
    // Keep the cursor inside the list as it shrinks under a query.
    void rows.length;
    if (cursor >= rows.length) cursor = Math.max(0, rows.length - 1);
  });

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      cursor = rows.length ? (cursor + 1) % rows.length : 0;
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      cursor = rows.length ? (cursor - 1 + rows.length) % rows.length : 0;
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const row = rows[cursor];
      if (!row) return;
      // §7: ⇧Enter opens the item, plain Enter copies. Commands ignore the modifier.
      if (event.shiftKey && row.key.startsWith('item:')) {
        onopen(row.key.slice('item:'.length));
        onclose();
      } else {
        void row.run();
      }
    }
  }
</script>

<Dialog width={600} maxHeight={440} align="top" bare {onclose}>
  <div class="search">
    <Icon name="search" size={16} />
    <!-- svelte-ignore a11y_autofocus -- the palette exists to be typed into; it is opened by an
         explicit keystroke and focusing anything else would be the surprise. -->
    <input
      bind:value={query}
      placeholder="Search items, tags, or commands"
      aria-label="Search items, tags, or commands"
      autofocus
      spellcheck="false"
      {onkeydown}
    />
    <span class="esc">esc</span>
  </div>

  <div class="results sb">
    {#if itemRows.length}
      <p class="group">Items</p>
      {#each itemRows as row, index (row.key)}
        <button
          class="row"
          class:on={cursor === index}
          onmouseenter={() => (cursor = index)}
          onclick={() => void row.run()}
        >
          <span class="glyph"><Icon name={row.icon} size={16} /></span>
          <span class="title">{row.title}</span>
          <span class="sub">{row.sub}</span>
          <span class="grow"></span>
          <span class="hint">{cursor === index ? row.hint : ''}</span>
        </button>
      {/each}
    {/if}

    {#if commandRows.length}
      <p class="group">Commands</p>
      {#each commandRows as row, index (row.key)}
        {@const at = itemRows.length + index}
        <button
          class="row"
          class:on={cursor === at}
          onmouseenter={() => (cursor = at)}
          onclick={() => void row.run()}
        >
          <span class="glyph accent"><Icon name={row.icon} size={16} /></span>
          <span class="title">{row.title}</span>
          <span class="grow"></span>
          <span class="hint">{row.hint}</span>
        </button>
      {/each}
    {/if}

    {#if rows.length === 0}
      <p class="none">No results for “{query.trim()}”</p>
    {/if}

    {#if error}
      <p class="error"><Icon name="alert" size={13} />{error}</p>
    {/if}
  </div>

  <div class="hints">
    <span><Icon name="enter" size={12} />Copy password</span>
    <span>⇧↵ Open item</span>
    <span>↑↓ Navigate</span>
  </div>
</Dialog>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    height: 44px;
    padding: 0 14px;
    border-bottom: 1px solid var(--border);
    color: var(--fg-muted);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: none;
    background: none;
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--text-md);
  }
  .search input:focus {
    outline: none;
  }
  .search input::placeholder {
    color: var(--fg-subtle);
  }
  .esc {
    padding: 0 var(--space-2);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: var(--text-micro);
    line-height: 15px;
    color: var(--fg-subtle);
  }

  .results {
    flex: 1;
    min-height: 0;
    padding: 6px 0;
    overflow-y: auto;
  }

  .group {
    padding: 6px 14px 3px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 11px;
    width: 100%;
    height: 34px;
    padding: 0 14px;
    text-align: left;
    transition: background var(--dur-instant) var(--ease-out);
  }
  /* The cursor is a fill and not a ring: in a palette there is exactly one active row and it is
     driven by the arrow keys, so §7's "focus distinct from selection" has nothing to separate. */
  .row.on {
    background: var(--bg-hover);
  }
  .row:focus-visible {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }

  .glyph {
    display: flex;
    color: var(--fg-muted);
  }
  /* Brass on a command glyph is *interaction* — it marks the rows that do something rather
     than the rows that are something. §2. */
  .glyph.accent {
    color: var(--accent);
  }

  .title {
    font-size: var(--text-base);
    white-space: nowrap;
  }
  .sub {
    font-size: var(--text-sm);
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .grow {
    flex: 1;
  }
  .hint {
    font-size: var(--text-micro);
    color: var(--fg-subtle);
    white-space: nowrap;
  }

  .none {
    padding: var(--space-7);
    text-align: center;
    font-size: var(--text-base);
    color: var(--fg-subtle);
  }
  .error {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-3);
    font-size: var(--text-sm);
    color: var(--danger);
  }

  .hints {
    display: flex;
    align-items: center;
    gap: 14px;
    flex: none;
    height: 32px;
    padding: 0 14px;
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
    font-size: var(--text-micro);
    color: var(--fg-subtle);
  }
  .hints span {
    display: flex;
    align-items: center;
    gap: 5px;
  }
</style>
