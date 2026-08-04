<script lang="ts">
  /**
   * MASTER.md §7. Primary = brass fill, 28px tall, 4px radius, 500 weight — no gradient, no
   * shadow, no hover lift. Secondary = transparent with a 1px `--border-strong`.
   *
   * There is no `destructive` variant here on purpose: §7 puts destructive text on transparent
   * and fills it *only inside a confirm dialog*, so it belongs to the dialog, not to a general
   * button.
   */
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary';
    type?: 'button' | 'submit';
    disabled?: boolean;
    /** Fills the row it sits in — used by the onboarding steps. */
    wide?: boolean;
    onclick?: () => void;
    children: Snippet;
  }

  const {
    variant = 'secondary',
    type = 'button',
    disabled = false,
    wide = false,
    onclick,
    children,
  }: Props = $props();
</script>

<button class="btn {variant}" class:wide {type} {disabled} {onclick}>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: 28px;
    padding: 0 var(--space-4);
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
    white-space: nowrap;
    /* §5: colour transitions only, 120ms. No transform, no shadow. */
    transition:
      background var(--dur-instant) var(--ease-out),
      border-color var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  .btn.wide {
    width: 100%;
  }
  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .primary {
    background: var(--accent);
    color: var(--on-accent);
    border: 1px solid var(--accent);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }

  .secondary {
    background: transparent;
    color: var(--fg);
    border: 1px solid var(--border-strong);
  }
  .secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  /* §7: never remove outlines. 2px accent ring, 2px offset, on every interactive element. */
  .btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  @media (pointer: coarse) {
    .btn {
      height: var(--target-min);
    }
  }
</style>
