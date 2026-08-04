<script lang="ts">
  /**
   * MASTER.md §2: **four discrete segments, not a rainbow gradient.**
   * `--fg-subtle` (empty) → `--danger` → `--warn` → `--ok`, always paired with the word and
   * the crack-time estimate in text.
   *
   * The brass accent is deliberately absent from this component. §2 reserves it for brand and
   * interaction — it never means "good", and a strength meter is the single most tempting
   * place to break that rule.
   */
  import type { Strength } from '../ipc';

  interface Props {
    strength: Strength | null;
  }

  const { strength }: Props = $props();

  /**
   * zxcvbn scores 0–4; the meter has four segments. 0 and 1 both fill one segment, which is
   * why the label collapses them to "Weak" too — the two agree rather than the bar showing
   * nothing for a password that has been typed.
   */
  const filled = $derived(strength && strength.label ? Math.max(1, strength.score) : 0);

  const tone = $derived(
    filled === 0 ? 'empty' : filled <= 1 ? 'danger' : filled === 2 ? 'warn' : 'ok',
  );
</script>

<div class="meter">
  <div
    class="segments"
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

  <p class="readout">
    {#if strength?.label}
      <!-- Word first, colour second. Status is never colour alone. -->
      <span class="label {tone}">{strength.label}</span>
      <span class="time">· {strength.crackTime} to crack offline</span>
    {:else}
      <span class="time">Choose something you can remember and nobody can guess.</span>
    {/if}
  </p>
</div>

<style>
  .meter {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .segments {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 3px;
  }

  .segment {
    height: 3px;
    border-radius: 1px;
    background: var(--bg-hover);
    transition: background var(--dur-instant) var(--ease-out);
  }
  .segment.empty {
    background: var(--bg-hover);
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

  .readout {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
  }
  .label {
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
    color: var(--fg-subtle);
  }
</style>
