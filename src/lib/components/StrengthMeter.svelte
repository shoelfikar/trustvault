<script lang="ts">
  /**
   * MASTER.md §2: **four discrete segments, not a rainbow gradient**, always paired with the
   * word and the crack-time estimate in text.
   *
   * Two details are the prototype's and not this file's invention. The filled segments all take
   * one colour chosen by how many there are — 1–2 danger, 3 warn, 4 ok — rather than a per-index
   * ramp, so a two-segment bar is not half-reassuring. And an empty segment is
   * `--border-strong`, where §2's prose says `--fg-subtle`: at 4px tall, `--fg-subtle` reads as
   * a *filled* segment and the meter stops being countable. Logged as D-34.
   *
   * The brass accent is deliberately absent. §2 reserves it for brand and interaction — it never
   * means "good", and a strength meter is the single most tempting place to break that rule.
   */
  import type { Strength } from '../ipc';

  interface Props {
    strength: Strength | null;
    /** Fixed segment-block width. The prototype uses 180px in the detail pane, 150px in dialogs. */
    width?: string;
    /** Replaces the crack-time text, for the kinds that are not scored at all. */
    note?: string;
  }

  const { strength, width = '', note = '' }: Props = $props();

  /**
   * zxcvbn scores 0–4; the meter has four segments. 0 and 1 both fill one segment, which is
   * why the label collapses them to "Weak" too — the two agree rather than the bar showing
   * nothing for a password that has been typed.
   */
  const filled = $derived(strength && strength.label ? Math.max(1, strength.score) : 0);

  const tone = $derived(
    filled === 0 ? 'empty' : filled <= 2 ? 'danger' : filled === 3 ? 'warn' : 'ok',
  );
</script>

<div class="meter">
  <div
    class="segments"
    style={width ? `width:${width};flex:none` : ''}
    role="meter"
    aria-valuemin={0}
    aria-valuemax={4}
    aria-valuenow={filled}
    aria-valuetext={strength?.label || 'No password entered'}
    aria-label="Password strength"
  >
    {#each [1, 2, 3, 4] as segment (segment)}
      <span class="segment {segment <= filled ? tone : 'empty'}"></span>
    {/each}
  </div>

  {#if strength?.label}
    <!-- Word first, colour second. Status is never colour alone. -->
    <span class="label {tone}">{strength.label}</span>
  {/if}
  <span class="time">{note || strength?.crackTime || ''}</span>
</div>

<style>
  .meter {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .segments {
    display: flex;
    flex: 1;
    gap: 3px;
  }

  .segment {
    flex: 1;
    height: 4px;
    border-radius: 2px;
    background: var(--border-strong);
    transition: background var(--dur-instant) var(--ease-out);
  }
  .segment.danger {
    background: var(--danger);
  }
  .segment.warn {
    background: var(--warn);
  }
  .segment.ok {
    background: var(--ok);
  }

  .label {
    flex: none;
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
  }
  .label.danger {
    color: var(--danger);
  }
  .label.warn {
    color: var(--warn);
  }
  .label.ok {
    color: var(--ok);
  }

  .time {
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    color: var(--fg-muted);
  }
</style>
