<script lang="ts">
  /**
   * A Watchtower verdict, rendered as an **icon and a word** — never colour alone.
   *
   * `MASTER.md` §2 calls this mandatory and names the DB anti-pattern it comes from
   * (`color-only-indicators`). The icon is not decoration beside the label; the two together
   * are what make the status readable to someone who cannot distinguish the colours.
   *
   * Brass is absent on purpose. §2 reserves `--accent` for brand and interaction — it never
   * means "good", and a status chip is where that rule is most often broken.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import type { ItemStatus } from '../ipc';

  interface Props {
    status: ItemStatus;
    /** Pip mode drops the word — used only where a label cannot fit, and never alone. */
    compact?: boolean;
  }

  const { status, compact = false }: Props = $props();

  /**
   * `unknown` renders as nothing at all rather than as a fourth colour.
   *
   * It is the state every item starts in, and drawing it would put a pip on every row of a
   * vault that has never been scanned — which reads as a warning about nothing.
   */
  const presentation: Record<ItemStatus, { icon: IconName; label: string; tone: string } | null> = {
    unknown: null,
    strong: { icon: 'shield-check', label: 'Strong', tone: 'ok' },
    weak: { icon: 'alert', label: 'Weak', tone: 'warn' },
    reused: { icon: 'copy', label: 'Reused', tone: 'warn' },
    breached: { icon: 'shield', label: 'Breached', tone: 'danger' },
    expired: { icon: 'clock', label: 'Expired', tone: 'warn' },
  };

  const shown = $derived(presentation[status]);
</script>

{#if shown}
  <span class="chip {shown.tone}" class:compact title={compact ? shown.label : undefined}>
    <Icon name={shown.icon} size={12} label={compact ? shown.label : undefined} />
    {#if !compact}<span>{shown.label}</span>{/if}
  </span>
{/if}

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    height: 18px;
    padding: 0 var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--text-micro);
    line-height: var(--text-micro-lh);
    font-weight: var(--weight-medium);
    white-space: nowrap;
  }
  .chip.compact {
    padding: 0;
    background: none;
  }

  /* Desaturated backgrounds: §2 keeps an all-red Watchtower from reading as a fire alarm. */
  .ok {
    color: var(--ok);
    background: color-mix(in srgb, var(--ok) 14%, transparent);
  }
  .warn {
    color: var(--warn);
    background: color-mix(in srgb, var(--warn) 14%, transparent);
  }
  .danger {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
  }
</style>
