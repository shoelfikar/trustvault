<script lang="ts">
  /**
   * Importing a Bitwarden export — R-29, D-42, and the second surface in this project designed
   * without the prototype (D-48 was the first).
   *
   * The design predates D-42 and draws no import anywhere, so `MASTER.md` is the only authority
   * here, which is exactly the case `CLAUDE.md` names. What it settles: no new radius, no
   * shadow, hairline borders doing the separating, status as an icon **and** a word, and the
   * brass accent reserved for brand and interaction rather than pressed into meaning "good".
   *
   * **Three steps, and the middle one is the requirement.** R-29 is met only when every field in
   * the export is either mapped or named in a refusal, so the preview does not summarise the
   * refusals — it lists **all of them**. A truncated list would be the silent drop the
   * requirement exists to forbid, moved one indirection out where it looks like restraint. That
   * is also why `refusals` has no "and 12 more": if the list is long, the pane scrolls.
   *
   * **The report shown after the import is the commit's, not the preview's.** §6.8 has
   * `import_commit` re-read and re-parse the file rather than hold the preview's result — a
   * parsed foreign vault kept alive across a user reading a screen is plaintext sitting outside
   * everything that locks. The cost is a real race: a file edited between the two calls imports
   * as it is at commit. Leaving the preview on screen afterwards would report the losing side of
   * that race as fact, so `report` is replaced rather than kept.
   *
   * Nothing here reads the file. The path comes from `pickImportFile`, the host opens it, and
   * what crosses is counts, titles, labels and sentences — never a value (§6.8, asserted in the
   * core by `a_report_carries_no_value_from_the_vault_it_describes`).
   */
  import Button from '../components/Button.svelte';
  import Callout from '../components/Callout.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import {
    asIpcError,
    importCommit,
    importPreview,
    pickImportFile,
    type ImportReport,
  } from '../ipc';
  import { typeLabel, TYPE_GLYPHS } from './views';

  interface Props {
    onclose: () => void;
    /** An import landed: the item list and the tag sidebar are both stale. */
    onimported: () => void;
  }

  const { onclose, onimported }: Props = $props();

  let path = $state('');
  let report = $state<ImportReport | null>(null);
  /** Which report is on screen. The word on the heading changes with it, and so does the footer. */
  let stage = $state<'empty' | 'preview' | 'done'>('empty');
  let busy = $state(false);
  let error = $state('');

  const fileName = $derived(path.split(/[/\\]/).pop() || path);

  /**
   * Nothing found is not an error, and it is worth its own sentence.
   *
   * An export with no items parses perfectly and imports nothing; reported as a success with a
   * zero in it, that reads as the import having failed silently — which is the one thing R-29 is
   * about.
   */
  const empty = $derived(report !== null && report.total === 0);

  async function choose() {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const picked = await pickImportFile();
      // `null` is the user closing the dialog. Ordinary, and not a failure to announce.
      if (picked !== null) {
        path = picked;
        report = await importPreview(path);
        stage = 'preview';
      }
    } catch (thrown) {
      error = asIpcError(thrown).message;
      // The path is kept so the file name stays on screen next to the error — "that file is not
      // there" is only useful beside the name of the file.
      report = null;
      stage = 'empty';
    }
    busy = false;
  }

  async function commit() {
    if (busy || !path || empty) return;
    busy = true;
    error = '';
    try {
      report = await importCommit(path);
      stage = 'done';
      onimported();
    } catch (thrown) {
      error = asIpcError(thrown).message;
      // Left on `preview`: the vault is untouched — the import is one transaction — so the
      // honest state is the one before it, with the button still there to try again.
    }
    busy = false;
  }
</script>

<Dialog
  title="Import from Bitwarden"
  icon="note"
  meta={stage === 'done' ? 'Imported' : fileName}
  width={520}
  maxHeight={560}
  onclose={busy ? () => {} : onclose}
>
  {#if stage === 'empty'}
    <div class="intro">
      <p class="lede">
        TrustVault reads Bitwarden's <strong>unencrypted JSON export</strong>. Nothing is sent
        anywhere: the file is read once on this computer and no copy of it is kept.
      </p>
      <!-- The one warning worth a callout, and it is about the file rather than about us. An
           unencrypted export is a plaintext copy of a whole password manager sitting in a
           downloads folder, and it stays dangerous after the import succeeds. -->
      <Callout tone="warn">
        The export file is <strong>not encrypted</strong> — every password in it is readable by anything
        on this computer. Delete it once the import is done.
      </Callout>
    </div>
  {:else if report}
    <div class="report">
      <p class="lede">
        {#if stage === 'done'}
          <span class="lede-glyph"><Icon name="check" size={15} /></span>
          {report.total}
          {report.total === 1 ? 'item is' : 'items are'} now in this vault.
        {:else if empty}
          <span class="lede-glyph"><Icon name="alert" size={15} /></span>
          This export contains no items. Nothing would be imported.
        {:else}
          {report.total}
          {report.total === 1 ? 'item' : 'items'} will be added to this vault. Nothing is written until
          you choose Import.
        {/if}
      </p>

      {#if report.perKind.length > 0}
        <section>
          <h3>By type</h3>
          <ul class="kinds">
            {#each report.perKind as entry (entry.kind)}
              <li>
                <span class="glyph"><Icon name={TYPE_GLYPHS[entry.kind]} size={15} /></span>
                <span class="kind-name">{typeLabel(entry.kind)}</span>
                <span class="count">{entry.count}</span>
              </li>
            {/each}
          </ul>
          <!-- Said once, here, rather than as five refusals nobody can act on. Bitwarden has no
               API-key type and no Wi-Fi type, so an export cannot produce two of our seven —
               and reaching them would mean guessing from a title, which is the inference D-43
               exists to prevent. A user who expected those two needs to know it is the format
               and not their file. -->
          <p class="note">
            Bitwarden has no API Key or Wi-Fi type, so no export produces those two.
          </p>
        </section>
      {/if}

      {#if report.tagsCreated.length > 0 || report.tagsMerged.length > 0}
        <section>
          <h3>Tags</h3>
          <!-- Folders become tags. The **merge** is the surprising half and is therefore the
               one spelled out: a folder quietly joining a tag that already exists is a change
               to items the import never touched. -->
          {#if report.tagsCreated.length > 0}
            <p class="tag-line">
              <span class="tag-label">New</span>
              {report.tagsCreated.join(', ')}
            </p>
          {/if}
          {#if report.tagsMerged.length > 0}
            <p class="tag-line">
              <span class="tag-label">Merged into existing</span>
              {report.tagsMerged.join(', ')}
            </p>
          {/if}
        </section>
      {/if}

      {#if report.converted.length > 0}
        <section>
          <h3>
            <Icon name="refresh" size={14} />
            Changed on the way in — {report.converted.length}
          </h3>
          <ul class="lines">
            {#each report.converted as entry, index (index)}
              <li>
                <span class="where">{entry.itemTitle} · {entry.field}</span>
                <span class="why">{entry.note}</span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      <section>
        <h3>
          <Icon name={report.refusals.length > 0 ? 'alert' : 'check'} size={14} />
          Not imported — {report.refusals.length}
        </h3>
        {#if report.refusals.length === 0}
          <p class="note">Every field in the export has a home in this vault.</p>
        {:else}
          <!-- Every one of them, never a count with a "…and 14 more". R-29 is the rule that
               nothing is dropped in silence, and a list that hides its tail is that rule
               broken one level further out where it looks like tidiness. -->
          <ul class="lines">
            {#each report.refusals as entry, index (index)}
              <li>
                <span class="where">{entry.itemTitle} · {entry.field}</span>
                <span class="why">{entry.reason}</span>
              </li>
            {/each}
          </ul>
          <p class="note">
            These fields have nowhere to go in TrustVault. They are listed rather than dropped so
            you can copy them over by hand.
          </p>
        {/if}
      </section>
    </div>
  {/if}

  {#if error}
    <!-- §2: an icon and a sentence, never colour alone. -->
    <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
  {/if}

  {#snippet footer()}
    {#if stage === 'done'}
      <Button variant="primary" onclick={onclose}>Done</Button>
    {:else}
      <Button disabled={busy} onclick={onclose}>Cancel</Button>
      <Button icon="note" disabled={busy} onclick={() => void choose()}>
        {stage === 'empty' ? 'Choose file…' : 'Choose another…'}
      </Button>
      <Button
        variant="primary"
        icon="plus"
        disabled={busy || stage !== 'preview' || empty}
        title={empty ? 'This export contains no items' : undefined}
        onclick={() => void commit()}
      >
        {#if busy && stage === 'preview'}
          Importing…
        {:else if report && !empty}
          Import {report.total}
          {report.total === 1 ? 'item' : 'items'}
        {:else}
          Import
        {/if}
      </Button>
    {/if}
  {/snippet}
</Dialog>

<style>
  .intro,
  .report {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  /* Deliberately **not** a flex container. It was one until the screenshot harness photographed
     it: flex makes every child a flex item, so the `<strong>` inside the sentence became a
     column of its own and the paragraph read as three fragments side by side. The status glyph
     is inline instead, which is what a glyph inside a sentence is. */
  .lede {
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }
  .lede strong {
    font-weight: var(--weight-medium);
    color: var(--fg);
  }
  .lede-glyph {
    display: inline-flex;
    margin-right: 4px;
    vertical-align: -2px;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  h3 {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  /* §4: hairline borders, no shadow, no radius above 6px. */
  .kinds {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--border);
    overflow: hidden;
  }
  .kinds li {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 9px 11px;
    background: var(--bg-raised);
  }
  .glyph {
    display: flex;
    color: var(--fg-subtle);
  }
  .kind-name {
    flex: 1;
    min-width: 0;
    font-size: var(--text-base);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* Counts are numbers to compare down a column — tabular, like every other figure in the app. */
  .count {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--text-base);
  }

  .tag-line {
    font-size: var(--text-base);
    line-height: var(--text-base-lh);
    text-wrap: pretty;
  }
  .tag-label {
    margin-right: 7px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
  }

  .lines {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .lines li {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 11px;
    border-top: 1px solid var(--border);
  }
  .lines li:first-child {
    border-top: none;
  }
  .where {
    font-size: var(--text-base);
  }
  .why {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .note {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 14px;
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
