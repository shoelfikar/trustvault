<script lang="ts">
  /**
   * A Watchtower verdict, rendered as an **icon and a word** — never colour alone.
   *
   * `MASTER.md` §2 calls this mandatory and names the DB anti-pattern it comes from
   * (`color-only-indicators`). In `pip` mode the word is dropped, which the item list needs and
   * which would break that rule on its own — it does not, because each status carries a
   * *different glyph*, and the glyph is the second channel. A pip is also given an accessible
   * name so the label is still there for anyone not reading pixels.
   *
   * Brass is absent on purpose. §2 reserves `--accent` for brand and interaction — it never
   * means "good", and a status chip is where that rule is most often broken.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import type { ItemStatus } from '../ipc';

  interface Props {
    status: ItemStatus;
    /** Drops the word, for the item list where a label cannot fit. */
    pip?: boolean;
    size?: number;
  }

  const { status, pip = false, size = 13 }: Props = $props();

  /**
   * `unknown` renders as nothing at all rather than as a fourth colour.
   *
   * It is the state every item starts in, and drawing it would put a pip on every row of a
   * vault that has never been scanned — which reads as a warning about nothing.
   *
   * `strong` is likewise absent from the *list*, because the prototype only flags what is
   * wrong; the detail pane passes `pip={false}` and gets the full chip including "Strong".
   */
  const presentation: Record<
    ItemStatus,
    { icon: IconName; label: string; tone: 'ok' | 'warn' | 'danger' } | null
  > = {
    unknown: null,
    strong: { icon: 'shield-check', label: 'Strong', tone: 'ok' },
    weak: { icon: 'alert', label: 'Weak', tone: 'warn' },
    reused: { icon: 'alert', label: 'Reused', tone: 'warn' },
    breached: { icon: 'shield', label: 'Breached', tone: 'danger' },
    expired: { icon: 'clock', label: 'Expired', tone: 'warn' },
  };

  const shown = $derived(presentation[status]);
  const visible = $derived(shown && !(pip && status === 'strong'));
</script>

{#if shown && visible}
  {#if pip}
    <span class="pip {shown.tone}" title={shown.label}>
      <Icon name={shown.icon} size={14} label={shown.label} />
    </span>
  {:else}
    <span class="chip {shown.tone}">
      <Icon name={shown.icon} {size} />
      <span>{shown.label}</span>
    </span>
  {/if}
{/if}

<style>
  /* The detail pane's chip: an outline in the status colour, never a fill. §2 keeps status
     surfaces desaturated so an all-red Watchtower does not read as a fire alarm. */
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: none;
    height: var(--control-h);
    padding: 0 9px;
    border: 1px solid currentcolor;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    white-space: nowrap;
  }

  .pip {
    display: flex;
    flex: none;
  }

  .ok {
    color: var(--ok);
  }
  .warn {
    color: var(--warn);
  }
  .danger {
    color: var(--danger);
  }
</style>
