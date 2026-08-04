<script lang="ts">
  /**
   * The bordered note with a 2px coloured left edge — onboarding's "cannot be recovered"
   * warning and the recovery flow's "the old kit stops working" warning.
   *
   * The left edge is the only place a status colour appears; the text stays `--fg-muted`. §2
   * asks for desaturated status surfaces, and a fully tinted panel for a sentence the user will
   * read once is exactly the "fire alarm" this app is not supposed to be.
   */
  import type { Snippet } from 'svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';

  interface Props {
    tone?: 'warn' | 'danger' | 'ok' | 'info';
    icon?: IconName;
    children: Snippet;
  }

  const { tone = 'warn', icon = 'alert', children }: Props = $props();
</script>

<div class="callout {tone}">
  <span class="glyph"><Icon name={icon} size={15} /></span>
  <p>{@render children()}</p>
</div>

<style>
  .callout {
    display: flex;
    gap: 10px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-left: 2px solid currentcolor;
    border-radius: var(--radius-sm);
  }
  .warn {
    color: var(--warn);
  }
  .danger {
    color: var(--danger);
  }
  .ok {
    color: var(--ok);
  }
  .info {
    color: var(--info);
  }

  .glyph {
    display: flex;
    flex: none;
    margin-top: 1px;
  }

  p {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }
</style>
