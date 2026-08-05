<script lang="ts">
  /**
   * The recovery flow — R-07. Two steps, drawn as the prototype draws them.
   *
   * **Step 2 is not the prototype's step 2, and that is deliberate.** The prototype sets a new
   * master password there. `unlock_recovery_kit` does not change the password: it unlocks with
   * the kit and *issues a fresh one*, because using a kit spends it. Setting a new password
   * would need a change-password command that does not exist, so step 2 shows the replacement
   * kit instead — the same two-step shape, carrying the thing that actually happened. Recorded
   * as D-35.
   *
   * The six blocks are six inputs rather than one field because that is how the kit is printed,
   * and a printed artefact typed into a box shaped like the print is a transcription with far
   * fewer places to lose your position.
   */
  import Button from '../components/Button.svelte';
  import Callout from '../components/Callout.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import { asIpcError, unlockRecoveryKit } from '../ipc';

  interface Props {
    path: string;
    displayName: string;
    onclose: () => void;
    onunlocked: () => void;
  }

  const { path, displayName, onclose, onunlocked }: Props = $props();

  let step = $state<1 | 2>(1);
  let blocks = $state(['', '', '', '', '', '']);
  let busy = $state(false);
  let error = $state('');
  /** The replacement kit, held only while step 2 is on screen. */
  let reissued = $state('');

  const filled = $derived(blocks.filter((block) => block.length === 4).length);
  const complete = $derived(filled === 6);
  const code = $derived(blocks.map((block) => block.toUpperCase()).join('-'));

  /** Advance on a full block, so six fields type like one. */
  function onBlockInput(index: number, event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const next = input.value
      .toUpperCase()
      .replace(/[^A-Z2-7]/g, '')
      .slice(0, 4);
    input.value = next;
    blocks[index] = next;
    if (next.length === 4 && index < 5) {
      const sibling = input.parentElement?.children[index + 1];
      (sibling as HTMLInputElement | undefined)?.focus();
    }
  }

  /** Backspace at the start of an empty block steps back, which is what typing expects. */
  function onBlockKey(index: number, event: KeyboardEvent) {
    const input = event.currentTarget as HTMLInputElement;
    if (event.key === 'Backspace' && input.value === '' && index > 0) {
      event.preventDefault();
      const sibling = input.parentElement?.children[index - 1];
      (sibling as HTMLInputElement | undefined)?.focus();
    }
    if (event.key === 'Enter' && complete) void submit();
  }

  /** A pasted whole kit fills every block rather than landing in the first one. */
  function onPaste(event: ClipboardEvent) {
    const text = event.clipboardData?.getData('text') ?? '';
    const groups = text.toUpperCase().replace(/[^A-Z2-7]/g, '');
    if (groups.length < 8) return;
    event.preventDefault();
    blocks = [0, 1, 2, 3, 4, 5].map((n) => groups.slice(n * 4, n * 4 + 4));
  }

  async function submit() {
    if (!complete || busy) return;
    busy = true;
    error = '';
    try {
      const kit = await unlockRecoveryKit(path, code);
      // The old kit is spent, so a new one was issued. It must be shown before moving on,
      // or the user leaves recovery with no way back in next time.
      reissued = kit.recoveryCode;
      blocks = ['', '', '', '', '', ''];
      step = 2;
    } catch (thrown) {
      error = asIpcError(thrown).message;
    } finally {
      busy = false;
    }
  }

  function done() {
    // Cleared before routing, not after: the next screen must not be able to observe it.
    reissued = '';
    onunlocked();
  }

  const groups = $derived(reissued.split('-'));
</script>

<Dialog
  title="Recover access to {displayName}"
  icon="note"
  meta="Step {step} of 2"
  width={420}
  closable={false}
  bare
  onclose={step === 1 ? onclose : done}
>
  {#if step === 1}
    <div class="pane">
      <h2>Enter your recovery key</h2>
      <p class="body">
        The 24-character key from the recovery kit you saved during setup — printed, in a PDF, or
        written down. It is the only way in without the master password.
      </p>

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="blocks" onpaste={onPaste}>
        {#each blocks as block, index (index)}
          <input
            value={block}
            maxlength="4"
            spellcheck="false"
            autocapitalize="characters"
            autocomplete="off"
            aria-label="Recovery key, group {index + 1} of 6"
            class:done={block.length === 4}
            oninput={(event) => onBlockInput(index, event)}
            onkeydown={(event) => onBlockKey(index, event)}
          />
        {/each}
      </div>

      <p class="hint" class:danger={Boolean(error)}>
        <Icon name={error ? 'alert' : complete ? 'check' : 'clock'} size={14} />
        <span>
          {#if error}{error}{:else if complete}Key complete — unlock to continue.{:else}
            {filled} of 6 groups entered.
          {/if}
        </span>
      </p>

      <div class="load">
        <!-- A file picker means `tauri-plugin-dialog`, and the manifest's standing rule is that
             a plugin arrives when a requirement needs one and not before — a plugin is widened
             attack surface in a process holding decrypted secrets. Typing the key works today. -->
        <Button icon="note" disabled title="Loading a kit file arrives with the file picker">
          Load recovery kit PDF…
        </Button>
      </div>
    </div>
  {:else}
    <div class="pane">
      <p class="accepted">
        <Icon name="shield-check" size={16} />
        <span>Recovery key accepted — vault decrypted</span>
      </p>

      <h2>Save your new recovery kit</h2>
      <p class="body">
        Using a kit spends it. This one replaces it — shown once, and not stored anywhere.
      </p>

      <div class="groups">
        {#each groups as group, index (index)}
          <span class="group">{group}</span>
        {/each}
      </div>

      <div class="callout">
        <Callout>
          The key you just typed no longer works. Print or save this one right away.
        </Callout>
      </div>
    </div>
  {/if}

  {#snippet footer()}
    {#if step === 1}
      <Button onclick={onclose}>Back</Button>
      <span class="grow"></span>
      <Button variant="primary" disabled={!complete || busy} onclick={submit}>
        {busy ? 'Unlocking…' : 'Unlock vault'}
      </Button>
    {:else}
      <Button icon="copy" onclick={() => window.print()}>Print</Button>
      <span class="grow"></span>
      <Button variant="primary" onclick={done}>Open vault</Button>
    {/if}
  {/snippet}
</Dialog>

<style>
  .pane {
    padding: 20px;
  }

  h2 {
    font-size: var(--text-md);
    line-height: var(--text-md-lh);
    font-weight: var(--weight-medium);
  }
  .body {
    margin-top: 6px;
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .blocks {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
    margin-top: 18px;
  }
  /* MASTER.md §3's transcription rule at its most load-bearing: this is the one string a user
     copies off paper under stress, and D-24 already removed 0/1/8/9 from the alphabet so the
     remaining confusions are the font's job. */
  .blocks input {
    height: 36px;
    padding: 0 var(--space-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--text-md);
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
    letter-spacing: 0.1em;
    text-align: center;
    text-transform: uppercase;
    outline: none;
    transition:
      border-color var(--dur-instant) var(--ease-out),
      box-shadow var(--dur-instant) var(--ease-out);
  }
  .blocks input.done {
    border-color: var(--fg-subtle);
  }
  .blocks input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-wash);
  }

  .hint {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-4);
    font-size: var(--text-sm);
    color: var(--fg-subtle);
    text-wrap: pretty;
  }
  /* Never colour alone — the glyph swaps with the state, not just the colour. */
  .hint.danger {
    color: var(--danger);
  }

  .load {
    margin-top: 14px;
  }

  .accepted {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: 14px;
    font-size: var(--text-base);
    color: var(--ok);
  }

  .groups {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
    margin-top: 18px;
  }
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

  .callout {
    margin-top: 14px;
  }

  .grow {
    flex: 1;
  }

  @media print {
    .callout,
    .accepted,
    .body {
      display: none;
    }
  }
</style>
