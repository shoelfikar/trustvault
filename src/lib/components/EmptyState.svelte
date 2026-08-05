<script lang="ts">
  /**
   * MASTER.md §7: **every list has one.** Line-art icon, one sentence, one primary action.
   * "Never a blank pane."
   *
   * The action is optional here for one honest reason: some empty states have no action yet,
   * because the thing that would fill them arrives in a later phase. An empty state with a
   * button that does nothing is worse than one that explains and waits.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import Button from './Button.svelte';

  interface Props {
    icon: IconName;
    /** One sentence. Not a paragraph — the pane is not a place to explain the product. */
    message: string;
    actionLabel?: string;
    actionDisabled?: boolean;
    actionTitle?: string;
    onaction?: () => void;
    /** The detail pane's variant: smaller glyph, no action, no padding block. */
    quiet?: boolean;
  }

  const {
    icon,
    message,
    actionLabel,
    actionDisabled = false,
    actionTitle,
    onaction,
    quiet = false,
  }: Props = $props();
</script>

<div class="empty" class:quiet>
  <span class="glyph"><Icon name={icon} size={quiet ? 30 : 34} /></span>
  <p>{message}</p>
  {#if actionLabel}
    <Button
      variant="primary"
      tall={false}
      disabled={actionDisabled}
      title={actionTitle}
      onclick={onaction}
    >
      {actionLabel}
    </Button>
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
    max-width: 200px;
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }
  .quiet {
    gap: 10px;
  }
  .quiet p {
    color: var(--fg-subtle);
  }
</style>
