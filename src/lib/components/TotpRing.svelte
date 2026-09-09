<script lang="ts">
  /**
   * The countdown ring beside a one-time code — R-20, and the prototype's own geometry.
   *
   * Two circles on a 24-unit viewBox, r=9, 2.2 stroke: a `--border-strong` track and a brass
   * arc over it, rotated -90° so it starts at twelve o'clock. Brass is right here for the
   * reason §2 permits it and no other — this is an interaction in progress, not a verdict. A
   * ring that turned red near zero would be using colour to mean "bad", which §2 bans, and it
   * would be wrong anyway: a code about to roll over is not a problem, it is a clock.
   *
   * **What is not the prototype's:** the prototype drives the arc with
   * `animation: totp 30s linear infinite`, a free-running CSS loop that starts whenever the
   * element mounts. Ours is driven by `expiresAt` from the host, because the loop is wrong in
   * the way that matters — the user is looking at their phone at the same time, and a ring that
   * says nine seconds next to a phone that says twenty-two is a ring nobody trusts again. The
   * appearance is identical; only what moves it is different.
   *
   * The seconds are drawn as text beside the arc, which is §2's "status is never colour alone"
   * applied to something that is not strictly a status: the arc alone cannot be read to the
   * second, and it cannot be read at all by someone who cannot see it.
   */
  interface Props {
    /** Epoch-ms the current code stops being valid — the step boundary, from the host. */
    expiresAt: number;
    /** Seconds in a full step, so the arc knows what fraction is left. */
    period: number;
    /** Diameter in px. 20 in the detail pane, 18 in the Add dialog's preview. */
    size?: number;
    /** Whether to draw the remaining seconds beside the ring. */
    showSeconds?: boolean;
  }

  const { expiresAt, period, size = 20, showSeconds = true }: Props = $props();

  /** 2πr for r = 9, which is the dash length the whole arc is measured in. */
  const CIRCUMFERENCE = 56.55;

  let now = $state(Date.now());

  $effect(() => {
    // Four ticks a second: enough that the arc moves smoothly and the digit never appears to
    // skip, cheap enough to leave running while the pane is open. Cleared on teardown, because
    // a timer surviving its component is how a locked vault keeps painting.
    const timer = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(timer);
  });

  const left = $derived(Math.max(0, (expiresAt - now) / 1000));
  const seconds = $derived(Math.ceil(left));
  /**
   * How much of the ring is still drawn.
   *
   * Clamped at both ends: a clock that has drifted past the boundary would otherwise produce a
   * negative dash offset, which renders as a full ring — the most misleading of the possible
   * wrong answers, because it looks like a code that has just refreshed.
   */
  const offset = $derived(CIRCUMFERENCE * (1 - Math.min(1, Math.max(0, left / period))));
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  class="ring"
  role="img"
  aria-label="{seconds} seconds until this code changes"
>
  <circle cx="12" cy="12" r="9" fill="none" stroke="var(--border-strong)" stroke-width="2.2" />
  <circle
    cx="12"
    cy="12"
    r="9"
    fill="none"
    stroke="var(--accent)"
    stroke-width="2.2"
    stroke-linecap="round"
    stroke-dasharray={CIRCUMFERENCE}
    stroke-dashoffset={offset}
  />
</svg>
{#if showSeconds}
  <span class="left" aria-hidden="true">{seconds}s</span>
{/if}

<style>
  .ring {
    flex: none;
    /* Twelve o'clock, so the arc drains the way a clock face reads. */
    transform: rotate(-90deg);
  }

  .left {
    width: 26px;
    font-size: var(--text-sm);
    /* A number that changes every second must not change width while it does — MASTER.md §3. */
    font-variant-numeric: tabular-nums;
    color: var(--fg-muted);
  }
</style>
