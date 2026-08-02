<script lang="ts">
  import { onMount } from 'svelte';
  import { contrastRatio, grade, resolveToken, type ContrastLevel } from '../a11y/contrast';

  /**
   * Token specimen. Phase 0 gate evidence, and a permanent regression surface:
   * if a token changes and breaks contrast, this page says so instead of a reviewer
   * having to notice.
   */

  interface Props {
    theme: 'dark' | 'light';
    uiScale: 'compact' | 'default' | 'large';
  }
  let { theme, uiScale }: Props = $props();

  const surfaces = [
    ['--bg-base', 'Base / chrome', 'Titlebar, sidebar, window edge'],
    ['--bg-surface', 'Surface', 'Main content pane'],
    ['--bg-raised', 'Raised', 'Cards, dialogs, detail pane'],
    ['--bg-hover', 'Hover', 'Row hover'],
    ['--bg-selected', 'Selected', 'Selected list row'],
  ] as const;

  const inks = [
    ['--fg', 'Text primary', false],
    ['--fg-muted', 'Text secondary', false],
    ['--fg-subtle', 'Text disabled', false],
    ['--accent', 'Accent (brass)', false],
    ['--border', 'Border', true],
    ['--border-strong', 'Border strong', true],
  ] as const;

  const status = [
    ['--ok', 'Strong / healthy'],
    ['--warn', 'Reused / weak'],
    ['--danger', 'Breached / expired'],
    ['--info', 'Neutral note'],
  ] as const;

  const typeScale = [
    ['--text-micro', 'micro', 'SECTION LABEL', 500],
    ['--text-sm', 'sm', 'Metadata, timestamps, helper text', 400],
    ['--text-base', 'base', 'List rows, field values, body', 400],
    ['--text-md', 'md', 'Item titles, dialog body', 500],
    ['--text-lg', 'lg', 'All Items · Watchtower', 600],
    ['--text-xl', 'xl', 'Vault locked', 600],
  ] as const;

  const spaces = [
    '--space-1',
    '--space-2',
    '--space-3',
    '--space-4',
    '--space-5',
    '--space-6',
    '--space-7',
  ];
  const radii = ['--radius-sm', '--radius-md', '--radius-full'];
  const shadows = ['--shadow-popover', '--shadow-dialog', '--shadow-window'];

  type Row = { token: string; label: string; ratio: number; level: ContrastLevel; ui: boolean };
  let ratios = $state<Row[]>([]);
  let failures = $derived(ratios.filter((r) => r.level === 'fail' || r.level === 'AA-large'));

  // Recompute whenever the theme changes — that is the whole point of measuring rather
  // than hardcoding, since the light theme darkens the accent to keep its ratio.
  function measure() {
    const bg = resolveToken('--bg-surface');
    ratios = inks.map(([token, label, ui]) => {
      const ratio = contrastRatio(resolveToken(token), bg);
      return { token, label, ratio, level: grade(ratio, ui), ui };
    });
  }

  onMount(measure);
  $effect(() => {
    theme;
    uiScale;
    queueMicrotask(measure);
  });
</script>

<div class="specimen sb">
  <header>
    <h1>Design tokens</h1>
    <p>
      Rendered from <code>src/lib/styles/tokens.css</code>. Authority is
      <code>design-system/password-manager/MASTER.md</code>. Contrast is computed live against
      <code>--bg-surface</code>, not copied from the spec.
    </p>
    {#if failures.length === 0}
      <p class="verdict ok">All measured pairs meet their WCAG threshold in the {theme} theme.</p>
    {:else}
      <p class="verdict bad">
        {failures.length} pair{failures.length === 1 ? '' : 's'} below threshold in the {theme} theme:
        {failures.map((f) => f.token).join(', ')}
      </p>
    {/if}
  </header>

  <section>
    <h2>Surfaces</h2>
    <div class="swatches">
      {#each surfaces as [token, label, use] (token)}
        <div class="swatch">
          <div class="chip" style:background="var({token})"></div>
          <div class="meta">
            <span class="name">{label}</span>
            <code>{token}</code>
            <span class="use">{use}</span>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section>
    <h2>Ink &amp; contrast</h2>
    <table>
      <thead>
        <tr><th>Token</th><th>Role</th><th>Sample</th><th>Ratio</th><th>Grade</th></tr>
      </thead>
      <tbody>
        {#each ratios as r (r.token)}
          <tr>
            <td><code>{r.token}</code></td>
            <td>{r.label}</td>
            <td style:color="var({r.token})">
              {r.ui ? '───────' : 'Handoff 0O1lI'}
            </td>
            <td class="mono num">{Number.isNaN(r.ratio) ? '—' : r.ratio.toFixed(2)}:1</td>
            <td
              ><span class="grade {r.level}">{r.level}</span>{#if r.ui}<span class="hint"
                  >3:1 UI</span
                >{/if}</td
            >
          </tr>
        {/each}
      </tbody>
    </table>
  </section>

  <section>
    <h2>Status</h2>
    <p class="note">
      Never conveyed by colour alone — every status carries an icon and a text label (<code
        >MASTER.md</code
      > §2).
    </p>
    <div class="pills">
      {#each status as [token, label] (token)}
        <span class="pill" style:color="var({token})" style:border-color="var({token})">
          <span class="dot" style:background="var({token})"></span>{label}
        </span>
      {/each}
    </div>
  </section>

  <section>
    <h2>Type scale</h2>
    <p class="note">
      13px body, not the web's 16px. UI scale is <strong>{uiScale}</strong> — every size below is
      multiplied by <code>--ui-scale</code>.
    </p>
    {#each typeScale as [token, name, sample, weight] (token)}
      <div class="typerow">
        <code class="tname">{token}</code>
        <span
          style:font-size="var({token})"
          style:line-height="var({token}-lh)"
          style:font-weight={weight}
          style:letter-spacing={name === 'micro'
            ? 'var(--tracking-micro)'
            : name === 'lg'
              ? 'var(--tracking-lg)'
              : name === 'xl'
                ? 'var(--tracking-xl)'
                : 'normal'}
          style:text-transform={name === 'micro' ? 'uppercase' : 'none'}>{sample}</span
        >
      </div>
    {/each}
    <div class="typerow">
      <code class="tname">--font-mono</code>
      <span class="mono">K9vTz-2mQr7-Wx4L · 481920 · 0O 1lI · 4811 7742 9930 4417</span>
    </div>
  </section>

  <section>
    <h2>Space</h2>
    <div class="spaces">
      {#each spaces as token (token)}
        <div class="sprow">
          <code>{token}</code>
          <span class="bar" style:width="var({token})"></span>
        </div>
      {/each}
    </div>
  </section>

  <section>
    <h2>Radius &amp; elevation</h2>
    <p class="note">
      Nothing at 12–16px. Shadows are for true overlays only — never rows or cards.
    </p>
    <div class="boxes">
      {#each radii as token (token)}
        <div class="box" style:border-radius="var({token})"><code>{token}</code></div>
      {/each}
      {#each shadows as token (token)}
        <div class="box raised" style:box-shadow="var({token})"><code>{token}</code></div>
      {/each}
    </div>
  </section>
</div>

<style>
  .specimen {
    padding: var(--space-7);
    max-width: 900px;
    margin: 0 auto;
    overflow-y: auto;
    height: 100%;
  }
  header {
    margin-bottom: var(--space-7);
  }
  h1 {
    font-size: var(--text-lg);
    line-height: var(--text-lg-lh);
    letter-spacing: var(--tracking-lg);
    font-weight: var(--weight-semibold);
    margin: 0 0 var(--space-2);
  }
  h2 {
    font-size: var(--text-micro);
    line-height: var(--text-micro-lh);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    font-weight: var(--weight-medium);
    color: var(--fg-subtle);
    margin: 0 0 var(--space-4);
  }
  section {
    margin-bottom: var(--space-7);
  }
  p {
    margin: 0 0 var(--space-3);
    color: var(--fg-muted);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    text-wrap: pretty;
  }
  code {
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }
  .verdict {
    font-size: var(--text-sm);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid;
    display: inline-block;
  }
  .verdict.ok {
    color: var(--ok);
    border-color: var(--ok);
  }
  .verdict.bad {
    color: var(--danger);
    border-color: var(--danger);
  }

  .swatches {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--space-3);
  }
  .swatch {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }
  .chip {
    width: 40px;
    height: 40px;
    flex: none;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-strong);
  }
  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .name {
    font-size: var(--text-base);
  }
  .use {
    font-size: var(--text-micro);
    color: var(--fg-subtle);
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-base);
  }
  th {
    text-align: left;
    font-size: var(--text-micro);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
    color: var(--fg-subtle);
    font-weight: var(--weight-medium);
    padding: 0 var(--space-3) var(--space-2) 0;
  }
  td {
    padding: var(--space-2) var(--space-3) var(--space-2) 0;
    border-top: 1px solid var(--border);
  }
  .num {
    font-variant-numeric: tabular-nums;
  }
  .grade {
    font-size: var(--text-micro);
    padding: 0 var(--space-2);
    border-radius: var(--radius-sm);
    border: 1px solid currentColor;
  }
  .grade.AAA,
  .grade.AA {
    color: var(--ok);
  }
  .grade.AA-large {
    color: var(--warn);
  }
  .grade.fail {
    color: var(--danger);
  }
  .hint {
    font-size: var(--text-micro);
    color: var(--fg-subtle);
    margin-left: var(--space-2);
  }

  .pills {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    height: var(--target-min);
    padding: 0 var(--space-3);
    border: 1px solid;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-full);
  }

  .typerow {
    display: flex;
    align-items: baseline;
    gap: var(--space-5);
    padding: var(--space-2) 0;
    border-top: 1px solid var(--border);
  }
  .tname {
    width: 120px;
    flex: none;
  }

  .spaces {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .sprow {
    display: flex;
    align-items: center;
    gap: var(--space-5);
  }
  .sprow code {
    width: 120px;
  }
  .bar {
    height: 12px;
    background: var(--accent);
    border-radius: 2px;
  }

  .boxes {
    display: flex;
    gap: var(--space-5);
    flex-wrap: wrap;
  }
  .box {
    width: 150px;
    height: 72px;
    display: grid;
    place-items: center;
    border: 1px solid var(--border);
    background: var(--bg-raised);
  }
  .box.raised {
    border: 0;
    border-radius: var(--radius-md);
  }
  .note {
    margin-bottom: var(--space-4);
  }
</style>
