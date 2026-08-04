<script lang="ts">
  /**
   * MASTER.md §7: **every list has one.** Line-art icon, one sentence, one primary action.
   * "Never a blank pane."
   *
   * The action is optional here for one honest reason: some empty states in Phase 2 have no
   * action yet, because the thing that would fill them arrives in Phase 3. An empty state with
   * a button that does nothing is worse than one that explains and waits.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import Button from './Button.svelte';

  interface Props {
    icon: IconName;
    /** One sentence. Not a paragraph — the pane is not a place to explain the product. */
    message: string;
    actionLabel?: string;
    onaction?: () => void;
  }

  const { icon, message, actionLabel, onaction }: Props = $props();
</script>

<div class="empty">
  <span class="glyph"><Icon name={icon} size={28} /></span>
  <p>{message}</p>
  {#if actionLabel && onaction}
    <Button variant="primary" onclick={onaction}>{actionLabel}</Button>
  {/if}
</div>

<style>
  .empty {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    /* §4: empty states get the breathing room the rest of the app does not. */
    padding: var(--space-7);
    text-align: center;
  }
  .glyph {
    display: flex;
    color: var(--fg-subtle);
  }
  p {
    max-width: 26ch;
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-subtle);
  }
</style>
