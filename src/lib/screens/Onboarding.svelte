<script lang="ts">
  /**
   * Vault creation, three steps — R-08. Step 3 shows the recovery kit once — R-07.
   *
   * The layout is the prototype's: a 300px rail carrying the three steps and their state, and a
   * centred 440px column carrying one step at a time.
   *
   * The one rule that shapes this file: **the recovery code is rendered and dropped.** It is
   * held in a single `$state` that step 3 owns, there is no command to fetch it again, and
   * leaving the flow clears it. That is the most this side of the boundary can do about a
   * secret in a heap that cannot be wiped, and it is why `finish` clears before it routes.
   */
  import Button from '../components/Button.svelte';
  import Callout from '../components/Callout.svelte';
  import TextField from '../components/TextField.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import Icon from '../icons/Icon.svelte';
  import Mark from '../icons/Mark.svelte';
  import {
    asIpcError,
    calibrateKdf,
    createVault,
    defaultVaultPath,
    scorePassword,
    type KdfSummary,
    type Strength,
  } from '../ipc';

  interface Props {
    /** Called once the kit has been shown and acknowledged. */
    ondone: () => void;
    /** Shown when the flow was entered from an open vault rather than at first launch. */
    oncancel?: () => void;
  }

  const { ondone, oncancel }: Props = $props();

  let step = $state<1 | 2 | 3>(1);
  let busy = $state(false);
  let error = $state('');

  let name = $state('Personal Vault');
  let path = $state('');
  /** Set once the user edits the path, so a later name change stops overwriting their choice. */
  let pathTouched = $state(false);

  let password = $state('');
  let confirmation = $state('');
  let strength = $state<Strength | null>(null);
  let kdf = $state<KdfSummary | null>(null);

  /** The one-time recovery code. Rendered on step 3, cleared when the flow ends. */
  let recoveryCode = $state('');
  let kitAcknowledged = $state(false);

  const rail = [
    { label: 'Vault name', desc: 'File & location' },
    { label: 'Master password', desc: 'Argon2id encryption' },
    { label: 'Recovery kit', desc: 'Offline backup' },
  ];

  const copy = {
    1: {
      title: 'Name your vault',
      body: 'A vault is a single encrypted file on this computer. You can create several later — one for personal, one for work.',
      cta: 'Continue',
      back: 'Cancel',
    },
    2: {
      title: 'Create a master password',
      body: 'One password unlocks everything. Use a long phrase that is easy for you to remember and impossible to guess.',
      cta: 'Create vault',
      back: 'Back',
    },
    3: {
      title: 'Save your recovery kit',
      body: 'Print this key or store it off this computer. Without your master password, it is the only way in.',
      cta: 'Finish & open vault',
      back: 'Back',
    },
  } as const;

  /** Keep the suggested path in step with the name until the user takes it over. */
  $effect(() => {
    if (pathTouched) return;
    const requested = name;
    void defaultVaultPath(requested).then((suggested) => {
      // The name may have changed while the round trip was in flight; only the answer to the
      // current question is allowed to win.
      if (!pathTouched && requested === name) path = suggested;
    });
  });

  /**
   * Score on a debounce — `docs/ipc-contract.md` §5. Not for safety, which debouncing does not
   * buy: the password is already in this heap. For the work, which is not free.
   */
  let scoreTimer: ReturnType<typeof setTimeout> | undefined;
  function onPassword(next: string) {
    clearTimeout(scoreTimer);
    if (!next) {
      strength = null;
      return;
    }
    scoreTimer = setTimeout(() => {
      // The vault name goes in as context, so a password built out of it scores as badly as it
      // deserves to.
      void scorePassword(next, [name, 'trustvault']).then((result) => {
        if (password === next) strength = result;
      });
    }, 180);
  }

  const nameValid = $derived(name.trim().length > 0);
  const pathValid = $derived(path.trim().endsWith('.tvault'));
  const passwordsMatch = $derived(password.length > 0 && password === confirmation);
  /**
   * Weak is allowed through with a warning rather than blocked. A creation flow that refuses
   * the password someone chose teaches them to add "1!" to it, which is not a stronger
   * password — it is the same password with a suffix the attacker also knows about.
   */
  const canCreate = $derived(passwordsMatch && !busy && Boolean(kdf));

  const canAdvance = $derived(
    step === 1 ? nameValid && pathValid : step === 2 ? canCreate : kitAcknowledged,
  );

  async function toStepTwo() {
    error = '';
    step = 2;
    if (kdf) return;
    busy = true;
    try {
      // Measured on this machine, not a constant (R-02). Held until create_vault writes it
      // into the header of the vault being made.
      kdf = await calibrateKdf();
    } catch (thrown) {
      error = asIpcError(thrown).message;
    } finally {
      busy = false;
    }
  }

  async function create() {
    if (!canCreate || !kdf) return;
    busy = true;
    error = '';
    try {
      const kit = await createVault(name.trim(), path.trim(), password, kdf);
      recoveryCode = kit.recoveryCode;
      // Dropped as soon as they have served their purpose. The vault is already written.
      password = '';
      confirmation = '';
      strength = null;
      step = 3;
    } catch (thrown) {
      error = asIpcError(thrown).message;
    } finally {
      busy = false;
    }
  }

  function next() {
    if (step === 1) void toStepTwo();
    else if (step === 2) void create();
    else finish();
  }

  function back() {
    if (step === 1) oncancel?.();
    else if (step === 2) step = 1;
    // Step 3 has no way back: the vault exists and the kit is on screen. The button is hidden.
  }

  function finish() {
    // Cleared before routing, not after: the next screen must not be able to observe it.
    recoveryCode = '';
    ondone();
  }

  /** Six groups of four, which is how R-07 says it is transcribed. */
  const groups = $derived(recoveryCode.split('-'));
</script>

<div class="onboarding">
  <aside class="rail">
    <span class="brand"><Mark size={30} /></span>
    <h1>Set up TrustVault</h1>
    <p class="lede">Three steps. Everything is stored locally and encrypted on this device.</p>

    <ol class="steps">
      {#each rail as entry, index (entry.label)}
        {@const n = index + 1}
        <li
          class:on={step === n}
          class:done={step > n}
          aria-current={step === n ? 'step' : undefined}
        >
          <span class="num">{n}</span>
          <span class="labels">
            <span class="label">{entry.label}</span>
            <span class="desc">{entry.desc}</span>
          </span>
        </li>
      {/each}
    </ol>

    <div class="spacer"></div>
    <p class="foot">TrustVault never sends your master password anywhere. No account, no server.</p>
  </aside>

  <div class="stage sb">
    <div class="column">
      <p class="eyebrow">Step {step} of 3</p>
      <h2>{copy[step].title}</h2>
      <p class="body">{copy[step].body}</p>

      <div class="content">
        {#if step === 1}
          <TextField
            label="Vault name"
            surface="raised"
            bind:value={name}
            autofocus
            onenter={() => canAdvance && next()}
          />
          <div class="gap"></div>
          <TextField
            label="File location"
            surface="raised"
            mono
            bind:value={path}
            error={path && !pathValid ? 'The file name must end in .tvault' : ''}
            hint={pathValid ? '' : 'Anywhere you like. The extension must be .tvault'}
            oninput={() => (pathTouched = true)}
            onenter={() => canAdvance && next()}
          >
            {#snippet trailing()}
              <!-- R-08 asks for "name & location", which a resolved default and an editable
                   path satisfies. A browse button means `tauri-plugin-dialog`, and a plugin
                   arrives when a requirement needs one and not before. -->
              <button
                class="change"
                type="button"
                disabled
                title="Type the path; a file picker
                would mean adding a plugin to a process that holds decrypted secrets"
              >
                Change
              </button>
            {/snippet}
          </TextField>
        {:else if step === 2}
          <TextField
            label="Master password"
            type="password"
            surface="raised"
            bind:value={password}
            autofocus
            oninput={onPassword}
          />
          <div class="meter"><StrengthMeter {strength} /></div>

          <div class="gap"></div>
          <TextField
            label="Confirm password"
            type="password"
            surface="raised"
            bind:value={confirmation}
            error={confirmation && !passwordsMatch ? 'The two do not match' : ''}
            onenter={() => canAdvance && next()}
          />

          {#if busy && !kdf}
            <p class="note">
              <Icon name="clock" size={13} />
              Measuring this machine to pick key-derivation settings…
            </p>
          {/if}

          <div class="callout">
            <Callout>
              Your master password cannot be recovered. If you forget it, the recovery kit in the
              next step is the only way back in.
            </Callout>
          </div>
        {:else}
          <div class="kit">
            <p class="kit-label">Recovery key</p>
            <div class="groups">
              {#each groups as group, index (index)}
                <span class="group">{group}</span>
              {/each}
            </div>
            <div class="kit-actions">
              <Button icon="copy" onclick={() => window.print()} title="Print or save as PDF">
                Print
              </Button>
              <Button icon="note" onclick={() => window.print()} title="Print or save as PDF">
                Save PDF
              </Button>
            </div>
          </div>

          <label class="ack">
            <input type="checkbox" bind:checked={kitAcknowledged} />
            <span>I have saved this kit somewhere safe</span>
          </label>
        {/if}

        {#if error}
          <p class="note danger"><Icon name="alert" size={13} />{error}</p>
        {/if}
      </div>

      <div class="cta">
        <Button variant="primary" tall disabled={!canAdvance} onclick={next}>
          {copy[step].cta}
        </Button>
        {#if step !== 3 && (step !== 1 || oncancel)}
          <Button tall onclick={back}>{copy[step].back}</Button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .onboarding {
    display: flex;
    height: 100vh;
    min-height: 0;
    background: var(--bg-surface);
  }

  /* ---- Rail ------------------------------------------------------------- */

  .rail {
    display: flex;
    flex-direction: column;
    width: 300px;
    flex: none;
    padding: var(--space-7) var(--space-6);
    background: var(--bg-base);
    /* §4: hairlines do the work. No shadow between the rail and the stage. */
    border-right: 1px solid var(--border);
  }
  .brand {
    display: flex;
    color: var(--accent);
  }
  .rail h1 {
    margin-top: var(--space-4);
    font-size: var(--text-md);
    line-height: var(--text-md-lh);
    font-weight: var(--weight-semibold);
  }
  .lede {
    margin-top: var(--space-1);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 26px;
  }
  .steps li {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    transition: background var(--dur-instant) var(--ease-out);
  }
  .steps li.on {
    background: var(--accent-wash);
  }

  .num {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    flex: none;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-full);
    font-family: var(--font-mono);
    font-size: var(--text-micro);
    font-weight: var(--weight-semibold);
    color: var(--fg-subtle);
  }
  .steps li.on .num {
    border-color: var(--accent);
    background: var(--accent-wash);
    color: var(--accent);
  }
  .steps li.done .num {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
  }

  .labels {
    display: flex;
    flex-direction: column;
  }
  .label {
    font-size: var(--text-base);
    color: var(--fg);
  }
  .steps li.on .label {
    color: var(--accent);
  }
  .desc {
    font-size: var(--text-sm);
    color: var(--fg-subtle);
  }

  .spacer {
    flex: 1;
  }
  .foot {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
    text-wrap: pretty;
  }

  /* ---- Stage ------------------------------------------------------------ */

  .stage {
    display: grid;
    place-items: center;
    flex: 1;
    min-width: 0;
    padding: var(--space-7);
    overflow-y: auto;
  }
  .column {
    width: 100%;
    max-width: 440px;
  }

  .eyebrow {
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--accent);
  }
  h2 {
    margin-top: var(--space-3);
    font-size: var(--text-lg);
    line-height: var(--text-lg-lh);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-lg);
  }
  .body {
    margin-top: 6px;
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .content {
    margin-top: var(--space-6);
  }
  .gap {
    height: var(--space-5);
  }
  .meter {
    margin-top: 10px;
  }

  .change {
    height: 24px;
    flex: none;
    padding: 0 var(--space-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }
  .change:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .callout {
    margin-top: 14px;
  }

  .note {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: 10px;
    font-size: var(--text-sm);
    color: var(--fg-subtle);
  }
  .note.danger {
    color: var(--danger);
  }

  .cta {
    display: flex;
    gap: var(--space-3);
    margin-top: 26px;
  }

  /* ---- Recovery kit ----------------------------------------------------- */

  .kit {
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
  }
  .kit-label {
    margin-bottom: 10px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }
  .groups {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
  }
  /* The kit is the most-transcribed string in the product. MASTER.md §3's mono + tabular +
     disambiguated glyphs rule is a correctness requirement here, not a style choice. */
  .group {
    padding: 6px var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    font-family: var(--font-mono);
    font-size: var(--text-base);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    letter-spacing: 0.06em;
    text-align: center;
    user-select: text;
  }
  .kit-actions {
    display: flex;
    gap: var(--space-3);
    margin-top: 14px;
  }
  .kit-actions :global(.btn) {
    flex: 1;
  }

  .ack {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 14px;
    font-size: var(--text-base);
    cursor: pointer;
  }
  .ack input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .ack input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* The kit must survive a print, which is half of what R-07 asks for. */
  @media print {
    .rail,
    .eyebrow,
    .body,
    .ack,
    .cta,
    .kit-actions {
      display: none;
    }
    .onboarding {
      display: block;
      height: auto;
      color: #000;
    }
    .kit {
      border-color: #000;
    }
    .group {
      background: none;
      color: #000;
    }
  }
</style>
