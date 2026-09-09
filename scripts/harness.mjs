/**
 * The fake Tauri host every browser-driven tool in this repo runs against.
 *
 * Extracted from `screenshots.mjs` on 2026-08-07, unchanged, because a second tool needed it:
 * `a11y.mjs` walks the same surfaces looking for focus rings and contrast rather than
 * photographing them. Two copies of the stub would have been the "two generators, one of them
 * the real one" mistake D-44 named — the fixtures decide what every screen renders, and a shot
 * taken against one fixture and an audit run against another are not evidence about the same
 * application.
 *
 * The gap it closes has been open since Phase 0: the app needs a Tauri host to render anything,
 * so every visual claim in this repo was a claim about the code. There is one way in — the
 * frontend cannot tell a real host from a fake, because `src/lib/ipc.ts` is the only file that
 * calls `invoke`, and CI keeps it that way.
 *
 * So: serve `dist/`, inject a stub `window.__TAURI_INTERNALS__` answering with fixture data,
 * and let headless Firefox drive it.
 *
 * Three things about this machine are baked in rather than discovered again:
 *
 * * **Firefox here is a snap** and cannot write outside `$HOME`. Output goes inside the repo,
 *   which is.
 * * **Firefox screenshots at the `load` event.** A scenario that has to click something would
 *   be captured before the click, so the page holds its own load event open with a slow image
 *   (`/__hold__`) served by this file, and the driving happens inside that window.
 * * **Motion is disabled** in the injected CSS. Otherwise a dialog is caught mid-fade and two
 *   runs of the same scenario differ, which is the difference between a screenshot and a
 *   regression test.
 */

import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)));
export const DIST = join(ROOT, 'dist');

/** How long the page holds its load event open, for scenarios that drive the UI. */
const HOLD_MS = 900;
/** Window size. 1280×860 is the prototype's own canvas plus the titlebar. */
export const SIZE = { width: 1280, height: 860 };

/* ---- Fixture data ---------------------------------------------------------
 *
 * Deliberately not a copy of the prototype's demo data: these values exercise the cases the
 * design has to survive — a long title, a vault name that overflows the sidebar, an item with
 * no tags, and every status the list can render.
 */

/** The id an item fixture gets, so a report can name one without repeating the shape. */
const ID = (key) => `00000000-0000-4000-8000-${key.padEnd(12, '0').slice(0, 12)}`;

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
    id: ID(id),
    kind,
    title,
    tags,
    status,
    favourite,
    created_at: 1735689600000,
    updated_at: 1785600000000,
  };
}

/**
 * What `watchtower_scan` answers with — §6.9, and it must agree with `ITEMS` above.
 *
 * The two halves of the Watchtower screen read from different places (the groups from this report,
 * *Safe* from each item's cached `status`), so a report that disagreed with the statuses would
 * photograph a screen the product cannot produce. Every finding here names an item whose status is
 * the verdict `worst()` would have cached for it.
 *
 * Three things it is built to exercise, none of which happen by accident:
 *
 * * **`bca` is in two groups.** It carries a `breached` finding *and* a `reused` one, which is the
 *   design's central asymmetry (§6.9): the report keeps both, and the item's single status pip is
 *   the louder of the two. A fixture with one verdict per item would have hidden that.
 * * **The reuse group has two members.** It has to: `shared_with` is rendered as *"Same as …"*, so
 *   a group of one would draw a sentence with nothing after it.
 * * **A `breached` finding, which nothing in the product produces yet.** Carried deliberately.
 *   `watchtower_breach_check` lands later this phase and the group's copy is otherwise on a screen
 *   no shot can reach — which is precisely how a false sentence sat undisturbed in Trash for a day.
 */
const WATCHTOWER_REPORT = {
  scanned_at: 1785600000000,
  passwords: 6,
  distinct: 5,
  findings: [
    {
      item_id: ID('aws'),
      field_id: 'f2',
      verdict: 'weak',
      score: 2,
      crack_time: '31 minutes',
      shared_with: [],
    },
    {
      item_id: ID('bca'),
      field_id: 'f2',
      verdict: 'breached',
      score: 3,
      crack_time: '3 hours',
      shared_with: [],
    },
    {
      item_id: ID('bca'),
      field_id: 'f2',
      verdict: 'reused',
      score: 3,
      crack_time: '3 hours',
      shared_with: [ID('ssh')],
    },
    {
      item_id: ID('ssh'),
      field_id: 'f2',
      verdict: 'reused',
      score: 3,
      crack_time: '3 hours',
      shared_with: [ID('bca')],
    },
  ],
};

/** A scanned vault with nothing wrong in it — the reassurance state, R-19. */
const CLEAN_REPORT = { scanned_at: 1785600000000, passwords: 2, distinct: 2, findings: [] };

/**
 * What `watchtower_breach_check` answers with — §6.9, R-25.
 *
 * It agrees with `ITEMS` in the same way `WATCHTOWER_REPORT` does, and for a sharper reason: the
 * Breached group draws its rows from each item's cached **status**, because the local scan never
 * produces that verdict and a report from this session is the only place a count exists. So a
 * fixture naming an item the statuses do not call breached would photograph a count with no row
 * to sit in.
 */
const BREACH_REPORT = {
  checked_at: 1785600000000,
  requested: 5,
  breached: [{ item_id: ID('bca'), field_id: 'f2', count: 1246 }],
  unchecked: [],
};

/**
 * The half-finished check — R-25's "not checked, never safe" in the state that produces it.
 *
 * The one state that cannot be reached by waiting: a network that fails for some values and not
 * others. Its own sentence is the point of the fixture, because "3 could not be checked" beside a
 * result is the difference between a report and a clean bill of health for everything silent.
 */
const PARTIAL_BREACH_REPORT = {
  checked_at: 1785600000000,
  requested: 3,
  breached: [{ item_id: ID('bca'), field_id: 'f2', count: 1246 }],
  unchecked: [
    { item_id: ID('github'), field_id: 'f2', reason: 'offline' },
    { item_id: ID('ssh'), field_id: 'f2', reason: 'offline' },
  ],
};

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

/**
 * Every field of `Settings` in `src-tauri/src/state.rs`, at its default.
 *
 * `ui_scale` and `launch_at_login` were **missing** until 2026-08-08 — both added by D-56 and
 * D-57 and neither backfilled here — so every screenshot and every a11y run of the Settings
 * screen had been taken against a settings object the host cannot produce. It was not cosmetic:
 * `Segmented` marks the option equal to `value` as its tab stop, so an absent `ui_scale` matched
 * no option and left **all three** Interface-size buttons at `tabindex="-1"`, which is a control
 * missing from the tab order entirely. The audits could not see it — `element.focus()` reaches a
 * `tabindex="-1"` button perfectly well, and that is the difference this fixture was hiding.
 *
 * Keep it exhaustive. A field added to `Settings` and not added here does not fail anything; it
 * quietly changes what every Settings surface is measured against.
 */
const SETTINGS = {
  theme: 'system',
  auto_lock_seconds: 300,
  clipboard_clear_seconds: 12,
  audit_log_enabled: false,
  sidebar_width: 232,
  list_width: 300,
  last_vault_path: '/home/shoel/Documents/personal.tvault',
  ui_scale: 'default',
  launch_at_login: false,
  // R-26, at its default. The two scenarios that turn it on do so by replacing this object,
  // which is how the Watchtower screen's "off" row and its "checked" row are both photographable.
  breach_check_enabled: false,
};

/** The same settings with the user opted in — R-26, and the only scenarios that send anything. */
const BREACH_ON = { ...SETTINGS, breach_check_enabled: true };

/* ---- Scenarios ------------------------------------------------------------ */

const UNLOCKED = {
  state: 'unlocked',
  path: '/home/shoel/Documents/personal.tvault',
  display_name: 'Personal Vault',
  item_count: ITEMS.length,
  // §6.9, and the same rule as `ui_scale` above: a field added to `VaultStatus` and not added
  // here does not fail anything, it quietly changes what every surface is measured against.
  // A scan has run; a breach check has not, which is the state **every** vault in v1 is in and
  // therefore the one the Watchtower screen should be photographed in.
  last_scan_at: 1785600000000,
  last_breach_check_at: null,
  // D-70. Empty, not absent, and not filled in: this is the state **every** vault is in until
  // somebody types a name, so it is what the twenty-odd existing surfaces should be measured
  // against. `NAMED` below is the other half, and it exists as a separate status for the three
  // scenarios that are about the profile rather than merely containing it.
  profile: { name: '', email: '' },
};

/** The same vault with an owner named — the state the design's mockups are all drawn in. */
const NAMED = {
  ...UNLOCKED,
  profile: { name: 'Budi Santoso', email: 'budi@warungpintar.id' },
};

export const SCENARIOS = {
  onboarding: {
    status: {
      state: 'no_vault',
      path: null,
      display_name: '',
      item_count: null,
      last_scan_at: null,
      last_breach_check_at: null,
      profile: null,
    },
  },
  lock: {
    status: {
      state: 'locked',
      path: '/home/shoel/Documents/personal.tvault',
      display_name: 'personal',
      item_count: null,
      // Both null while locked, for `item_count`'s reason — they live in the sealed body and
      // there is no key to read them with. A lock screen that drew "never checked" here would
      // be making a claim about a vault it cannot read.
      last_scan_at: null,
      last_breach_check_at: null,
      // `null` rather than empty, and it is the assertion the lock screen is worth having: a
      // locked vault cannot read its own profile, so a surface that drew one here would be
      // drawing something the host can never send — D-70.
      profile: null,
    },
  },
  shell: { status: UNLOCKED },
  settings: { status: UNLOCKED, drive: `click('[title="Settings"]')` },
  watchtower: { status: UNLOCKED, drive: `clickText('button', 'Watchtower')` },
  // The third state of the same screen, and the only one the prototype does not draw at all: a
  // scan the host refused. It matters more than it looks — empty groups and a refused scan are
  // indistinguishable to a reader unless the screen says which it is, which is R-25's "not
  // checked, never safe" applied to the local half. `fail` makes the stub reject that one command.
  watchtowerError: {
    status: { ...UNLOCKED, last_scan_at: null },
    fail: 'watchtower_scan',
    drive: `clickText('button', 'Watchtower')`,
  },
  // The other half of the same screen, and the one nobody would otherwise look at again: a vault
  // that was scanned and came back clean. Its copy is reassurance rather than a blank pane (R-19),
  // and it is a separate scenario because "no findings" is a different fixture — a clean report and
  // items whose status is `strong` — rather than a different rendering of the same one.
  watchtowerClean: {
    status: UNLOCKED,
    items: [
      item('github', 'login', 'GitHub', ['work', 'dev'], 'strong', true),
      item('stripe', 'login', 'Stripe', ['work'], 'strong', false),
    ],
    report: CLEAN_REPORT,
    drive: `clickText('button', 'Watchtower')`,
  },
  // The breach check, run. Driven through its own button rather than fed in as state, because
  // the button is the only thing in the product that starts it — R-26 makes the check something
  // a person asks for, and a scenario that set the report directly would photograph a screen no
  // sequence of clicks can produce. The drive matches the **verb** rather than the whole label:
  // this scenario carries a stamp from an earlier session so its button reads *Check again*,
  // and matching "Check now" here clicked nothing and photographed the state before the run.
  watchtowerChecked: {
    status: { ...UNLOCKED, last_breach_check_at: 1785600000000 },
    settings: BREACH_ON,
    drive: `clickText('button', 'Watchtower'); await sleep(200);
            clickText('button', 'Check'); await sleep(400)`,
  },
  // The same screen after a check that only partly landed. R-25's whole sentence lives here:
  // what was found, and how much was not checked, in one line that cannot be read as a pass.
  watchtowerPartial: {
    status: { ...UNLOCKED, last_breach_check_at: null },
    settings: BREACH_ON,
    breach: PARTIAL_BREACH_REPORT,
    drive: `clickText('button', 'Watchtower'); await sleep(200);
            clickText('button', 'Check'); await sleep(400)`,
  },
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
  // The three profile surfaces — D-70. Driven off `[aria-haspopup="menu"]` rather than a class,
  // because Svelte hashes component classes and there is exactly one popup trigger in the shell.
  //
  // `profileMenu` and `editProfile` run against `NAMED`, which is the state they are drawn for.
  // `settings` above keeps `UNLOCKED`, so the Settings shot stays the **empty** profile card —
  // what a user sees on their first visit, and the one state nobody remembers to look at
  // because whoever built it always has a profile. `settingsProfile` is the filled-in half.
  profileMenu: {
    status: NAMED,
    drive: `click('[aria-haspopup="menu"]'); await sleep(300)`,
  },
  editProfile: {
    status: NAMED,
    drive: `click('[aria-haspopup="menu"]'); await sleep(200);
            clickText('button', 'Edit profile…'); await sleep(300)`,
  },
  settingsProfile: { status: NAMED, drive: `click('[title="Settings"]')` },
  deleteItem: { status: UNLOCKED, drive: `click('[title="Delete item"]')` },
  // The two empty states R-19 is actually about, and the only two a screenshot can reach: the
  // vault with nothing in it, and Trash — which is empty by construction and stays that way
  // (D-49), so it is the one whose copy nobody would otherwise ever look at again.
  emptyVault: {
    // `last_scan_at: null` and not the inherited timestamp: a vault created a minute ago has
    // never been scanned, and every empty state on this screen is drawn for that vault.
    status: { ...UNLOCKED, display_name: 'Fresh Vault', item_count: 0, last_scan_at: null },
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
function harness(scenario, theme, audit) {
  // A scenario may replace the item fixture wholesale — that is how an empty vault is shot,
  // and it is a substitution rather than a flag because "the list is empty" is a different
  // fixture, not a different rendering of the same one.
  const {
    status,
    drive,
    items = ITEMS,
    report = WATCHTOWER_REPORT,
    settings = SETTINGS,
    breach = BREACH_REPORT,
    fail = '',
  } = SCENARIOS[scenario];
  return `
<script>
window.__SHOT__ = ${JSON.stringify({ scenario, theme })};
const STATUS = ${JSON.stringify(status)};
const ITEMS = ${JSON.stringify(items)};
const FIELDS = ${JSON.stringify(FIELDS)};
const SETTINGS = ${JSON.stringify(settings)};
const IMPORT_REPORT = ${JSON.stringify(IMPORT_REPORT)};
const WATCHTOWER_REPORT = ${JSON.stringify(report)};
const BREACH_REPORT = ${JSON.stringify(breach)};
const FAIL = ${JSON.stringify(fail)};

let listener = 0;
function respond(cmd, args) {
  // One command made to fail, as an IpcError -- the shape asIpcError() reads. Thrown rather than
  // returned, because a refusal that came back as a value would exercise the success path.
  if (cmd === FAIL) throw { kind: 'io', message: 'The vault file could not be written.' };
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
    // D-70. Null because the real one returns nothing, and the shell re-reads vault_status
    // afterwards -- which here answers with the static STATUS, so a driven save shows the
    // dialog closing rather than the name changing. That is the honest limit of a stub.
    case 'set_profile': return null;
    case 'create_vault': case 'unlock_recovery_kit':
      return { recovery_code: 'K7QX-2MRE-9WVT-4HDP-6SNA-3JFB' };
    // D-69 split the write off create_vault. Answered here so a drive that reaches step 3's
    // acknowledgement completes instead of erroring, and answered with null because the real
    // one returns nothing -- a stub that invented a payload would let a caller start reading one.
    case 'commit_vault': return null;
    // The picker (D-59). A fixed path, because the name is on screen in the dialog header and a
    // shot whose header changes per machine cannot be a baseline. This is also the one stub in
    // the harness that stands in for a **native** dialog rather than for a command: there is no
    // file chooser in a headless browser, so without this the surface is unphotographable.
    case 'pick_import_file': return '/home/shoel/Downloads/bitwarden_export.json';
    case 'pick_vault_file': return null;
    // D-60's save dialog. Null, like a cancelled picker, so a shot of onboarding keeps the
    // resolved default path in the field rather than one this harness chose.
    case 'pick_new_vault_path': return null;
    // Both halves answer with the same report, which is what the real pair does for a file
    // nobody edited in between -- and the difference between them (commit re-reads, so it can
    // differ) is a race no screenshot can hold still anyway.
    case 'import_preview': case 'import_commit': return IMPORT_REPORT;
    // §6.9. The real one re-scores every password and saves the vault; this one is a fixture that
    // agrees with ITEMS, because the screen reads its groups from here and its Safe count from the
    // statuses over there.
    case 'watchtower_scan': return WATCHTOWER_REPORT;
    // The one command in the product that would open a socket. Here it opens nothing and answers
    // a fixture -- which is also why the scenarios drive it through its button: the shot is of
    // the screen a user reaches by asking, not of a state assembled behind the surface.
    case 'watchtower_breach_check': return BREACH_REPORT;
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
  ${audit ? `await sleep(150);\n${audit}` : ''}
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

/**
 * Serves `dist/` with the stub injected, on an ephemeral port.
 *
 * `audits` is a map of name to module source, selected with `?audit=<name>`, and appended
 * after the scenario's own driving. `onReport` is what makes a browser that can only produce a
 * PNG useful for something other than a picture: the audit posts its findings to `/__report__`
 * and this hands them back to the tool. Firefox's headless mode has no way to return a value,
 * so the page has to say what it found while it is still on screen.
 */
export async function serve({ onReport, audits = {} } = {}) {
  const template = await readFile(join(DIST, 'index.html'), 'utf8');

  const server = createServer(async (request, response) => {
    const url = new URL(request.url, 'http://localhost');

    if (url.pathname === '/__report__' && request.method === 'POST') {
      const chunks = [];
      for await (const chunk of request) chunks.push(chunk);
      try {
        onReport?.(JSON.parse(Buffer.concat(chunks).toString('utf8')));
      } catch (error) {
        onReport?.({ error: String(error) });
      }
      response.writeHead(204).end();
      return;
    }

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
      const audit = audits[url.searchParams.get('audit')] ?? null;
      const page = template
        .replace('<head>', `<head>${harness(scenario, theme, audit)}`)
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
