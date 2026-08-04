<script lang="ts">
  /**
   * A square icon-only control — toolbar, field row, dialog close.
   *
   * `MASTER.md` §8's colour ramp is the whole component: `--fg-muted` at rest, `--fg` on hover,
   * `--accent` when active. The `tone` prop exists for the two places the ramp ends somewhere
   * else — delete goes to `--danger`, a completed copy goes to `--ok` — and nowhere else, so a
   * third tone is a decision rather than a prop value.
   *
   * `label` is required, not optional. An icon with no text beside it is the one case where a
   * missing accessible name is invisible until somebody tries to use the app without a screen.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';

  interface Props {
    icon: IconName;
    /** Accessible name; also the tooltip unless `title` overrides it. */
    label: string;
    title?: string;
    size?: number;
    tone?: 'default' | 'danger' | 'ok' | 'accent';
    active?: boolean;
    disabled?: boolean;
    onclick?: (event: MouseEvent) => void;
  }

  const {
    icon,
    label,
    title,
    size = 15,
    tone = 'default',
    active = false,
    disabled = false,
    onclick,
  }: Props = $props();
</script>

<button
  type="button"
  class="icon-button {tone}"
  class:active
  title={title ?? label}
  aria-label={label}
  {disabled}
  {onclick}
>
  <Icon name={icon} {size} />
</button>

<style>
  .icon-button {
    display: grid;
    place-items: center;
    flex: none;
    width: var(--control-h);
    height: var(--control-h);
    border-radius: var(--radius-sm);
    color: var(--fg-muted);
    transition:
      background var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  .icon-button:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .icon-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .icon-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .icon-button.active {
    color: var(--accent);
  }
  .danger:hover:not(:disabled) {
    color: var(--danger);
  }
  .ok {
    color: var(--ok);
  }
  .ok:hover:not(:disabled) {
    color: var(--ok);
  }
  .accent {
    color: var(--accent);
  }
  .accent:hover:not(:disabled) {
    background: var(--accent-wash);
    color: var(--accent);
  }
</style>
