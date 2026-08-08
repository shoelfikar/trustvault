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
    commitVault,
    createVault,
    defaultVaultPath,
    pickNewVaultPath,
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
    step === 1 ? nameValid && pathValid : step === 2 ? canCreate : kitAcknowledged && !busy,
  );

  /**
   * The path field's "Change" — a native save dialog, D-60.
   *
   * `pathTouched` is set on the way **in**, before the dialog is awaited, and not only on the
   * way out: opening the picker is the user taking the path over, and the name-watching
   * `$effect` above would otherwise overwrite whatever they chose the next time the name
   * changed. A cancelled dialog leaves the path alone, which is what a cancelled dialog means.
   */
  async function browse() {
    pathTouched = true;
    error = '';
    try {
      const chosen = await pickNewVaultPath(path.trim() || 'vault.tvault');
      if (chosen) path = chosen;
    } catch (thrown) {
      error = asIpcError(thrown).message;
    }
  }

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
      const failure = asIpcError(thrown);
      error = failure.message;
      // The refusal is about a field on the *previous* step, so the flow goes back to it —
      // D-62. An error naming a path, shown under a password field, is one the user reads
      // twice and then re-types their password for nothing. `pathTouched` stays set, so the
      // name no longer moves the path out from under them while they fix it.
      if (failure.kind === 'path_in_use') step = 1;
    } finally {
      busy = false;
    }
  }

  function next() {
    if (step === 1) void toStepTwo();
    else if (step === 2) void create();
    else void finish();
  }

  function back() {
    if (step === 1) oncancel?.();
    else if (step === 2) step = 1;
    // Step 3 still has no way back, and since D-69 the reason has changed: the vault does not
    // exist yet, so going back is now *possible* — but the kit on screen belongs to the vault
    // that would be discarded, and R-07 shows it once. A back button here would have to say it
    // throws the kit away, which is a decision rather than a hidden button.
  }

  /**
   * The acknowledgement is what writes the vault — D-69.
   *
   * Until this was split, `create_vault` wrote the file at the end of step 2, so closing the
   * window while reading the kit left a `.tvault` whose kit had never been recorded: shown
   * exactly once (R-07), no command to fetch it again, and the remembered path sending the next
   * launch to a lock screen with no recovery route out of it. Nothing is on disk until here.
   *
   * The code is cleared **after** the write rather than before, and only on success: a failed
   * commit leaves the user on step 3 with the kit still on screen, which is the only screen it
   * will ever be on. A failure here is a real possibility rather than a formality — the
   * directory can have gone away, or filled up, while the kit was being read.
   */
  async function finish() {
    if (busy) return;
    busy = true;
    error = '';
    try {
      await commitVault();
      recoveryCode = '';
      ondone();
    } catch (thrown) {
      error = asIpcError(thrown).message;
    } finally {
      busy = false;
    }
  }

  /** Six groups of four, which is how R-07 says it is transcribed. */
  const groups = $derived(recoveryCode.split('-'));

  /**
   * Step 3's two keyboard defects, both found by walking row 3 on 2026-08-08 — finding 4.
   *
   * **Focus.** Steps 1 and 2 land focus with `autofocus` on their text field; step 3 has no text
   * field, so it had nothing, and the password field being removed from the DOM dropped focus to
   * `document.body` — the first Tab then restarted from the top of the *document* rather than
   * from the kit on screen. The first control in the step gets focus instead, which is the same
   * place `autofocus` puts it on the two steps before. Not the acknowledgement checkbox, though
   * it is the required action: starting there puts Print and Save PDF *behind* the user, and the
   * kit is the one thing on this screen that cannot be shown again.
   *
   * **Enter.** `onenter` lives on the input inside `TextField`, so a step with no text field had
   * no Enter path at all — the row's "Enter finishes" was never implemented rather than broken.
   *
   * Once focus lands in the step, *most* of Enter is native and wants no handler: Enter on Print
   * prints, and Enter on the CTA finishes, which is the row's clause satisfied by a button being
   * a button. The one place it was still dead is the acknowledgement checkbox — a checkbox
   * toggles on **Space** and does nothing on Enter, in every browser, and that is the control the
   * whole step exists to collect. Reported twice from the walk, which is what it looks like when
   * a key does nothing: correct by the row's letter and wrong at the keyboard.
   *
   * So Enter toggles it, exactly as Space does. Row 3 is unchanged and is now more true rather
   * than less: Space still toggles, and Enter still finishes — from the CTA, which is where Tab
   * lands the moment the box is ticked and the button stops being disabled.
   */
  let kitStep = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (step !== 3) return;
    kitStep?.querySelector<HTMLElement>('button, input')?.focus();
  });

  function onKitKeydown(event: KeyboardEvent) {
    const target = event.target;
    if (event.key !== 'Enter') return;
    if (!(target instanceof HTMLInputElement) || target.type !== 'checkbox') return;
    // Not `finish()` even when the box is already ticked: the same key on the same control doing
    // two different things depending on state is worse than the dead key this replaces.
    event.preventDefault();
    kitAcknowledged = !kitAcknowledged;
  }
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
              <!-- Drawn-and-disabled from D-36 until 2026-08-07, with "a file picker would mean
                   adding a plugin" in its `title`. The plugin arrived with D-59 and that
                   sentence died with it; D-60 is the third door, a save dialog, because the
                   file being named does not exist yet. Typing the path still works — this is
                   the surface, not the mechanism. -->
              <button class="change" type="button" disabled={busy} onclick={() => void browse()}>
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
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div bind:this={kitStep} onkeydown={onKitKeydown}>
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
          </div>
        {/if}

        {#if error}
          <p class="note danger" role="alert"><Icon name="alert" size={13} />{error}</p>
        {/if}
      </div>

      <div class="cta">
        <!-- `create_vault` derives the key at the settings `calibrateKdf` just measured, so this
             is the slowest thing the application ever does and it is deliberately slow — R-02.
             `canCreate` already held the button disabled through it, which on its own is the
             worst signal available: a dead control is what a frozen window looks like. The label
             is the same pattern the other five long operations use (`Unlocking…`, `Saving…`,
             `Importing…`, `Deleting…`) rather than a spinner, because §5 bans the theatre and a
             present participle says which operation is running where a spinner does not. -->
        <Button variant="primary" tall disabled={!canAdvance} onclick={next}>
          {#if busy && step === 2}Creating vault…{:else if busy && step === 3}Saving vault…{:else}{copy[
              step
            ].cta}{/if}
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
  /* The numeral is `--fg` and not `--accent`, which is the one place in the app where brass on
     its own wash was doing the reading: 4.4:1 in both themes, and the last two findings
     `scripts/a11y.mjs` had left (D-63). Brass still marks the step — it is the border and the
     wash — and the digit is the part that has to be legible. Filling it like `li.done` would
     have been the other fix and is wrong: the current step and a finished step would then look
     the same, which is the whole thing the rail is for. */
  .steps li.on .num {
    border-color: var(--accent);
    background: var(--accent-wash);
    color: var(--fg);
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
  .change:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--fg);
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
