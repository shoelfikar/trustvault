<script lang="ts">
  import Specimen from './lib/screens/Specimen.svelte';
  import { Lock, Palette } from '@lucide/svelte';

  /**
   * Phase 0 shell. This is scaffolding, not the app — the real three-pane shell arrives
   * in Phase 2 (see phases/phase-2-shell-unlock.md). Everything here exists to prove the
   * tokens, fonts, icons, and theme switching work end to end.
   */

  type Theme = 'dark' | 'light';
  type UiScale = 'compact' | 'default' | 'large';

  let theme = $state<Theme>('dark');
  let uiScale = $state<UiScale>('default');

  $effect(() => {
    document.documentElement.dataset.theme = theme;
  });
  $effect(() => {
    document.documentElement.dataset.uiScale = uiScale;
  });

  const scales: UiScale[] = ['compact', 'default', 'large'];
</script>

<div class="chrome">
  <div class="brand">
    <Lock size={15} strokeWidth={1.6} color="var(--accent)" />
    <span class="wordmark">TrustVault</span>
    <span class="phase">Phase 0 · Workbench</span>
  </div>

  <div class="spacer"></div>

  <div class="segmented" role="group" aria-label="UI scale">
    {#each scales as s (s)}
      <button
        type="button"
        class:on={uiScale === s}
        aria-pressed={uiScale === s}
        onclick={() => (uiScale = s)}>{s}</button
      >
    {/each}
  </div>

  <button
    type="button"
    class="theme"
    onclick={() => (theme = theme === 'dark' ? 'light' : 'dark')}
    aria-label="Switch to {theme === 'dark' ? 'light' : 'dark'} theme"
  >
    <Palette size={14} strokeWidth={1.6} />
    {theme}
  </button>
</div>

<main>
  <Specimen {theme} {uiScale} />
</main>

<style>
  .chrome {
    height: var(--titlebar-h);
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: 0 var(--space-4);
    background: var(--bg-base);
    border-bottom: 1px solid var(--border);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .wordmark {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-lg);
  }
  .phase {
    font-size: var(--text-micro);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }
  .spacer {
    flex: 1;
  }

  .segmented {
    display: flex;
    gap: 2px;
    padding: 3px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
  }
  .segmented button {
    padding: 3px var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--fg-muted);
    text-transform: capitalize;
    transition:
      background var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  .segmented button:hover {
    color: var(--fg);
  }
  .segmented button.on {
    background: var(--accent-wash);
    color: var(--accent);
  }

  .theme {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 26px;
    padding: 0 var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    font-size: var(--text-sm);
    color: var(--fg-muted);
    text-transform: capitalize;
    transition:
      color var(--dur-instant) var(--ease-out),
      border-color var(--dur-instant) var(--ease-out);
  }
  .theme:hover {
    color: var(--fg);
    border-color: var(--border-strong);
  }

  main {
    height: calc(100vh - var(--titlebar-h));
    background: var(--bg-surface);
  }
</style>
