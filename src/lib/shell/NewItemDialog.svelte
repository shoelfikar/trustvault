<script lang="ts">
  /**
   * The New-item dialog — every field the prototype draws, per type.
   *
   * **Nothing here can be saved yet.** `add_item` is a Phase 3 command by the roadmap's own
   * split, and Phase 2 deliberately shipped no mutation commands. The dialog is drawn in full
   * because the shape of what an item *is* is a design decision that belongs with the rest of
   * the design, and disabled at the one place it would lie — the Save button, which carries the
   * reason.
   *
   * The typed password is scored through `score_password` in the host, the same path onboarding
   * uses: zxcvbn's dictionaries are a few hundred kilobytes and S-01 budgets 800ms for cold
   * start, so they never enter the bundle.
   */
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import IconButton from '../components/IconButton.svelte';
  import Segmented from '../components/Segmented.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import Toggle from '../components/Toggle.svelte';
  import { ALL_SETS, generatePassword } from '../passwords';
  import { scorePassword, type ItemKind, type Strength } from '../ipc';
  import { TYPE_GLYPHS, tagColour, typeLabel } from './views';

  interface Props {
    vaultName: string;
    vaultFile: string;
    /** Tags already in the vault, offered as chips. */
    tags: string[];
    onclose: () => void;
  }

  const { vaultName, vaultFile, tags, onclose }: Props = $props();

  const KINDS: ItemKind[] = ['login', 'api_key', 'card', 'note', 'wifi', 'ssh_key', 'identity'];

  let kind = $state<ItemKind>('login');
  let title = $state('');
  let password = $state('');
  let revealed = $state(false);
  let totpOn = $state(false);
  let totpSecret = $state('');
  let environment = $state('Production');
  let chosen = $state<string[]>([]);
  let strength = $state<Strength | null>(null);

  let scoreTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const current = password;
    clearTimeout(scoreTimer);
    if (!current) {
      strength = null;
      return;
    }
    scoreTimer = setTimeout(() => {
      void scorePassword(current, [title, vaultName]).then((result) => {
        if (password === current) strength = result;
      });
    }, 180);
  });

  const placeholder = $derived(
    {
      login: 'Tokopedia',
      api_key: 'Anthropic API',
      card: 'BCA Visa Platinum',
      note: '2FA recovery codes',
      wifi: 'Home Wi-Fi',
      ssh_key: 'Production server',
      identity: 'Passport',
    }[kind],
  );

  const toggleTag = (tag: string) =>
    (chosen = chosen.includes(tag) ? chosen.filter((one) => one !== tag) : [...chosen, tag]);

  const glyph = (candidate: ItemKind): IconName => TYPE_GLYPHS[candidate];
</script>

<Dialog title="New item" icon="plus" meta={vaultName} width={560} maxHeight={720} {onclose}>
  <p class="section">Item type</p>
  <div class="chips">
    {#each KINDS as candidate (candidate)}
      <button
        type="button"
        class="chip"
        class:on={kind === candidate}
        aria-pressed={kind === candidate}
        onclick={() => (kind = candidate)}
      >
        <Icon name={glyph(candidate)} size={14} />
        {typeLabel(candidate)}
      </button>
    {/each}
  </div>

  <div class="form">
    <div class="control">
      <label for="new-title">Title</label>
      <!-- svelte-ignore a11y_autofocus -- the dialog exists to be filled in from the top. -->
      <input id="new-title" bind:value={title} {placeholder} autofocus />
    </div>

    {#if kind === 'login'}
      <div class="control">
        <label for="new-user">Username or email</label>
        <input id="new-user" placeholder="budi.santoso@gmail.com" />
      </div>

      <div class="control">
        <label for="new-pass">Password</label>
        <div class="composite">
          <input
            id="new-pass"
            class="mono"
            type={revealed ? 'text' : 'password'}
            bind:value={password}
          />
          <IconButton
            icon={revealed ? 'eye-off' : 'eye'}
            label={revealed ? 'Hide password' : 'Reveal password'}
            onclick={() => (revealed = !revealed)}
          />
          <Button
            variant="ghost"
            icon="refresh"
            title="Generate a new password on this device"
            onclick={() => {
              password = generatePassword(20, ALL_SETS);
              revealed = true;
            }}
          >
            Generate
          </Button>
        </div>
        <div class="meter"><StrengthMeter {strength} width="150px" /></div>
      </div>

      <div class="control">
        <label for="new-site">Website</label>
        <input id="new-site" class="mono" placeholder="tokopedia.com" />
      </div>

      <div class="switch">
        <Toggle label="Save 2FA secret" checked={totpOn} onchange={(next) => (totpOn = next)} />
        <div class="switch-text">
          <p class="switch-label">Save 2FA secret (TOTP)</p>
          <p class="hint">
            When you turn on 2FA, the site shows a QR code and a secret key. Save it here —
            TrustVault generates the codes locally.
          </p>
        </div>
      </div>

      {#if totpOn}
        <div class="nested">
          <div class="control">
            <label for="new-totp">Secret key</label>
            <div class="composite">
              <input
                id="new-totp"
                class="mono"
                bind:value={totpSecret}
                placeholder="JBSW Y3DP EHPK 3PXP or otpauth://…"
                spellcheck="false"
              />
            </div>
          </div>
          <p class="hint">
            Paste the secret first — the live 6-digit preview arrives with TOTP support.
          </p>
        </div>
      {/if}
    {:else if kind === 'api_key'}
      <div class="control">
        <label for="new-token">API key / token</label>
        <textarea
          id="new-token"
          class="mono"
          rows="3"
          spellcheck="false"
          placeholder="sk-ant-api03-… or ghp_… — paste the token here"></textarea>
        <p class="hint">Stored encrypted. Only the last 4 characters show in the list.</p>
      </div>
      <div class="split">
        <div class="control grow">
          <span class="label-as-text">Environment</span>
          <Segmented
            label="Environment"
            options={[
              { value: 'Production', label: 'Production' },
              { value: 'Staging', label: 'Staging' },
              { value: 'Local', label: 'Local' },
            ]}
            value={environment}
            onchange={(next) => (environment = next as string)}
          />
        </div>
        <div class="control narrow">
          <label for="new-exp">Expiry (optional)</label>
          <input id="new-exp" class="mono" placeholder="31 Dec 2026" />
        </div>
      </div>
      <div class="control">
        <label for="new-docs">Docs / console URL</label>
        <input id="new-docs" class="mono" placeholder="console.anthropic.com" />
      </div>
    {:else if kind === 'card'}
      <div class="control">
        <label for="new-holder">Name on card</label>
        <input id="new-holder" placeholder="BUDI SANTOSO" />
      </div>
      <div class="control">
        <label for="new-number">Card number</label>
        <input id="new-number" class="mono" placeholder="4811 7742 9930 4417" />
      </div>
      <div class="split">
        <div class="control grow">
          <label for="new-card-exp">Expiry</label>
          <input id="new-card-exp" class="mono" placeholder="08 / 29" />
        </div>
        <div class="control narrow">
          <label for="new-cvv">CVV</label>
          <input id="new-cvv" class="mono" type="password" placeholder="•••" />
        </div>
      </div>
    {:else if kind === 'note'}
      <div class="control">
        <label for="new-note">Note</label>
        <textarea id="new-note" rows="5" placeholder="This note is encrypted just like a password."
        ></textarea>
      </div>
    {:else if kind === 'wifi'}
      <div class="control">
        <label for="new-ssid">Network name (SSID)</label>
        <input id="new-ssid" placeholder="Santoso-5G" />
      </div>
      <div class="control">
        <label for="new-wifi-pass">Password</label>
        <input id="new-wifi-pass" class="mono" type="password" />
      </div>
      <div class="control">
        <label for="new-sec">Security</label>
        <input id="new-sec" placeholder="WPA3-Personal" />
      </div>
    {:else if kind === 'ssh_key'}
      <div class="control">
        <label for="new-host">Host</label>
        <input id="new-host" class="mono" placeholder="10.4.1.22" />
      </div>
      <div class="control">
        <label for="new-ssh-user">User</label>
        <input id="new-ssh-user" class="mono" placeholder="deploy" />
      </div>
      <div class="control">
        <label for="new-passphrase">Passphrase</label>
        <input id="new-passphrase" class="mono" type="password" />
      </div>
    {:else}
      <div class="control">
        <label for="new-full">Full name</label>
        <input id="new-full" placeholder="Budi Santoso" />
      </div>
      <div class="control">
        <label for="new-email">Email</label>
        <input id="new-email" class="mono" placeholder="budi@example.com" />
      </div>
      <div class="control">
        <label for="new-phone">Phone</label>
        <input id="new-phone" class="mono" placeholder="+62 812 8891 4402" />
      </div>
    {/if}

    {#if tags.length}
      <div class="control">
        <span class="label-as-text">Tags</span>
        <div class="chips">
          {#each tags as tag (tag)}
            <button
              type="button"
              class="chip tag"
              class:on={chosen.includes(tag)}
              aria-pressed={chosen.includes(tag)}
              onclick={() => toggleTag(tag)}
            >
              <span class="dot" style="background:{tagColour(tag)}"></span>{tag}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <span class="stored">Stored encrypted in {vaultFile}</span>
    <Button onclick={onclose}>Cancel</Button>
    <Button variant="primary" disabled title="Saving arrives with the mutation commands">
      Save item
    </Button>
  {/snippet}
</Dialog>

<style>
  .section {
    margin-bottom: var(--space-3);
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: flex;
    align-items: center;
    gap: 7px;
    height: var(--control-h);
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    color: var(--fg-muted);
    white-space: nowrap;
    transition:
      background var(--dur-instant) var(--ease-out),
      border-color var(--dur-instant) var(--ease-out),
      color var(--dur-instant) var(--ease-out);
  }
  .chip:hover:not(.on) {
    color: var(--fg);
  }
  /* Brass marks the engaged chip — interaction, not a verdict. §2. */
  .chip.on {
    border-color: var(--accent);
    background: var(--accent-wash);
    color: var(--accent);
  }
  .chip:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .chip.tag {
    height: 24px;
    padding: 0 9px;
    gap: 6px;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 2px;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-top: 18px;
  }

  .control {
    display: flex;
    flex-direction: column;
  }
  .control label,
  .label-as-text {
    margin-bottom: 6px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  input,
  textarea {
    width: 100%;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    outline: none;
    transition:
      border-color var(--dur-instant) var(--ease-out),
      box-shadow var(--dur-instant) var(--ease-out);
  }
  input {
    height: 32px;
  }
  textarea {
    padding: var(--space-3) 10px;
    line-height: 18px;
    resize: none;
  }
  input:focus,
  textarea:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }
  input::placeholder,
  textarea::placeholder {
    color: var(--fg-subtle);
  }

  /* MASTER.md §3 — anything the user may transcribe renders in the mono face. */
  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
  }

  .composite {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 32px;
    padding: 0 var(--space-2) 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    transition:
      border-color var(--dur-instant) var(--ease-out),
      box-shadow var(--dur-instant) var(--ease-out);
  }
  .composite:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }
  .composite input {
    height: 100%;
    padding: 0;
    border: none;
    background: none;
  }
  .composite input:focus {
    box-shadow: none;
  }

  .meter {
    margin-top: var(--space-3);
  }

  .split {
    display: flex;
    gap: var(--space-4);
  }
  .grow {
    flex: 1;
  }
  .narrow {
    width: 150px;
  }

  .switch {
    display: flex;
    align-items: flex-start;
    gap: 9px;
  }
  .switch-text {
    flex: 1;
    min-width: 0;
  }
  .switch-label {
    font-size: var(--text-base);
  }

  .hint {
    margin-top: 6px;
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
    text-wrap: pretty;
  }
  .switch-text .hint {
    margin-top: 0;
  }

  .nested {
    padding: var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
  }

  .stored {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--fg-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
