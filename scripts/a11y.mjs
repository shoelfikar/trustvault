#!/usr/bin/env node
/**
 * Walks every surface in both themes and reports what a keyboard and a pair of eyes would hit.
 *
 *   npm run build && node scripts/a11y.mjs [--audit focus] [--scenario shell] [--theme dark]
 *                                          [--stops]
 *
 * Three audits, all from `MASTER.md` §10's checklist and all previously unmeasurable:
 *
 * * **focus** — every interactive element changes appearance when it takes keyboard focus (§7).
 * * **contrast** — every text node clears 4.5:1 against the background actually behind it (§9).
 * * **taborder** — every operable control is in the tab order, once (`docs/keyboard-audit.md`).
 *
 * `--stops` prints the tab order itself, in document order, for the surfaces selected. It is how
 * finding 7 was found and it fails nothing: how many stops a surface *should* have is a row in
 * `docs/keyboard-audit.md`, not a property of the DOM.
 *
 * It runs against `scripts/harness.mjs`, the same fake host and the same fixtures the screenshot
 * tool photographs, so a finding here is about the surface in the shot and not about a second
 * application assembled for testing.
 *
 * **What it cannot see** is worth as much as what it can, and it is the same blind spot the
 * screenshot harness has: it stubs `invoke`, so nothing about the real host is exercised. Nor
 * can any of the three press Tab — `taborder` reads what the browser *would* treat as a stop,
 * which answers "is this reachable at all" but not "does Tab escape this dialog". Focus traps
 * and the order a person was actually reading in stay rows in `docs/keyboard-audit.md`, walked
 * by hand with the pointer unplugged, which is what S-08 asks for.
 */

import { spawn } from 'node:child_process';
import { readFile, mkdir, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { DIST, ROOT, SCENARIOS, SIZE, serve } from './harness.mjs';

const PROFILE = join(ROOT, 'target/a11y-profile');
const AUDITS = ['focus', 'contrast', 'taborder'];
/** Long enough for a cold Firefox plus the longest scenario's own hold, and no longer. */
const TIMEOUT_MS = 15000;

/**
 * Loads the audit sources as text.
 *
 * They live in `scripts/audits/*.js` rather than as strings in this file for one reason: a
 * hundred lines of JavaScript inside a template literal is a hundred lines nothing checks, and
 * `node --check` reads a file.
 */
async function loadAudits(names) {
  const entries = await Promise.all(
    names.map(async (name) => [
      name,
      await readFile(join(ROOT, 'scripts/audits', `${name}.js`), 'utf8'),
    ]),
  );
  return Object.fromEntries(entries);
}

/**
 * Opens `url` in headless Firefox and resolves when its audit has posted, or after `TIMEOUT_MS`.
 *
 * **It waits for the report and not for the process**, which is the one thing about driving
 * Firefox from a script that has to be got right here. With no `--screenshot` argument the
 * browser has no reason to exit at all, and a `firefox` that finds an existing instance exits
 * *immediately* after handing the URL over — so both "wait for exit" and "wait a fixed time
 * then kill" produce a run that reports nothing and calls it clean. A dedicated `--profile` and
 * `--new-instance` keep it a process of its own; the report is what says the page got there.
 */
function run(url, arrived) {
  return new Promise((done) => {
    const firefox = spawn(
      'firefox',
      [
        '--headless',
        '--new-instance',
        '--profile',
        PROFILE,
        '--window-size',
        `${SIZE.width},${SIZE.height}`,
        url,
      ],
      { stdio: 'ignore' },
    );
    const finish = () => {
      clearTimeout(stop);
      firefox.kill('SIGTERM');
      done();
    };
    const stop = setTimeout(finish, TIMEOUT_MS);
    void arrived.then(finish);
  });
}

/* ---- Main ----------------------------------------------------------------- */

const args = process.argv.slice(2);
const flag = (name) => {
  const at = args.indexOf(`--${name}`);
  return at === -1 ? null : args[at + 1];
};
const has = (name) => args.includes(`--${name}`);

if (!existsSync(join(DIST, 'index.html'))) {
  console.error('dist/index.html is missing — run `npm run build` first.');
  process.exit(1);
}

const audits = flag('audit') ? [flag('audit')] : AUDITS;
const scenarios = flag('scenario') ? [flag('scenario')] : Object.keys(SCENARIOS);
const themes = flag('theme') ? [flag('theme')] : ['light', 'dark'];

for (const audit of audits) {
  if (!AUDITS.includes(audit)) {
    console.error(`unknown audit "${audit}" — one of ${AUDITS.join(', ')}`);
    process.exit(1);
  }
}

await rm(PROFILE, { recursive: true, force: true });
await mkdir(PROFILE, { recursive: true });

const reports = [];
let announce = () => {};
const { server, port } = await serve({
  onReport: (report) => {
    reports.push(report);
    announce();
  },
  audits: await loadAudits(AUDITS),
});

for (const audit of audits) {
  for (const scenario of scenarios) {
    for (const theme of themes) {
      const arrived = new Promise((resolve) => (announce = resolve));
      const url = `http://127.0.0.1:${port}/?scenario=${scenario}&theme=${theme}&audit=${audit}`;
      await run(url, arrived);
      if (reports.at(-1)?.scenario !== scenario || reports.at(-1)?.audit !== audit) {
        // Silence is a result. A run whose page never posted is reported as its own failure
        // rather than left out of the tally, because a missing surface and a clean one look
        // identical in a summary that only counts findings.
        reports.push({ audit, scenario, theme, total: 0, findings: [] });
      }
    }
  }
}

server.close();

/* ---- The report ----------------------------------------------------------- */

let problems = 0;
let silent = 0;

for (const report of reports) {
  const where = `${report.scenario} (${report.theme})`;
  if (report.error) {
    console.error(`  ${where}: the audit itself failed — ${report.error}`);
    problems += 1;
    continue;
  }

  // A surface that reported nothing to look at is not a clean surface: it is a drive that did
  // not reach its screen, and it would otherwise be indistinguishable from a perfect one.
  if (report.total === 0) {
    const nothing = { focus: 'focusable element', contrast: 'text', taborder: 'tab stop' };
    console.error(
      `  ${report.audit} ${where}: nothing to measure — the scenario drew no ${nothing[report.audit]}`,
    );
    silent += 1;
    continue;
  }

  const failures = report.findings.filter(
    (finding) => finding.verdict === 'no-indicator' || finding.verdict === 'fail',
  );
  const unfocusable = report.findings.filter((finding) => finding.verdict === 'unfocusable');

  // Exemptions are printed on the clean line rather than hidden by it. A surface that passes
  // because six things were excused is a different fact from one that passes because six things
  // are readable, and the difference has to survive the summary.
  const excused = report.exempt ? `, ${report.exempt} exempt` : '';
  // The tab-order audit counts stops rather than checks, and the difference matters in the
  // output: "13 checked" reads as a coverage number, "13 tab stops" is the thing itself — the
  // number a person compares against the surface in front of them.
  const counted = report.audit === 'taborder' ? 'tab stops' : 'checked';
  // A modal narrows what the audit measured, so it is printed on every line rather than left to
  // be inferred from a count that got smaller.
  const within = report.modal ? ` within ${report.modal}` : '';

  // Printed on a clean surface as well as a failing one: the sequence is evidence, not a
  // diagnostic, and the defect it exists to expose (a surface that is thirteen consecutive
  // stops) fails no rule here.
  const sequence = () => {
    if (!has('stops') || !report.stops) return;
    report.stops.forEach((stop, index) =>
      console.log(`      ${String(index).padStart(2)}  ${stop}`),
    );
  };

  if (failures.length === 0 && unfocusable.length === 0) {
    console.log(`  ${report.audit} ${where}: ${report.total} ${counted}${within}${excused}, clean`);
    sequence();
    continue;
  }

  problems += failures.length + unfocusable.length;
  console.log(`  ${report.audit} ${where}: ${report.total} ${counted}${within}`);

  if (report.audit === 'taborder') {
    for (const finding of failures) {
      console.log(`      ${finding.kind}: ${finding.name} — ${finding.detail}`);
      for (const member of finding.members ?? []) console.log(`          ${member}`);
    }
    sequence();
    continue;
  }

  if (report.audit === 'focus') {
    for (const finding of [...failures, ...unfocusable]) {
      console.log(`      ${finding.verdict}: ${finding.name}`);
    }
    continue;
  }

  // Contrast findings are grouped by the *pair* rather than listed one text node at a time.
  // A token used in forty places fails in forty places, and forty lines saying the same thing
  // buries the second colour that failed in three. The pair is the thing that gets fixed.
  const pairs = new Map();
  for (const finding of failures) {
    const key = `${finding.colour} on ${finding.on}`;
    const seen = pairs.get(key) ?? { ...finding, count: 0, example: finding.text };
    seen.count += 1;
    pairs.set(key, seen);
  }
  for (const pair of pairs.values()) {
    console.log(
      `      ${pair.ratio}:1 (needs ${pair.required})  ${pair.colour} on ${pair.on}  ×${pair.count}  e.g. "${pair.example}"`,
    );
  }
}

console.log(
  problems === 0 && silent === 0
    ? `\n${reports.length} surface-audits, no findings.`
    : `\n${reports.length} surface-audits, ${problems} findings, ${silent} surfaces measured nothing.`,
);
process.exit(problems === 0 && silent === 0 ? 0 : 1);
