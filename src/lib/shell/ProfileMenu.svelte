<script lang="ts">
  /**
   * The sidebar footer's dropdown — the prototype's 216px popover, with the two rows removed
   * that had nothing behind them.
   *
   * The design draws five: *Profile & account*, *Switch vault*, *Settings*, *Lock vault*, and
   * *Sign out of TrustVault*. Four survive. **Sign out is deleted**, not disabled: D-03 put
   * sync out of scope and there is no account, so there is nothing to sign out *of* — this is
   * the D-49/D-55/D-61 rule for the fifth time, a control that promises what v1 does not ship
   * is deleted and the reason stays where the control was. *Profile & account* becomes
   * **Edit profile…**, which is what it does here: the prototype's row navigated to Settings,
   * where a second button opened the dialog this one opens directly.
   *
   * **A menu, not a dialog.** It gets `role="menu"` and arrow keys rather than `Dialog.svelte`'s
   * focus trap, because a trap is for a surface you must finish or cancel and this is a surface
   * you glance at. What it borrows from `Dialog` is global rule 4's other half — Esc closes and
   * **focus goes back to the trigger**, since a menu that drops focus on `<body>` strands the
   * user at the top of the window exactly as a dialog would.
   *
   * It is rendered by `Shell.svelte` rather than inside `Sidebar.svelte` and positioned from the
   * trigger's measured rect. The sidebar scrolls, so a popover parented inside it would be
   * clipped by its own overflow; and the position is measured rather than the prototype's
   * `bottom: 56px` because R-21's interface scale moves the footer, which would make any
   * constant wrong at two of its three settings.
   */
  import { untrack } from 'svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';

  interface Row {
    key: string;
    label: string;
    icon: IconName;
    /** The shortcut that reaches the same place, or '' when there is none. */
    hint: string;
    /** Draws the hairline above this row — the prototype's one separator. */
    separated?: boolean;
    run: () => void;
  }

  interface Props {
    /** The footer button. Measured for placement, and given focus back on close. */
    anchor: HTMLElement;
    /** Empty when the vault has no owner named, which is every vault until one is typed. */
    name: string;
    email: string;
    initials: string;
    /** What the second line falls back to when no profile has been filled in. */
    vaultName: string;
    onclose: () => void;
    oneditprofile: () => void;
    onvaults: () => void;
    onsettings: () => void;
    onlock: () => void;
  }

  const {
    anchor,
    name,
    email,
    initials,
    vaultName,
    onclose,
    oneditprofile,
    onvaults,
    onsettings,
    onlock,
  }: Props = $props();

  /**
   * Placed above the trigger, aligned to its left edge.
   *
   * `position: fixed` in viewport coordinates, so it is not clipped by the sidebar's scroll
   * container nor by the shell's `overflow: hidden`. The 6px gap is the only constant, and a
   * gap is the one measurement that does not need to scale.
   */
  const placement = untrack(() => {
    const rect = anchor.getBoundingClientRect();
    const bottom = Math.round(window.innerHeight - rect.top + 6);
    return `left:${Math.round(rect.left)}px;bottom:${bottom}px`;
  });

  /** The name to show, and the reason the vault's name is the fallback rather than a blank. */
  const heading = $derived(name || vaultName);
  const subheading = $derived(email || 'No profile set — this vault only');

  const rows: Row[] = $derived([
    {
      key: 'profile',
      label: name ? 'Edit profile…' : 'Add your profile…',
      icon: 'user',
      hint: '',
      run: oneditprofile,
    },
    { key: 'vaults', label: 'Switch vault…', icon: 'vault', hint: '', run: onvaults },
    { key: 'settings', label: 'Settings', icon: 'settings', hint: '⌘,', run: onsettings },
    {
      key: 'lock',
      label: 'Lock vault',
      icon: 'lock',
      hint: '⌘L',
      separated: true,
      run: onlock,
    },
  ]);

  let menu = $state<HTMLElement | null>(null);

  /** Focus lands on the first row, which is what makes the menu operable without a pointer. */
  $effect(() => {
    menu?.querySelector<HTMLElement>('[data-row]')?.focus();
  });

  $effect(() => {
    return () => {
      // Global rule 4's other half. `isConnected` for the reason `Dialog.svelte` gives: the
      // trigger can be gone by the time the menu closes, and focusing a detached node silently
      // focuses `<body>` — the outcome this exists to prevent.
      if (anchor.isConnected) anchor.focus();
    };
  });

  /** Runs a row and closes. Written once so no row can forget the second half. */
  function choose(row: Row) {
    onclose();
    row.run();
  }

  function onkeydown(event: KeyboardEvent, index: number) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onclose();
      return;
    }
    const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (!step) return;
    event.preventDefault();
    const next = (index + step + rows.length) % rows.length;
    menu?.querySelector<HTMLElement>(`[data-row="${next}"]`)?.focus();
  }
</script>

<svelte:window onresize={onclose} />

<!-- svelte-ignore a11y_click_events_have_key_events -- Esc is handled on every row, which is the
     keyboard equivalent of clicking away; a handler on the transparent catcher is unreachable. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="catcher" onclick={onclose}></div>

<div class="menu" bind:this={menu} role="menu" aria-label="Profile and vault" style={placement}>
  <div class="head">
    <span class="avatar">{initials}</span>
    <div class="lines">
      <p class="name">{heading}</p>
      <p class="email">{subheading}</p>
    </div>
  </div>

  <div class="rows">
    {#each rows as row, index (row.key)}
      <button
        data-row={index}
        role="menuitem"
        class:separated={row.separated}
        onclick={() => choose(row)}
        onkeydown={(event) => onkeydown(event, index)}
      >
        <Icon name={row.icon} size={15} />
        <span class="label">{row.label}</span>
        {#if row.hint}<span class="hint">{row.hint}</span>{/if}
      </button>
    {/each}
  </div>
</div>

<style>
  /* Transparent, full-window, and below the menu: a click anywhere else closes it. */
  .catcher {
    position: fixed;
    inset: 0;
    z-index: 30;
  }

  .menu {
    position: fixed;
    z-index: 31;
    width: 216px;
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    /* §4: a shadow is for a true overlay, and this is one. */
    box-shadow: var(--shadow-popover);
    overflow: hidden;
    animation: pop-in var(--dur-enter) var(--ease-enter);
  }

  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(4px) scale(0.99);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  /* §5: reduced motion drops to opacity alone. */
  @media (prefers-reduced-motion: reduce) {
    .menu {
      animation: fade-in var(--dur-enter) var(--ease-out);
    }
    @keyframes fade-in {
      from {
        opacity: 0;
      }
      to {
        opacity: 1;
      }
    }
  }

  .head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: var(--space-4) 14px;
    border-bottom: 1px solid var(--border);
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    flex: none;
    /* §4: --radius-full is for avatars only, and this is one. */
    border-radius: var(--radius-full);
    background: var(--accent-wash);
    font-size: var(--text-base);
    font-weight: var(--weight-semibold);
    color: var(--accent);
  }
  .lines {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .email {
    font-size: var(--text-micro);
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rows {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: var(--space-2);
  }

  button {
    display: flex;
    align-items: center;
    gap: 9px;
    height: var(--control-h);
    padding: 0 9px;
    border-radius: var(--radius-sm);
    color: var(--fg);
    font-size: var(--text-base);
    text-align: left;
    transition: background var(--dur-instant) var(--ease-out);
  }
  button:hover {
    background: var(--bg-hover);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  /* The prototype's one separator, above Lock vault. Drawn as a border on the row rather than
     an <hr>, so the 1px gap between rows stays the only spacing rule in here. */
  button.separated {
    margin-top: var(--space-2);
    border-top: 1px solid var(--border);
    border-radius: 0 0 var(--radius-sm) var(--radius-sm);
  }

  .label {
    flex: 1;
  }
  .hint {
    font-size: var(--text-micro);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg-subtle);
  }
</style>
