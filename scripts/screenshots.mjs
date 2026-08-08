#!/usr/bin/env node
/**
 * Renders the app's screens to PNG without a Tauri host.
 *
 *   npm run build && node scripts/screenshots.mjs [--scenario lock] [--theme dark]
 *
 * The fixtures, the stub host and the server all live in `scripts/harness.mjs` — extracted
 * there on 2026-08-07 when `a11y.mjs` needed the same fake application to walk. What is left
 * here is the part that is only about pictures: spawning Firefox, and where the files go.
 */

import { spawn } from 'node:child_process';
import { mkdir, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { DIST, ROOT, SCENARIOS, SIZE, serve } from './harness.mjs';

/** Inside the repo, because Firefox here is a snap and cannot write outside `$HOME`. */
const OUT = join(ROOT, 'docs/screenshots');
const PROFILE = join(ROOT, 'target/screenshot-profile');

/* ---- Firefox -------------------------------------------------------------- */

function shoot(url, output) {
  return new Promise((done, fail) => {
    const firefox = spawn(
      'firefox',
      [
        '--headless',
        '--profile',
        PROFILE,
        '--window-size',
        `${SIZE.width},${SIZE.height}`,
        '--screenshot',
        output,
        url,
      ],
      { stdio: ['ignore', 'ignore', 'pipe'] },
    );
    let stderr = '';
    firefox.stderr.on('data', (chunk) => (stderr += chunk));
    firefox.on('exit', (code) =>
      // Firefox exits non-zero on plenty of things it also screenshots fine through, so the
      // file's existence is the test, not the exit code.
      existsSync(output) ? done() : fail(new Error(`firefox failed (${code}): ${stderr}`)),
    );
  });
}

/* ---- Main ----------------------------------------------------------------- */

const args = process.argv.slice(2);
const flag = (name) => {
  const at = args.indexOf(`--${name}`);
  return at === -1 ? null : args[at + 1];
};

if (!existsSync(join(DIST, 'index.html'))) {
  console.error('dist/index.html is missing — run `npm run build` first.');
  process.exit(1);
}

const scenarios = flag('scenario') ? [flag('scenario')] : Object.keys(SCENARIOS);
const themes = flag('theme') ? [flag('theme')] : ['light', 'dark'];

for (const scenario of scenarios) {
  if (!SCENARIOS[scenario]) {
    console.error(`unknown scenario "${scenario}" — one of ${Object.keys(SCENARIOS).join(', ')}`);
    process.exit(1);
  }
}

await mkdir(OUT, { recursive: true });
await rm(PROFILE, { recursive: true, force: true });
await mkdir(PROFILE, { recursive: true });

const { server, port } = await serve();
let failures = 0;

for (const scenario of scenarios) {
  for (const theme of themes) {
    const output = join(OUT, `${scenario}-${theme}.png`);
    const url = `http://127.0.0.1:${port}/?scenario=${scenario}&theme=${theme}`;
    try {
      await rm(output, { force: true });
      await shoot(url, output);
      console.log(`  ${scenario} (${theme}) -> docs/screenshots/${scenario}-${theme}.png`);
    } catch (error) {
      failures += 1;
      console.error(`  ${scenario} (${theme}) FAILED: ${error.message}`);
    }
  }
}

server.close();
process.exit(failures === 0 ? 0 : 1);
