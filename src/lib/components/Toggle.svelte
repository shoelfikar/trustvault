<script lang="ts">
  /**
   * A 34×20 switch. Brass track when on — §2 allows the accent here because a switch is
   * *interaction state*, not a verdict; nothing about "on" claims the setting is the good one.
   *
   * It is a `switch`-role button with `aria-checked`, so the state is announced rather than
   * inferred from which side the knob is on.
   */
  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onchange: (next: boolean) => void;
  }

  const { checked, label, disabled = false, onchange }: Props = $props();
</script>

<button
  type="button"
  role="switch"
  class="toggle"
  class:on={checked}
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    display: flex;
    justify-content: flex-start;
    flex: none;
    width: 34px;
    height: 20px;
    padding: 2px;
    /* `--radius-full`, not a hardcoded 10px. The two paint identically at this height, and one
       of them survives the row getting a pixel taller — found 2026-08-07 while ticking §10's
       "no radius > 6px outside avatars", which is the line the track was quietly breaking. §4
       now names the switch track beside the avatar, because the knob below has always used the
       token and a track that is not a pill is not a switch. */
    border-radius: var(--radius-full);
    background: var(--border-strong);
    transition:
      background var(--dur-instant) var(--ease-out),
      justify-content var(--dur-instant) var(--ease-out);
  }
  .toggle.on {
    justify-content: flex-end;
    background: var(--accent);
  }
  .toggle:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .toggle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .knob {
    width: 16px;
    height: 16px;
    border-radius: var(--radius-full);
    background: var(--bg-raised);
  }
  .toggle.on .knob {
    background: var(--on-accent);
  }
</style>
