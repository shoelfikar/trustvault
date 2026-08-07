#!/usr/bin/env node
/**
 * Renders the app's screens to PNG without a Tauri host.
 *
 * The gap this closes has been open since Phase 0: every visual claim in this repo was a claim
 * about the code, because the app needs a Tauri host to render anything and there was no way
 * to look at it. There is one — the frontend cannot tell a real host from a fake, because
 * `src/lib/ipc.ts` is the only file that calls `invoke`, and CI keeps it that way.
 *
 * So: serve `dist/`, inject a stub `window.__TAURI_INTERNALS__` answering with fixture data,
 * and let headless Firefox screenshot it.
 *
 *   npm run build && node scripts/screenshots.mjs [--scenario lock] [--theme dark]
 *
 * Three things about this machine are baked in rather than discovered again:
 *
 * * **Firefox here is a snap** and cannot write outside `$HOME`. Output goes to
 *   `docs/screenshots/`, inside the repo, which is.
 * * **Firefox screenshots at the `load` event.** A scenario that has to click something would
 *   be captured before the click, so the page holds its own load event open with a slow image
 *   (`/__hold__`) served by this script, and the driving happens inside that window.
 * * **Motion is disabled** in the injected CSS. Otherwise a dialog is caught mid-fade and two
 *   runs of the same scenario differ, which is the difference between a screenshot and a
 *   regression test.
 */

import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { readFile, mkdir, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)));
const DIST = join(ROOT, 'dist');
const OUT = join(ROOT, 'docs/screenshots');
const PROFILE = join(ROOT, 'target/screenshot-profile');

/** How long the page holds its load event open, for scenarios that drive the UI. */
const HOLD_MS = 900;
/** Window size. 1280×860 is the prototype's own canvas plus the titlebar. */
const SIZE = { width: 1280, height: 860 };

/* ---- Fixture data ---------------------------------------------------------
 *
 * Deliberately not a copy of the prototype's demo data: these values exercise the cases the
 * design has to survive — a long title, a vault name that overflows the sidebar, an item with
 * no tags, and every status the list can render.
 */

const ITEMS = [
  item('github', 'login', 'GitHub', ['work', 'dev'], 'strong', true),
  item('aws', 'api_key', 'AWS Production Access Key', ['work', 'infra'], 'weak', false),
  item('bca', 'login', 'Bank Central Asia', ['finance'], 'breached', false),
  item('wifi', 'wifi', 'Kantor 5GHz', [], 'unknown', false),
  item('ssh', 'ssh_key', 'deploy@production', ['infra'], 'reused', false),
  item('note', 'note', 'Recovery phrases', ['personal'], 'unknown', false),
  item('card', 'card', 'Visa •••• 4417', ['finance'], 'expired', false),
];

function item(id, kind, title, tags, status, favourite) {
  return {
    id: `00000000-0000-4000-8000-${id.padEnd(12, '0').slice(0, 12)}`,
    kind,
    title,
    tags,
    status,
    favourite,
    created_at: 1735689600000,
    updated_at: 1785600000000,
  };
}

const FIELDS = [
  { id: 'f1', label: 'Username', kind: 'username', secret: false, value: 'octocat', mask: null },
  {
    id: 'f2',
    label: 'Password',
    kind: 'password',
    secret: true,
    value: null,
    mask: '••••••••••••',
  },
  { id: 'f3', label: 'URL', kind: 'url', secret: false, value: 'https://github.com', mask: null },
  {
    id: 'f4',
    label: 'Recovery code',
    kind: 'text',
    secret: true,
    value: null,
    mask: '••••••••••••',
  },
  // The seed the detail pane's one-time code row is drawn from. It is `kind: 'otp'` and
  // elided like any other secret — the pane reads the kind to know a code exists, never the
  // value, which is the whole shape §6.6 asks for.
  {
    id: 'f5',
    label: '2FA secret',
    kind: 'otp',
    secret: true,
    value: null,
    mask: '••••••••••••',
  },
];

const SETTINGS = {
  theme: 'system',
  auto_lock_seconds: 300,
  clipboard_clear_seconds: 12,
  audit_log_enabled: false,
  sidebar_width: 232,
  list_width: 300,
  last_vault_path: '/home/shoel/Documents/personal.tvault',
};

/* ---- Scenarios ------------------------------------------------------------ */

const UNLOCKED = {
  state: 'unlocked',
  path: '/home/shoel/Documents/personal.tvault',
  display_name: 'Personal Vault',
  item_count: ITEMS.length,
};

const SCENARIOS = {
  onboarding: { status: { state: 'no_vault', path: null, display_name: '', item_count: null } },
  lock: {
    status: {
      state: 'locked',
      path: '/home/shoel/Documents/personal.tvault',
      display_name: 'personal',
      item_count: null,
    },
  },
  shell: { status: UNLOCKED },
  settings: { status: UNLOCKED, drive: `click('[title="Settings"]')` },
  watchtower: { status: UNLOCKED, drive: `clickText('button', 'Watchtower')` },
  palette: { status: UNLOCKED, drive: `key('k', { ctrlKey: true })` },
  // The palette with a query that matches nothing. Its own empty state (R-19) is otherwise
  // unreachable in a shot, and it is the one place in the app where "no results" and "the
  // search broke" look the same if the pane is left blank.
  paletteEmpty: {
    status: UNLOCKED,
    drive: `key('k', { ctrlKey: true }); await sleep(120);
            fill('.search input', 'zzzz'); await sleep(400)`,
  },
  generator: { status: UNLOCKED, drive: `key('g', { ctrlKey: true })` },
  newItem: { status: UNLOCKED, drive: `key('n', { ctrlKey: true })` },
  // The Add dialog with its 2FA seed filled in, so the live preview strip is in the baseline
  // rather than only in the code. It is a separate scenario and not a change to `newItem`,
  // because the empty dialog is the shot that shows what the form looks like before anyone
  // touches it — and that is the one most likely to regress unnoticed.
  newItemTotp: {
    status: UNLOCKED,
    drive: `key('n', { ctrlKey: true }); await sleep(120);
            click('[role="switch"][aria-label="Save 2FA secret"]'); await sleep(120);
            fill('#new-totp', 'GEZDGNBVGY3TQOJQ'); await sleep(600)`,
  },
  editItem: { status: UNLOCKED, drive: `clickText('button', 'Edit')` },
  deleteItem: { status: UNLOCKED, drive: `click('[title="Delete item"]')` },
  // The two empty states R-19 is actually about, and the only two a screenshot can reach: the
  // vault with nothing in it, and Trash — which is empty by construction and stays that way
  // (D-49), so it is the one whose copy nobody would otherwise ever look at again.
  emptyVault: {
    status: { ...UNLOCKED, display_name: 'Fresh Vault', item_count: 0 },
    items: [],
  },
  trash: { status: UNLOCKED, drive: `clickText('button', 'Trash')` },
  // Both steps of the import (D-59), and they are two scenarios rather than one because they
  // are two different arguments. The first is what the user reads *before* choosing a file —
  // the warning that the export is plaintext and stays plaintext — and the second is the
  // report, whose refusal list is the whole of R-29's acceptance criterion rendered.
  //
  // This surface is the one that most needs the harness. It cannot be reached without a native
  // file dialog, which means it cannot be reached in a browser at all: the picker is stubbed
  // here, so these are the only two shots of it that will ever exist outside a real desktop.
  importIntro: {
    status: UNLOCKED,
    drive: `click('[title="Settings"]'); await sleep(150);
            clickText('button', 'Import…'); await sleep(300)`,
  },
  importPreview: {
    status: UNLOCKED,
    hold: 1600,
    drive: `click('[title="Settings"]'); await sleep(150);
            clickText('button', 'Import…'); await sleep(300);
            clickText('button', 'Choose file…'); await sleep(500)`,
  },
  // The state after the import, which is a *different* screen and not a toast: the report on it
  // is the commit's, and its refusal list is the only record of what did not come across. It is
  // shot because it is seen once per user and is therefore the copy most likely to rot.
  importDone: {
    status: UNLOCKED,
    hold: 2200,
    drive: `click('[title="Settings"]'); await sleep(150);
            clickText('button', 'Import…'); await sleep(300);
            clickText('button', 'Choose file…'); await sleep(500);
            clickText('button', 'Import 34'); await sleep(400)`,
  },
};

/**
 * What `import_preview` and `import_commit` answer with — D-59.
 *
 * Written to exercise the parts of the pane that are easy to get wrong and impossible to see by
 * accident: a **merged** tag beside a created one, a conversion, and more than one refusal. A
 * fixture with an empty refusal list would photograph the happy path and leave the list that
 * carries R-29 untested by the only tool that looks at it.
 *
 * No field *values* anywhere in it, which is not a fixture style choice — §6.8 forbids the real
 * report from carrying one, and a fixture that quoted values would put a shape on screen that
 * the product cannot produce.
 */
const IMPORT_REPORT = {
  total: 34,
  per_kind: [
    { kind: 'login', count: 27 },
    { kind: 'card', count: 3 },
    { kind: 'note', count: 3 },
    { kind: 'identity', count: 1 },
  ],
  tags_created: ['Banking', 'Shopping'],
  tags_merged: ['Work'],
  converted: [{ item_title: 'Fastmail', field: 'Requires 2FA', note: 'stored as the text "true"' }],
  refusals: [
    { item_title: 'Dropbox', field: 'recovery-codes.txt', reason: 'attachments are not imported' },
    { item_title: 'Chase', field: 'Linked account', reason: 'a card has nowhere to hold it' },
  ],
};

/* ---- The injected page ---------------------------------------------------- */

/**
 * The stub host, as a string injected before the app's module script.
 *
 * It answers `invoke` from the fixtures above. Three commands are answered with intent rather
 * than data: `reveal_field` returns a real-looking password because the whole point of the
 * detail pane is how a revealed secret is set; `copy_field` returns only a timestamp, because
 * that absence is the product's central claim and a stub that returned a value would make a
 * screenshot of a leak look fine; and `generate_password` returns a **fixed** string, because
 * the real command returns a different one every time and a shot that never matches itself
 * cannot become a baseline.
 */
function harness(scenario, theme) {
  // A scenario may replace the item fixture wholesale — that is how an empty vault is shot,
  // and it is a substitution rather than a flag because "the list is empty" is a different
  // fixture, not a different rendering of the same one.
  const { status, drive, items = ITEMS } = SCENARIOS[scenario];
  return `
<script>
window.__SHOT__ = ${JSON.stringify({ scenario, theme })};
const STATUS = ${JSON.stringify(status)};
const ITEMS = ${JSON.stringify(items)};
const FIELDS = ${JSON.stringify(FIELDS)};
const SETTINGS = ${JSON.stringify(SETTINGS)};
const IMPORT_REPORT = ${JSON.stringify(IMPORT_REPORT)};

let listener = 0;
function respond(cmd, args) {
  switch (cmd) {
    case 'vault_status': return STATUS;
    case 'build_info': return { version: '0.0.0', format_version: 1, extension: 'tvault' };
    case 'get_settings': return { ...SETTINGS, theme: ${JSON.stringify(theme)} };
    case 'set_settings': return { ...SETTINGS, ...(args.settings ?? {}) };
    case 'list_items': return ITEMS;
    // The palette asks the host to rank; the ranking itself is Rust (D-46) and is tested
    // there. What a screenshot needs is rows, so the stub matches on the title alone — a
    // deliberately dumber rule than the real one, because a second fuzzy implementation
    // living here is the "two generators, one of them real" mistake D-44 named.
    case 'search_items':
      return ITEMS.filter((item) =>
        item.title.toLowerCase().includes(String(args.query ?? '').trim().toLowerCase()),
      ).slice(0, args.limit ?? 6);
    case 'get_item': return { ...ITEMS[0], fields: FIELDS };
    // Guarded rather than assumed: the empty-vault scenario has no ITEMS[0], and an unguarded
    // read there would be a TypeError inside the stub host rather than a blank screenshot.
    case 'reveal_field': return { value: 'tR7-vault-2026!qz', remask_at: Date.now() + 10000 };
    case 'copy_field': return { clears_at: Date.now() + 12000 };
    case 'score_password': return { score: 3, label: 'Strong', crack_time: 'centuries' };
    // Fixed rather than random, because a screenshot that differs on every run cannot be
    // compared against a baseline — and this is the value the generator dialog renders.
    case 'generate_password':
      return { password: 'k4Vq-7pXm-2Rtz-9Bhw', score: 4, label: 'Excellent', crack_time: 'centuries' };
    case 'copy_generated': return { clears_at: Date.now() + 12000 };
    // Fixed, for the reason generate_password is: the real code changes every 30 seconds and
    // a shot that never matches itself cannot become a baseline. The 22 seconds left is the
    // prototype's own figure, so the ring is drawn at the fraction the design shows.
    case 'totp_code': case 'totp_preview':
      return { code: '418209', expires_at: Date.now() + 22000, period: 30, digits: 6 };
    case 'calibrate_kdf': return { m_cost: 262144, t_cost: 3, p_cost: 1 };
    case 'default_vault_path': return '/home/shoel/Documents/personal.tvault';
    case 'lock': case 'unlock': return null;
    // The mutation commands are answered so a screenshot run cannot be the thing that writes
    // to a vault, and so an accidental call is a no-op rather than an unhandled warning. The
    // list they return to is static, which is why nothing here changes ITEMS.
    // (No backticks in this block: it lives inside the template literal that builds the page.)
    case 'add_item': return { item_id: ITEMS[0]?.id ?? 'new' };
    case 'update_item': case 'delete_item': return null;
    case 'create_vault': case 'unlock_recovery_kit':
      return { recovery_code: 'K7QX-2MRE-9WVT-4HDP-6SNA-3JFB' };
    // The picker (D-59). A fixed path, because the name is on screen in the dialog header and a
    // shot whose header changes per machine cannot be a baseline. This is also the one stub in
    // the harness that stands in for a **native** dialog rather than for a command: there is no
    // file chooser in a headless browser, so without this the surface is unphotographable.
    case 'pick_import_file': return '/home/shoel/Downloads/bitwarden_export.json';
    case 'pick_vault_file': return null;
    // Both halves answer with the same report, which is what the real pair does for a file
    // nobody edited in between -- and the difference between them (commit re-reads, so it can
    // differ) is a race no screenshot can hold still anyway.
    case 'import_preview': case 'import_commit': return IMPORT_REPORT;
    default:
      // Tauri's event plugin rides the same channel. Anything else is a command the harness
      // has not been taught, and it is loud rather than silently undefined.
      if (cmd.startsWith('plugin:event|')) return ++listener;
      console.warn('[shot] unhandled command', cmd);
      return null;
  }
}

window.__TAURI_INTERNALS__ = {
  invoke: (cmd, args) => Promise.resolve(respond(cmd, args ?? {})),
  transformCallback: (callback) => {
    const id = ++listener;
    window['_' + id] = callback;
    return id;
  },
};

// The theme is set here as well as through get_settings, because the lock screen paints
// before the settings round-trip resolves -- which is the reason D-33 put the theme in a
// config file in the first place.
document.documentElement.dataset.theme = ${JSON.stringify(theme)};
<\/script>
<style>
  /* A screenshot of a transition is a screenshot that differs between runs. */
  *, *::before, *::after { animation: none !important; transition: none !important; }
  /* The caret blinks, and it blinks in exactly one of two frames. */
  * { caret-color: transparent !important; }
</style>
<script type="module">
  const sleep = (ms) => new Promise((done) => setTimeout(done, ms));
  const click = (selector) => document.querySelector(selector)?.click();
  const clickText = (selector, text) =>
    [...document.querySelectorAll(selector)]
      .find((el) => el.textContent.trim().includes(text))?.click();
  const key = (k, init = {}) =>
    window.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, ...init }));
  // Typing, for the surfaces that only appear once a field has something in it. The value is
  // set and an input event dispatched, which is what Svelte's bind:value listens for --
  // assigning .value alone updates the DOM and tells the component nothing.
  // (No backticks in this block: it lives inside the template literal that builds the page.)
  const fill = (selector, value) => {
    const el = document.querySelector(selector);
    if (!el) return;
    el.value = value;
    el.dispatchEvent(new Event('input', { bubbles: true }));
  };

  // Let Svelte mount and the first commands resolve, then drive.
  await sleep(250);
  ${drive ? `${drive};` : ''}
<\/script>
`;
}

/* ---- Server --------------------------------------------------------------- */

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
  '.png': 'image/png',
};

async function serve() {
  const template = await readFile(join(DIST, 'index.html'), 'utf8');

  const server = createServer(async (request, response) => {
    const url = new URL(request.url, 'http://localhost');

    // The load-event hold. Firefox waits for images, so this is what buys `drive` its time.
    //
    // Per-scenario since 2026-08-06, because the import flow is the first drive with **three**
    // clicks in it and the default hold expired mid-sequence — the shot came out as the step
    // before, which is the worst way for this to fail: a screenshot of the wrong state still
    // looks like a screenshot. A scenario declares its own `hold` rather than the default
    // rising for all thirty-odd, most of which need none of it.
    if (url.pathname === '/__hold__') {
      const hold = SCENARIOS[url.searchParams.get('scenario')]?.hold ?? HOLD_MS;
      setTimeout(() => {
        response.writeHead(200, { 'content-type': 'image/svg+xml' });
        response.end('<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>');
      }, hold);
      return;
    }

    if (url.pathname === '/' || url.pathname === '/index.html') {
      const scenario = url.searchParams.get('scenario') ?? 'shell';
      const theme = url.searchParams.get('theme') ?? 'light';
      const page = template
        .replace('<head>', `<head>${harness(scenario, theme)}`)
        .replace(
          '</body>',
          `<img src="/__hold__?scenario=${encodeURIComponent(scenario)}" alt="" width="1" height="1"></body>`,
        );
      response.writeHead(200, { 'content-type': TYPES['.html'] });
      response.end(page);
      return;
    }

    try {
      const file = join(DIST, url.pathname);
      if (!file.startsWith(DIST)) throw new Error('outside dist');
      const body = await readFile(file);
      response.writeHead(200, {
        'content-type': TYPES[extname(file)] ?? 'application/octet-stream',
      });
      response.end(body);
    } catch {
      response.writeHead(404).end('not found');
    }
  });

  await new Promise((ready) => server.listen(0, '127.0.0.1', ready));
  return { server, port: server.address().port };
}

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
