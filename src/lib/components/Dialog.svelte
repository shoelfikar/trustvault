<script lang="ts">
  /**
   * The modal shell every dialog in the prototype shares: a scrim, a raised card with a 40px
   * header strip, and a `--bg-surface` footer.
   *
   * Two behaviours belong here rather than in each caller. **Esc closes**, because §7 says so
   * for the palette and there is no reason for a dialog to be the exception. And **focus is
   * trapped inside the card while it is open** — D-09 rejected a component library, so the trap
   * is twenty lines here rather than a dependency, and it is the reason `MASTER.md` §7's "full
   * keyboard operation" survives a modal.
   *
   * The scrim is `rgba(8,7,6,.5)` in the prototype rather than a token: it is the *desk* seen
   * through a dim, and no token names it. It is written as a colour-mix over `--desk` so it
   * still tracks the theme.
   */
  import type { Snippet } from 'svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import IconButton from './IconButton.svelte';

  interface Props {
    /** Header strip title. Omit for the bare card the delete confirm uses. */
    title?: string;
    icon?: IconName;
    /** Right-aligned text in the header strip — "Step 1 of 2", the vault name. */
    meta?: string;
    width?: number;
    /** Scrolls the body and caps the card, for the tall New-item dialog. */
    maxHeight?: number;
    /** Aligns the card to the top, which is where the command palette sits. */
    align?: 'center' | 'top';
    /** Drops the padding, for a body that draws its own. */
    bare?: boolean;
    /**
     * Hides the header's close button. The recovery flow uses it: the dialog owns a
     * multi-step transaction, so leaving happens through its own Back, not through an X that
     * would abandon a half-spent recovery key with no explanation.
     */
    closable?: boolean;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  }

  const {
    title = '',
    icon,
    meta = '',
    width = 440,
    maxHeight,
    align = 'center',
    bare = false,
    closable = true,
    onclose,
    children,
    footer,
  }: Props = $props();

  let card = $state<HTMLElement | null>(null);

  const FOCUSABLE =
    'a[href],button:not([disabled]),input:not([disabled]),select,textarea,[tabindex]:not([tabindex="-1"])';

  function onkeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onclose();
      return;
    }
    if (event.key !== 'Tab' || !card) return;
    const stops = [...card.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
      (node) => node.offsetParent !== null,
    );
    if (stops.length === 0) return;
    const first = stops[0]!;
    const last = stops[stops.length - 1]!;
    const active = document.activeElement;
    if (event.shiftKey && (active === first || !card.contains(active))) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  $effect(() => {
    // Move focus in on open. Without it, Tab from the trigger lands behind the scrim.
    const target = card?.querySelector<HTMLElement>(FOCUSABLE);
    target?.focus();
  });
</script>

<svelte:window {onkeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -- Esc is handled on the window above, which
     is the keyboard equivalent of clicking the scrim; a keyboard handler on the backdrop itself
     would be unreachable. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="scrim" class:top={align === 'top'} onclick={onclose}>
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div
    class="card"
    bind:this={card}
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-label={title || undefined}
    style="width:{width}px{maxHeight ? `;max-height:${maxHeight}px` : ''}"
    onclick={(event) => event.stopPropagation()}
  >
    {#if title}
      <header>
        {#if icon}<span class="glyph"><Icon name={icon} size={15} /></span>{/if}
        <span class="title">{title}</span>
        {#if meta}<span class="meta">{meta}</span>{/if}
        {#if closable}<IconButton icon="x" label="Close" onclick={onclose} />{/if}
      </header>
    {/if}

    <div class="body" class:bare class:scrolls={Boolean(maxHeight)}>
      {@render children()}
    </div>

    {#if footer}
      <footer>{@render footer()}</footer>
    {/if}
  </div>
</div>

<style>
  .scrim {
    position: absolute;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--desk) 55%, transparent);
    animation: scrim-in var(--dur-enter) var(--ease-out);
  }
  .scrim.top {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 96px;
  }

  @keyframes scrim-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .card {
    display: flex;
    flex-direction: column;
    max-width: calc(100vw - var(--space-7));
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    /* The one surface entitled to a shadow — §4: shadows are for true overlays only. */
    box-shadow: var(--shadow-dialog);
    overflow: hidden;
    animation: pop-in var(--dur-enter) var(--ease-enter);
  }

  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.99);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  /* §5: reduced motion drops to opacity-only. */
  @media (prefers-reduced-motion: reduce) {
    .card {
      animation: scrim-in var(--dur-enter) var(--ease-out);
    }
  }

  header {
    display: flex;
    align-items: center;
    gap: 9px;
    flex: none;
    height: var(--toolbar-h);
    padding: 0 var(--space-3) 0 var(--space-5);
    border-bottom: 1px solid var(--border);
  }
  .glyph {
    display: flex;
    color: var(--fg-muted);
  }
  .title {
    flex: 1;
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
  }
  .meta {
    font-size: var(--text-micro);
    font-family: var(--font-mono);
    color: var(--fg-subtle);
  }

  .body {
    padding: var(--space-5);
  }
  .body.bare {
    padding: 0;
  }
  .body.scrolls {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  /* A bare capped body owns its own scroll region — the command palette keeps a fixed search
     strip and a fixed hint strip while only the results move. */
  .body.bare.scrolls {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  footer {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: none;
    padding: var(--space-4) var(--space-5);
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
  }
</style>
