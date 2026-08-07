<script lang="ts">
  /**
   * Settings — the prototype's grouped rows, carrying the four values D-33 actually persists.
   *
   * The prototype's Profile card is a *vault* card here. There is no account, no email and no
   * sync (D-03), so a profile would be three invented fields on the one screen where a user
   * goes to check what is true about their vault. The geometry is unchanged; the content is the
   * vault's own.
   *
   * **UI scale** and **Launch at login** were absent rather than disabled until 2026-08-06,
   * because the host had nowhere to keep them and a control that forgets on relaunch is worse
   * than one that is not there. Both are here now, and the second is the only control in the
   * application whose change can be **refused by the operating system** — see `error` below.
   */
  import Button from '../components/Button.svelte';
  import Icon from '../icons/Icon.svelte';
  import Segmented from '../components/Segmented.svelte';
  import Toggle from '../components/Toggle.svelte';
  import type { Settings, Theme, UiScale, VaultStatus } from '../ipc';

  interface Props {
    settings: Settings;
    status: VaultStatus;
    itemCount: number;
    /** Rejected by the host — today only `launch_at_login`, which writes outside this process. */
    error?: string;
    onchange: (next: Settings) => void;
    ondeletevault: () => void;
    onimport: () => void;
  }

  const {
    settings,
    status,
    itemCount,
    error = '',
    onchange,
    ondeletevault,
    onimport,
  }: Props = $props();

  const fileName = $derived((status.path ?? '').split(/[/\\]/).pop() || 'vault.tvault');

  const initials = $derived(
    status.displayName
      .trim()
      .split(/\s+/)
      .map((word) => word[0] ?? '')
      .slice(0, 2)
      .join('')
      .toUpperCase() || 'TV',
  );

  const facts = $derived([
    { k: 'Items stored', v: `${itemCount} items` },
    { k: 'Sync', v: 'Offline — this device only' },
    { k: 'Vault file', v: fileName },
  ]);

  const themes: { value: Theme; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
  ];

  const lockChoices = [
    { value: 60, label: '1 min' },
    { value: 300, label: '5 min' },
    { value: 900, label: '15 min' },
  ];

  const clipChoices = [
    { value: 10, label: '10s' },
    { value: 12, label: '12s' },
    { value: 30, label: '30s' },
  ];

  /* The percentages are in the labels, not only in the names. "Compact" alone asks the user to
     find out by trying it, and this is a control they change once. */
  const scaleChoices: { value: UiScale; label: string }[] = [
    { value: 'compact', label: 'Compact' },
    { value: 'default', label: 'Default' },
    { value: 'large', label: 'Large' },
  ];

  const patch = (next: Partial<Settings>) => onchange({ ...settings, ...next });
</script>

<div class="settings sb">
  <div class="column">
    <h1>Settings</h1>

    <p class="group-label">Vault</p>
    <div class="card vault">
      <div class="vault-head">
        <span class="avatar">{initials}</span>
        <div class="vault-titles">
          <p class="vault-name">{status.displayName}</p>
          <p class="vault-path">{status.path ?? ''}</p>
        </div>
        <Button disabled title="Renaming arrives with the mutation commands">Rename vault</Button>
      </div>
      <div class="facts">
        {#each facts as fact, index (fact.k)}
          <div class="fact" class:first={index === 0}>
            <p class="fact-k">{fact.k}</p>
            <p class="fact-v">{fact.v}</p>
          </div>
        {/each}
      </div>
    </div>

    <p class="group-label">Appearance</p>
    <div class="card">
      <div class="row first">
        <div class="row-text">
          <p class="row-label">Theme</p>
          <p class="row-desc">System follows this computer's light and dark setting.</p>
        </div>
        <Segmented
          label="Theme"
          options={themes}
          value={settings.theme}
          onchange={(next) => patch({ theme: next as Theme })}
        />
      </div>
      <div class="row">
        <div class="row-text">
          <p class="row-label">Interface size</p>
          <!-- The percentages belong in the description rather than in the chip labels: three
               words read as three sizes, and "Compact 92%" in a 40px chip does not fit at
               115% scale, which is exactly the setting that would break it. -->
          <p class="row-desc">
            Scales everything together — text, rows and controls. Compact 92 %, Default 100 %, Large
            115 %.
          </p>
        </div>
        <Segmented
          label="Interface size"
          options={scaleChoices}
          value={settings.uiScale}
          onchange={(next) => patch({ uiScale: next as UiScale })}
        />
      </div>
    </div>

    <p class="group-label">Security</p>
    <div class="card">
      <div class="row first">
        <div class="row-text">
          <p class="row-label">Auto-lock</p>
          <p class="row-desc">When idle for:</p>
        </div>
        <Segmented
          label="Auto-lock"
          options={lockChoices}
          value={settings.autoLockSeconds}
          onchange={(next) => patch({ autoLockSeconds: next as number })}
        />
      </div>
      <div class="row">
        <div class="row-text">
          <p class="row-label">Clear clipboard</p>
          <p class="row-desc">
            Remove secrets from the clipboard after copying. Clipboard managers may keep their own
            copy — this is a best effort, not a guarantee.
          </p>
        </div>
        <Segmented
          label="Clear clipboard"
          options={clipChoices}
          value={settings.clipboardClearSeconds}
          onchange={(next) => patch({ clipboardClearSeconds: next as number })}
        />
      </div>
      <div class="row">
        <div class="row-text">
          <p class="row-label">Log every reveal</p>
          <!-- D-31: off by default, and the copy must not imply the log is running when it is
               not. It also records copies, which is why this does not say "reveal" alone. -->
          <p class="row-desc">
            Records every time a secret is shown or copied, inside the encrypted vault. Off by
            default; the last 1000 entries are kept.
          </p>
        </div>
        <Toggle
          label="Log every reveal"
          checked={settings.auditLogEnabled}
          onchange={(next) => patch({ auditLogEnabled: next })}
        />
      </div>
    </div>

    <p class="group-label">System</p>
    <div class="card">
      <div class="row first">
        <div class="row-text">
          <p class="row-label">Start at login</p>
          <!-- It says "locked" because that is what happens, and a user who expects to find
               their vault open would otherwise read this as a security regression. -->
          <p class="row-desc">
            Open TrustVault when you sign in to this computer. It starts locked — you still unlock
            it yourself.
          </p>
        </div>
        <Toggle
          label="Start at login"
          checked={settings.launchAtLogin}
          onchange={(next) => patch({ launchAtLogin: next })}
        />
      </div>
    </div>

    <!-- Import lives in Settings because the prototype draws it nowhere (D-42 postdates the
         design) and this is the screen that already answers "what is true about my vault".
         It is a **row with a button**, matching the shape every other row on this screen has,
         rather than a card of its own competing with the vault card at the top.

         Export is not here and is not disabled-with-a-reason either: `trustvault-project.md`
         puts it out of scope for v1, so a greyed control would be a promise the product has
         decided not to make. A group named "Import" says what it is. -->
    <p class="group-label">Import</p>
    <div class="card">
      <div class="row first">
        <div class="row-text">
          <p class="row-label">Import from Bitwarden</p>
          <p class="row-desc">
            Reads an unencrypted JSON export. You see exactly what will be added — and what cannot
            be — before anything is written.
          </p>
        </div>
        <Button icon="note" onclick={onimport}>Import…</Button>
      </div>
    </div>

    <!-- The only place in Settings a save can fail. It sits under the card whose toggle can
         cause it, and it is announced: a toggle that snapped back with nothing said would read
         as the app being broken rather than as the OS refusing. §2 — icon and text, never
         colour alone. -->
    {#if error}
      <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
    {/if}

    <div class="danger">
      <div class="row-text">
        <p class="danger-label">Delete this vault</p>
        <p class="row-desc">
          {itemCount} items will be gone for good. You will have to type the vault name first.
        </p>
      </div>
      <Button variant="outlineDanger" onclick={ondeletevault}>Delete vault</Button>
    </div>
  </div>
</div>

<style>
  .settings {
    flex: 1;
    min-width: 0;
    background: var(--bg-surface);
    overflow-y: auto;
  }
  .column {
    max-width: 720px;
    padding: var(--space-6) var(--space-7) var(--space-7);
  }

  h1 {
    margin-bottom: 20px;
    font-size: var(--text-lg);
    line-height: var(--text-lg-lh);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-lg);
  }

  .group-label {
    margin-bottom: var(--space-3);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  /* §4: no shadow on a card. */
  .card {
    margin-bottom: 26px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    overflow: hidden;
  }

  .vault-head {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: var(--space-5);
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    flex: none;
    /* §4: --radius-full is for avatars only. */
    border-radius: var(--radius-full);
    background: var(--accent-wash);
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--accent);
  }
  .vault-titles {
    flex: 1;
    min-width: 0;
  }
  .vault-name {
    font-size: var(--text-md);
    font-weight: var(--weight-medium);
  }
  /* Left-to-right with a plain ellipsis. Truncating from the *left* would be more useful for a
     long path, but `direction: rtl` moves the leading slash to the end and prints a path that
     does not exist — the first thing anyone reads here is whether it is the file they think. */
  .vault-path {
    font-size: var(--text-sm);
    font-family: var(--font-mono);
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .facts {
    display: flex;
    border-top: 1px solid var(--border);
  }
  .fact {
    flex: 1;
    min-width: 0;
    padding: var(--space-4) var(--space-5);
    border-left: 1px solid var(--border);
  }
  .fact.first {
    border-left: none;
  }
  .fact-k {
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }
  .fact-v {
    margin-top: 3px;
    font-size: var(--text-base);
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    min-height: 44px;
    padding: var(--space-3) 14px;
    border-top: 1px solid var(--border);
  }
  .row.first {
    border-top: none;
  }
  .row-text {
    flex: 1;
    min-width: 0;
  }
  .row-label {
    font-size: var(--text-base);
  }
  .row-desc {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  /* Status is never colour alone — §2. The icon and the sentence carry it. */
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: -14px 0 26px;
    font-size: var(--text-sm);
    color: var(--danger);
  }

  .danger {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .danger-label {
    font-size: var(--text-base);
    color: var(--danger);
  }
</style>
