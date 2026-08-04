<script lang="ts">
  /**
   * MASTER.md §7. Primary = brass fill, 28px tall, 4px radius, 500 weight — no gradient, no
   * shadow, no hover lift. Secondary = transparent with a 1px `--border-strong`. Ghost is the
   * same shape with no border, which the prototype uses for the "or…" affordances.
   *
   * There is no `destructive` variant here on purpose: §7 puts destructive text on transparent
   * and fills it *only inside a confirm dialog*, so it belongs to the dialog, not to a general
   * button. `outlineDanger` is the un-filled half of that rule — a bordered danger button — and
   * it is used by the vault-deletion card, which is a warning surface and not the dialog.
   */
  import type { Snippet } from 'svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost' | 'outlineDanger';
    type?: 'button' | 'submit';
    disabled?: boolean;
    /** Fills the row it sits in — used by the onboarding steps and the lock card. */
    wide?: boolean;
    /** Taller 32px form control, used where the button sits in a column of 32px inputs. */
    tall?: boolean;
    icon?: IconName;
    iconSize?: number;
    title?: string;
    onclick?: () => void;
    children: Snippet;
  }

  const {
    variant = 'secondary',
    type = 'button',
    disabled = false,
    wide = false,
    tall = false,
    icon,
    iconSize = 14,
    title,
    onclick,
    children,
  }: Props = $props();
</script>

<button class="btn {variant}" class:wide class:tall {type} {title} {disabled} {onclick}>
  {#if icon}<Icon name={icon} size={iconSize} />{/if}
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: var(--control-h);
    padding: 0 var(--space-4);
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
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
  .btn.tall {
    height: 32px;
    padding: 0 var(--space-5);
    font-size: var(--text-base);
  }
  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .primary {
    background: var(--accent);
    color: var(--on-accent);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .secondary {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border-strong);
  }
  .secondary:hover:not(:disabled) {
    color: var(--fg);
    border-color: var(--fg-subtle);
  }

  .ghost {
    background: transparent;
    color: var(--accent);
  }
  .ghost:hover:not(:disabled) {
    background: var(--accent-wash);
  }

  .outlineDanger {
    background: transparent;
    color: var(--danger);
    border-color: var(--danger);
  }
  .outlineDanger:hover:not(:disabled) {
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
