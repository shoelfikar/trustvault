<script lang="ts">
  /**
   * The settings segmented control — a 1px bordered track with a brass-wash pill on the
   * selected option.
   *
   * It is a `radiogroup`, not a row of buttons: arrow keys move between options and only the
   * selected one is a tab stop, which is what a keyboard user expects from a control shaped
   * like this and what §7's "full keyboard operation" costs here.
   */
  interface Option<T> {
    value: T;
    label: string;
  }

  interface Props<T> {
    options: Option<T>[];
    value: T;
    label: string;
    disabled?: boolean;
    onchange: (next: T) => void;
  }

  // svelte-ignore non_reactive_update
  let { options, value, label, disabled = false, onchange }: Props<unknown> = $props();

  /**
   * Which option carries the group's single tab stop.
   *
   * The obvious form of this — `tabindex={option.value === value ? 0 : -1}` — has a failure mode
   * that is silent and total: when `value` matches **no** option, every button is `-1` and the
   * whole control drops out of the tab order while still being visible, still being clickable,
   * and still passing a focus audit, because `element.focus()` reaches a `-1` button perfectly
   * well. It is not hypothetical. The screenshot/a11y fixture was missing `ui_scale`, so Interface
   * size had been unreachable by keyboard in every harness run since D-57, and no tool noticed.
   *
   * Falling back to the first option is what ARIA prescribes for a radiogroup with nothing
   * checked, so the group stays reachable and the arrows still work from there.
   */
  const tabStop = $derived(
    Math.max(
      0,
      options.findIndex((option) => option.value === value),
    ),
  );

  function onkeydown(event: KeyboardEvent, index: number) {
    const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
    if (!step) return;
    event.preventDefault();
    const next = options[(index + step + options.length) % options.length];
    if (!next) return;
    onchange(next.value);
    const buttons = (event.currentTarget as HTMLElement).parentElement?.children;
    (buttons?.[options.indexOf(next)] as HTMLElement | undefined)?.focus();
  }
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each options as option, index (option.label)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      tabindex={index === tabStop ? 0 : -1}
      class:on={option.value === value}
      {disabled}
      onclick={() => onchange(option.value)}
      onkeydown={(event) => onkeydown(event, index)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: flex;
    gap: 2px;
    flex: none;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: 5px;
  }

  button {
    height: 22px;
    padding: 0 9px;
    border-radius: 3px;
    font-size: var(--text-sm);
    color: var(--fg-muted);
    white-space: nowrap;
    transition:
      background var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  button:hover:not(:disabled):not(.on) {
    color: var(--fg);
  }
  /* Brass here is *interaction* — which option is engaged — not a verdict about it. §2. */
  button.on {
    background: var(--accent-wash);
    color: var(--accent);
  }
  button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }
</style>
