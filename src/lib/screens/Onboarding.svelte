<script lang="ts">
  /**
   * Vault creation, three steps — R-08. Step 3 shows the recovery kit once — R-07.
   *
   * The one rule that shapes this file: **the recovery code is rendered and dropped.** It is
   * held in a single `$state` that step 3 owns, there is no command to fetch it again, and
   * leaving the flow clears it. That is the most this side of the boundary can do about a
   * secret in a heap that cannot be wiped, and it is why `finish` clears before it routes.
   */
  import Button from '../components/Button.svelte';
  import TextField from '../components/TextField.svelte';
  import StrengthMeter from '../components/StrengthMeter.svelte';
  import Icon from '../icons/Icon.svelte';
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
  }

  const { ondone }: Props = $props();

  let step = $state<1 | 2 | 3>(1);
  let busy = $state(false);
  let error = $state('');

  let name = $state('Personal');
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
  const canCreate = $derived(passwordsMatch && !busy);

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

  function finish() {
    // Cleared before routing, not after: the next screen must not be able to observe it.
    recoveryCode = '';
    ondone();
  }

  /** Six groups of four, which is how R-07 says it is transcribed. */
  const groups = $derived(recoveryCode.split('-'));
</script>

<div class="onboarding">
  <header>
    <span class="mark"><Icon name="vault" size={18} /></span>
    <h1>{step === 3 ? 'Your recovery kit' : 'Create your vault'}</h1>
    <ol class="steps" aria-label="Progress">
      {#each [1, 2, 3] as index (index)}
        <li class:done={index < step} class:now={index === step} aria-current={index === step}>
          <span class="dot"></span>
          <span class="sr">Step {index}</span>
        </li>
      {/each}
    </ol>
  </header>

  {#if step === 1}
    <section>
      <p class="lede">
        The vault is a single file you own. Move it, back it up, or carry it on a stick — it is
        useless to anyone without your password.
      </p>

      <TextField
        label="Vault name"
        bind:value={name}
        autofocus
        onenter={() => nameValid && toStepTwo()}
      />

      <TextField
        label="File"
        bind:value={path}
        mono
        hint="Anywhere you like. The extension must be .tvault"
        error={path && !pathValid ? 'The file name must end in .tvault' : ''}
        oninput={() => (pathTouched = true)}
      />

      <Button variant="primary" wide disabled={!nameValid || !pathValid} onclick={toStepTwo}>
        Continue
      </Button>
    </section>
  {:else if step === 2}
    <section>
      <p class="lede">
        This password is the only thing between your vault and anyone holding the file. It is never
        stored and it cannot be recovered — that is the point.
      </p>

      <TextField
        label="Master password"
        type="password"
        bind:value={password}
        autofocus
        oninput={onPassword}
      />
      <StrengthMeter {strength} />

      <TextField
        label="Confirm"
        type="password"
        bind:value={confirmation}
        error={confirmation && !passwordsMatch ? 'The two do not match' : ''}
        onenter={create}
      />

      {#if busy && !kdf}
        <p class="note">
          <Icon name="clock" size={13} />
          Measuring this machine to pick key-derivation settings…
        </p>
      {/if}
      {#if error}
        <p class="note danger"><Icon name="alert" size={13} />{error}</p>
      {/if}

      <div class="row">
        <Button onclick={() => (step = 1)}>Back</Button>
        <Button variant="primary" disabled={!canCreate || !kdf} onclick={create}>
          Create vault
        </Button>
      </div>
    </section>
  {:else}
    <section>
      <p class="lede">
        This opens your vault if you forget your password. It is shown <strong>once</strong> and is not
        stored anywhere — write it down or print it now.
      </p>

      <div class="kit">
        {#each groups as group, index (index)}
          <span class="group">{group}</span>
        {/each}
      </div>

      <label class="ack">
        <input type="checkbox" bind:checked={kitAcknowledged} />
        <span>I have written this down and stored it somewhere safe.</span>
      </label>

      <div class="row">
        <Button onclick={() => window.print()}>Print</Button>
        <Button variant="primary" disabled={!kitAcknowledged} onclick={finish}>
          Open my vault
        </Button>
      </div>
    </section>
  {/if}
</div>

<style>
  .onboarding {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    width: 100%;
    max-width: 420px;
    margin: 0 auto;
    padding: var(--space-7) var(--space-6);
  }

  header {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--space-3);
  }
  .mark {
    display: flex;
    color: var(--accent);
  }
  h1 {
    font-size: var(--text-lg);
    line-height: var(--text-lg-lh);
    font-weight: var(--weight-semibold);
  }

  .steps {
    display: flex;
    gap: var(--space-2);
  }
  .steps li {
    display: flex;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--border-strong);
    transition: background var(--dur-instant) var(--ease-out);
  }
  .steps li.done .dot,
  .steps li.now .dot {
    background: var(--accent);
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .lede {
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
  }

  .row {
    display: flex;
    gap: var(--space-3);
    justify-content: flex-end;
  }

  .note {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--fg-subtle);
  }
  .note.danger {
    color: var(--danger);
  }

  /* The kit is the most-transcribed string in the product. MASTER.md §3's mono + tabular +
     disambiguated glyphs rule is a correctness requirement here, not a style choice. */
  .kit {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
    padding: var(--space-5);
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .group {
    font-family: var(--font-mono);
    font-size: var(--text-md);
    line-height: var(--text-md-lh);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    letter-spacing: var(--tracking-lg);
    text-align: center;
    color: var(--fg);
  }

  .ack {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
  }
  .ack input {
    margin-top: 2px;
    accent-color: var(--accent);
  }
  .ack input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  /* The kit must survive a print, which is half of what R-07 asks for. */
  @media print {
    .onboarding {
      max-width: none;
      color: #000;
    }
    .lede,
    .ack,
    .steps,
    .row {
      display: none;
    }
    .kit {
      border-color: #000;
    }
    .group {
      color: #000;
    }
  }
</style>
