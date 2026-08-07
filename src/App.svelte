<script lang="ts">
  /**
   * The app's only router. One window with screens, not a site with routes.
   *
   * **Routing is driven by `vault_status`, which the host owns.** The frontend never decides
   * it is unlocked: it asks. That is the same property `docs/ipc-contract.md` §9 check 6
   * tests from the Rust side, expressed here as the absence of any local `unlocked` boolean.
   * A webview reload therefore lands back on whatever the host says, which for a locked vault
   * is the lock screen.
   */
  import LockScreen from './lib/screens/LockScreen.svelte';
  import Onboarding from './lib/screens/Onboarding.svelte';
  import Shell from './lib/shell/Shell.svelte';
  import {
    getSettings,
    onVaultLocked,
    vaultStatus,
    type LockReason,
    type Settings,
    type VaultStatus,
  } from './lib/ipc';

  let status = $state<VaultStatus | null>(null);
  let settings = $state<Settings | null>(null);
  let lockReason = $state<LockReason | null>(null);

  /**
   * The one screen the host cannot ask for — D-62.
   *
   * `vault_status` answers `no_vault` exactly once in a vault's life, so onboarding was
   * unreachable ever after and the switcher's *New vault* had nowhere to go. This flag is the
   * user having asked for that screen, and it is deliberately the **only** thing on this side
   * that decides what is drawn: it can show the create flow, and it cannot show the shell. Lock
   * state stays the host's — §9 check 6 — because creating a vault is a request, not a claim
   * about whether one is open.
   */
  let creating = $state(false);

  async function refresh() {
    status = await vaultStatus();
  }

  $effect(() => {
    void refresh();
    void getSettings().then((loaded) => (settings = loaded));

    // The host announces every lock, including the ones the user did not ask for. Re-reading
    // status rather than assuming keeps the single source of truth single.
    const unlisten = onVaultLocked((reason) => {
      lockReason = reason;
      void refresh();
    });
    return () => void unlisten.then((stop) => stop());
  });

  /**
   * R-28 — follow the OS, with a manual override.
   *
   * `tokens.css` already handles the follow half through `:root:not([data-theme])`, so the
   * override is expressed by *removing* the attribute rather than by resolving "system" to a
   * concrete theme here. Resolving it in JS would mean the OS changing theme while the app is
   * open does nothing until a reload, which is the half of R-28 that is easy to lose.
   */
  $effect(() => {
    const root = document.documentElement;
    if (!settings || settings.theme === 'system') {
      delete root.dataset.theme;
    } else {
      root.dataset.theme = settings.theme;
    }
  });

  /**
   * R-21 — the interface scale, applied to the root and nowhere else.
   *
   * `tokens.css` derives every size from `--ui-scale`, so setting one attribute here moves
   * text, row heights, controls and spacing **together**. That is the whole point of it living
   * in the tokens: a scale implemented by growing the type alone gives you large text in rows
   * that did not grow with it, which is worse than not scaling at all.
   *
   * Set on `documentElement` rather than on the app's own root because the lock screen, the
   * onboarding flow and every dialog are children of `<body>` — a scale that stopped at the
   * shell would be a setting that only applies once you are inside.
   */
  $effect(() => {
    const root = document.documentElement;
    if (!settings || settings.uiScale === 'default') {
      delete root.dataset.uiScale;
    } else {
      root.dataset.uiScale = settings.uiScale;
    }
  });
</script>

{#if !status}
  <!-- One frame at most: vault_status is a memory read. No skeleton shimmer -- MASTER.md §5
       forbids theatre over a local read that finishes in microseconds. -->
  <div class="boot"></div>
{:else if status.state === 'no_vault' || creating}
  <!-- `oncancel` is passed only for the second case, which is what draws step 1's Cancel: at
       first launch there is nothing to go back to, and a Cancel that lands on an empty window
       is worse than no Cancel. -->
  <Onboarding
    ondone={() => {
      creating = false;
      void refresh();
    }}
    oncancel={creating ? () => (creating = false) : undefined}
  />
{:else if status.state === 'locked'}
  <LockScreen
    path={status.path ?? ''}
    displayName={status.displayName}
    reason={lockReason}
    autoLockSeconds={settings?.autoLockSeconds ?? 300}
    onunlocked={() => {
      lockReason = null;
      void refresh();
    }}
  />
{:else if settings}
  <Shell
    {status}
    {settings}
    onsettings={(next) => (settings = next)}
    onvaultchanged={refresh}
    oncreatevault={() => (creating = true)}
  />
{/if}

<style>
  .boot {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-surface);
    font-size: var(--text-base);
    color: var(--fg);
  }
</style>
