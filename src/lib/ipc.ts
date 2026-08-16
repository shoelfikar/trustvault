/**
 * The webview's half of `docs/ipc-contract.md`.
 *
 * Every command in the app goes through this file, and nothing else calls `invoke` directly.
 * That is not tidiness — it is the only place a reviewer has to read to answer "what can reach
 * plaintext from here", and a stray `invoke` elsewhere would make that answer wrong.
 *
 * The four functions that can return a secret are grouped and labelled at the bottom. If that
 * group grows, the contract's budget of four has been spent without anyone deciding to.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/* ---- Shapes, mirroring §6.1 --------------------------------------------- */

export type VaultState = 'no_vault' | 'locked' | 'unlocked';

/**
 * Who the vault belongs to, as a label — §5, D-70.
 *
 * **Not an account.** Nothing authenticates against these two strings and nothing is sent
 * anywhere (D-03); they label the sidebar footer, the Settings card and the recovery kit. Both
 * are empty until the user fills them in, because onboarding's three steps do not ask.
 */
export interface Profile {
  name: string;
  email: string;
}

export interface VaultStatus {
  state: VaultState;
  path: string | null;
  /** While locked this is the **file stem**, not the name typed at onboarding — §5. */
  displayName: string;
  itemCount: number | null;
  /**
   * When the last local Watchtower scan finished, or `null` — §6.9.
   *
   * **`null` has two causes and they must not be merged**: the vault is locked, so the timestamp
   * cannot be read out of the sealed body, or it is unlocked and has never been scanned. Read
   * `state` to tell them apart before rendering "never checked" at anybody.
   */
  lastScanAt: number | null;
  /**
   * When the last breach check finished, or `null` — §6.9.
   *
   * Always `null` today: nothing writes it until the breach check ships. It is separate from
   * `lastScanAt` on purpose — a local scan from this morning must never be shown as evidence
   * that a breach check ran.
   */
  lastBreachCheckAt: number | null;
  /**
   * Who the vault belongs to, or `null` while locked — §5, D-70.
   *
   * `null` and `{ name: '', email: '' }` are different answers and the footer draws them
   * differently: `null` is "locked, so unknown" and falls back to the vault's own name, while
   * the empty pair is "unlocked, and nobody has filled it in". Do not collapse them.
   */
  profile: Profile | null;
}

export type FieldKind =
  'text' | 'username' | 'password' | 'url' | 'email' | 'otp' | 'note' | 'date';

export type ItemKind = 'login' | 'api_key' | 'card' | 'note' | 'wifi' | 'ssh_key' | 'identity';

export type ItemStatus = 'unknown' | 'strong' | 'weak' | 'reused' | 'breached' | 'expired';

export interface FieldSummary {
  id: string;
  label: string;
  kind: FieldKind;
  secret: boolean;
  /** Present only when `secret` is false. */
  value: string | null;
  /**
   * Present only when `secret` is true. **Fixed width, not the secret's length** (D-32) —
   * never present it as one, and never derive anything from `mask.length`.
   */
  mask: string | null;
  /**
   * Whether the user or an import added this field, rather than it being one of the type's
   * own — D-43. Stored in the vault, never inferred here: deriving it from the label is the
   * mistake `secret` already refuses to make.
   */
  custom: boolean;
}

export interface ItemSummary {
  id: string;
  kind: ItemKind;
  title: string;
  tags: string[];
  status: ItemStatus;
  favourite: boolean;
  createdAt: number;
  updatedAt: number;
}

export type ItemDetail = ItemSummary & { fields: FieldSummary[] };

/**
 * One field of an item being created — §6.4.
 *
 * The only shape in this file carrying plaintext **outbound**, and §5 already concedes that
 * direction: the password is in this heap the moment the user types it, and nothing here can
 * take it back out. What the host does with it is move it into a `SecretString` that zeroizes.
 */
export interface NewField {
  label: string;
  kind: FieldKind;
  value: string;
  secret: boolean;
  custom: boolean;
}

/**
 * One field as the edit form submits it — §6.4.
 *
 * Two nullable members meaning different things, and the second is the single most
 * load-bearing detail on this boundary:
 *
 * - `id: null` **creates** a field; an id **edits** the one it names.
 * - **`value: null` means UNCHANGED.** The edit form never received the secret values — §6.1
 *   elides them, which is the whole architecture — so a form that sent back what it is holding
 *   would send back masks, and renaming an item would overwrite every password in it with
 *   `"••••••••••••"`. The previous values would land in `history`, where no v1 surface reaches.
 *
 * And a third rule with no syntax to carry it: **omission deletes**, so this array must hold
 * every field that is to survive, in the order they are to appear.
 */
export interface EditField {
  id: string | null;
  label: string;
  kind: FieldKind;
  value: string | null;
  secret: boolean;
  custom: boolean;
}

export type Theme = 'system' | 'light' | 'dark';

/** 92 % / 100 % / 115 % — R-21. The three steps `tokens.css` names, not a free percentage. */
export type UiScale = 'compact' | 'default' | 'large';

export interface Settings {
  theme: Theme;
  autoLockSeconds: number;
  clipboardClearSeconds: number;
  /** Off by default — D-31. */
  auditLogEnabled: boolean;
  /** Sidebar width in px, clamped to MASTER.md §4's 180–320. */
  sidebarWidth: number;
  /** Item-list width in px, clamped to §4's 240–460. */
  listWidth: number;
  /**
   * The vault opened last — **host-owned, read-only here** (D-40).
   *
   * It rides in this struct because it shares the settings file, not because the webview may
   * set it: the host overwrites whatever arrives in this field with what it already had. It is
   * what makes a relaunch land on the lock screen instead of onboarding.
   */
  readonly lastVaultPath: string | null;
  /**
   * How large the whole interface draws — R-21.
   *
   * Applied by setting `data-ui-scale` on the root, which `tokens.css` turns into
   * `--ui-scale`; every size token is derived from it, so this is the one setting that moves
   * every measurement in the application at once.
   */
  uiScale: UiScale;
  /**
   * Whether the OS starts TrustVault at login — R-21.
   *
   * **The only setting whose write can fail.** It is a desktop entry, a `LaunchAgent` or a
   * registry value depending on the platform, and `setSettings` rejects with `io` rather than
   * storing a value the platform refused — so a caller must re-read from the resolved
   * settings rather than assume its own optimistic value took.
   */
  launchAtLogin: boolean;
  /**
   * Window geometry — R-27, and **host-owned like `lastVaultPath`**.
   *
   * Only the host measures the window, and it does so on every resize. These ride in the
   * struct because they share the settings file (D-33's one-store argument); sending changed
   * values back here does nothing, because the host keeps its own.
   */
  readonly windowWidth: number;
  readonly windowHeight: number;
  readonly windowMaximized: boolean;
  /**
   * Whether Watchtower may ask Have I Been Pwned about a password — R-26. **Off by default.**
   *
   * The switch on the only network call in the product. It is read in the **host**, inside
   * `watchtower_breach_check`, never here: a caller that passed the flag as an argument would put
   * the decision to send anything at all in the layer this whole boundary exists not to trust.
   * Turning it off here turns the egress off there.
   */
  breachCheckEnabled: boolean;
}

export interface KdfSummary {
  mCost: number;
  tCost: number;
  pCost: number;
}

export interface Strength {
  score: 0 | 1 | 2 | 3 | 4;
  /** Weak / Fair / Strong / Excellent, or empty before anything is typed. */
  label: string;
  crackTime: string;
}

/** Which character classes a generated password may draw from — §7. */
export interface CharSets {
  lowercase: boolean;
  uppercase: boolean;
  digits: boolean;
  symbols: boolean;
}

/** All four classes, which is what every surface asks for today. */
export const ALL_SETS: CharSets = {
  lowercase: true,
  uppercase: true,
  digits: true,
  symbols: true,
};

/**
 * What `generate_password` returns — §7.
 *
 * The score rides along rather than taking a second call to `score_password`, and that is a
 * safety property rather than a convenience: re-scoring would send the value across the
 * boundary a second time for a number the generating side already had the inputs for.
 */
export type Generated = Strength & { password: string };

/* ---- Watchtower, mirroring §6.9 ------------------------------------------ */

/**
 * What a finding says about one password field — §6.9.
 *
 * `breached` comes from the breach check and nothing produces it yet; `expired` is produced by
 * **nothing at all** in v1 — no requirement defines a rotation date, so the view's group for it
 * stays empty rather than implying a check that is not running.
 */
export type Verdict = 'weak' | 'reused' | 'breached' | 'expired';

/**
 * One verdict about one password field — §6.9, R-23, R-24.
 *
 * Carries no password and nothing derived from one. The reuse grouping key is a hash of a
 * password and **a hash of a short secret is a secret**, so it never crosses: `sharedWith` names
 * the other **items**, which are ids this side already holds from the item list.
 *
 * A field can be both weak and reused, and then there are **two** findings for it rather than
 * one ranked verdict. The ranking happens only in the item's cached `status`, which is what the
 * list draws its single pip from.
 */
export interface Finding {
  itemId: string;
  fieldId: string;
  verdict: Verdict;
  /** zxcvbn 0–4 — R-24. */
  score: number;
  /** zxcvbn's own phrasing: "31 minutes", "centuries" — R-24. */
  crackTime: string;
  /** The other items carrying the same value. Empty unless `verdict` is `reused`. */
  sharedWith: string[];
}

/**
 * What a local scan concluded — §6.9.
 *
 * **A clean field is the absence of a row**, never a row saying strong: `findings` holds only
 * what is worth acting on, and an item that passes is absent. `distinct` is what a breach check
 * would cost — one range request per distinct value, never per item (S-07b).
 */
export interface WatchtowerReport {
  scannedAt: number;
  passwords: number;
  distinct: number;
  findings: Finding[];
}

/** One password field found in a breach corpus — §6.9, R-25. */
export interface BreachHit {
  itemId: string;
  fieldId: string;
  /**
   * How many times the value appears in the corpus.
   *
   * A property of the **corpus**, not of the password: narrowing a password from it would need
   * the range response, and that never leaves the host.
   */
  count: number;
}

/**
 * One password field the check could not answer for — §6.9, R-25.
 *
 * **This is "not checked", and it must never render as "safe".** A field here kept whatever
 * status the local scan gave it; nothing about it was proven in either direction.
 */
export interface UncheckedField {
  itemId: string;
  fieldId: string;
  /** `off` — the setting; `offline` — it never reached the service; `http` — a refusal. */
  reason: 'off' | 'offline' | 'http';
}

/**
 * What a breach check concluded — §6.9, R-25, R-26.
 *
 * Carries no prefix, no suffix and no hash. `requested` is a **count** of range requests and
 * deliberately not a list of prefixes: a list would be a description of this vault's passwords,
 * in the one heap that cannot be wiped. It is `0` exactly when the setting is off.
 */
export interface BreachReport {
  checkedAt: number;
  requested: number;
  breached: BreachHit[];
  unchecked: UncheckedField[];
}

export type ErrorKind =
  | 'not_a_vault'
  | 'unsupported_version'
  | 'unreadable'
  | 'malformed_recovery_code'
  | 'locked'
  | 'no_such_item'
  | 'no_such_field'
  | 'not_secret'
  | 'clipboard'
  // The Phase 3 kinds (§4). All three are decided **before** any key material or vault content
  // is involved, which is the whole of why they are safe to distinguish while `unreadable`
  // is not. `not_importable` was missing from this union until 2026-08-06 — the host has
  // returned it since the importer landed, and this side would have narrowed it to `internal`.
  | 'not_importable'
  | 'malformed_totp_secret'
  // The typed name in the vault-deletion dialog did not match — R-18. The one confirmation
  // the **host** verifies, so this side must be able to name it: narrowed to `internal`, the
  // dialog would tell a user who mistyped that something went wrong inside TrustVault.
  | 'confirmation_mismatch'
  // A vault was asked for at a path that already holds a file — D-62. Reachable from onboarding
  // only, and the one refusal in the product that protects a file the user is not looking at.
  | 'path_in_use'
  | 'io'
  | 'internal';

export interface IpcError {
  kind: ErrorKind;
  message: string;
}

/**
 * Narrows an unknown rejection to the contract's error shape.
 *
 * Tauri rejects with whatever the command returned, so a thrown value is *usually* an
 * `IpcError` and must not be assumed to be one — a panic in the host arrives here as a string.
 */
export function asIpcError(error: unknown): IpcError {
  if (
    typeof error === 'object' &&
    error !== null &&
    'kind' in error &&
    'message' in error &&
    typeof (error as IpcError).message === 'string'
  ) {
    return error as IpcError;
  }
  return { kind: 'internal', message: 'Something went wrong inside TrustVault.' };
}

/* ---- Case translation ---------------------------------------------------- */

/**
 * Rust serializes `snake_case`; the frontend reads `camelCase`.
 *
 * Done here rather than with a serde attribute so the wire format stays exactly what
 * `docs/ipc-contract.md` documents. A rename attribute would make the contract describe a
 * shape no one could see in the Rust source.
 */
function camel<T>(value: unknown): T {
  if (Array.isArray(value)) return value.map((entry) => camel(entry)) as T;
  if (value === null || typeof value !== 'object') return value as T;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, entry]) => [
      key.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase()),
      camel(entry),
    ]),
  ) as T;
}

/**
 * The mirror of `camel`, and it descends into arrays for the same reason that one does.
 *
 * Written without the array case first, which was harmless only because every shape crossing
 * outbound happened to have single-word keys. `fields: EditField[]` is the first array of
 * objects on this boundary, so the omission would have become a silently dropped key the day
 * one of them was named with two words.
 */
function snake<T>(value: T): T {
  if (Array.isArray(value)) return value.map((entry) => snake(entry)) as T;
  if (value === null || typeof value !== 'object') return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, entry]) => [
      key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`),
      snake(entry),
    ]),
  ) as T;
}

async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  return camel<T>(await invoke(command, snake(args)));
}

/* ---- Ambient — §5 -------------------------------------------------------- */

export const buildInfo = () =>
  call<{ version: string; formatVersion: number; extension: string }>('build_info');

export const vaultStatus = () => call<VaultStatus>('vault_status');

export const scorePassword = (password: string, inputs: string[] = []) =>
  call<Strength>('score_password', { password, inputs });

export const calibrateKdf = () => call<KdfSummary>('calibrate_kdf');

/**
 * Copies a **not-yet-stored** password and schedules the clear — §5, D-37, D-44.
 *
 * The only function here that hands a secret outbound on purpose. It is safe for the reason
 * the contract states: the host reads nothing and returns nothing, so it cannot disclose
 * anything this side did not already hold — and what it buys is the one thing this side
 * cannot do for itself, a clipboard clear scheduled in Rust. Copying a generated password
 * with `navigator.clipboard` instead would leave it in the clipboard for good.
 */
export const copyGenerated = (password: string) =>
  call<{ clearsAt: number }>('copy_generated', { password });

/**
 * One code and what the countdown ring needs — §5, §6.6.
 *
 * `expiresAt` is the **step** boundary, not "now plus the period": every authenticator in the
 * world rolls over at the same instant, and a ring that started when the pane opened would
 * disagree with the phone lying beside the keyboard.
 */
export interface TotpCode {
  code: string;
  expiresAt: number;
  period: number;
  digits: number;
}

/**
 * One code from a seed the user is still typing — §5, R-20.
 *
 * Ambient because there is no item yet. It doubles as the seed's validator, which is the point
 * of having it: a base32 string that will not decode is caught while the field is on screen,
 * rather than a month later at a login prompt with the phone already wiped.
 */
export const totpPreview = (secret: string) => call<TotpCode>('totp_preview', { secret });

export const getSettings = () => call<Settings>('get_settings');

/**
 * Replaces the settings — §6.3.
 *
 * **Can reject.** `launch_at_login` writes outside the process, and the host returns `io`
 * without storing anything when the platform refuses. A caller must therefore take the
 * *returned* settings as the truth rather than the object it sent, or the screen shows a
 * toggle promising something nothing registered.
 */
export const setSettings = (settings: Settings) => call<Settings>('set_settings', { settings });

/* ---- The picker — §5, D-59 ------------------------------------------------ */

/**
 * Asks the host to open a native file dialog, and gets back a **path**.
 *
 * Not an `<input type="file">`, and that is the whole reason these two exist. An `<input>`
 * hands *this heap* the file's contents, and for an import those contents are another password
 * manager's plaintext — the one thing `CLAUDE.md` says can never be wiped once it is here. The
 * host opens the file itself; nothing but a string crosses.
 *
 * `null` means the user closed the dialog. That is the ordinary outcome of opening a picker and
 * must never be reported as an error.
 *
 * The title and the filter are fixed in Rust, so there is no call from here that turns "choose
 * an export" into "choose anything". The plugin's own dialog commands are **denied** to this
 * webview by `capabilities/default.json`; these are the only three doors, and `ipc_audit.rs`
 * asserts that has not changed.
 */
export const pickImportFile = () => call<string | null>('pick_import_file');

/** The switcher's "Open vault file…" — the path goes straight to `switchVault`. */
export const pickVaultFile = () => call<string | null>('pick_vault_file');

/**
 * Onboarding's "Change" — a **save** dialog, because the vault does not exist yet — D-60.
 *
 * `suggested` only pre-fills the file-name field; the host reduces it to its own file name, and
 * the user reads the result in the dialog before confirming it.
 */
export const pickNewVaultPath = (suggested: string) =>
  call<string | null>('pick_new_vault_path', { suggested });

/* ---- Import — §6.8, R-29, D-42 -------------------------------------------- */

/**
 * One field that could not be imported at all, named but **never quoted** — §6.8.
 *
 * `field` is the label, not the value. A report that carried the values it failed to understand
 * would be a plaintext dump of exactly the parts of the foreign vault we understood least.
 */
export interface Refusal {
  itemTitle: string;
  field: string;
  reason: string;
}

/** A field that landed with a shape change worth telling the user about. Also never quoted. */
export interface Converted {
  itemTitle: string;
  field: string;
  note: string;
}

/**
 * What an import would do, or did — §6.8.
 *
 * Three outcomes and not two: **mapped** (counted in `perKind`, listed nowhere), **converted**,
 * and **refused**. R-29 is met only when every field in the export is one of the three, so the
 * surface has to show `refusals` in full rather than as a count — a truncated refusal list is
 * the silent drop the requirement exists to forbid, one indirection further out.
 */
export interface ImportReport {
  total: number;
  perKind: { kind: ItemKind; count: number }[];
  tagsCreated: string[];
  tagsMerged: string[];
  converted: Converted[];
  refusals: Refusal[];
}

/** Reports what an import would do. **Writes nothing** — §6.8. */
export const importPreview = (path: string) => call<ImportReport>('import_preview', { path });

/**
 * Imports the export as one transaction, then saves — R-29.
 *
 * It **re-reads and re-parses the file** rather than taking the preview's result, so a file
 * edited between the two calls imports as it is now, not as it was previewed. That is why this
 * returns a report of its own, and why the report shown *after* an import is the authoritative
 * one — the surface must display this one, not keep the preview on screen.
 */
export const importCommit = (path: string) => call<ImportReport>('import_commit', { path });

/* ---- Multi-vault — §6.7, R-22 -------------------------------------------- */

/**
 * One row of the switcher.
 *
 * `displayName` is the **file stem** for every vault except the open one, and the surface must
 * not imply otherwise: the real name is inside the sealed body, so with no key there is
 * nothing to read it with.
 */
export interface VaultRef {
  path: string;
  displayName: string;
  lastOpenedAt: number | null;
}

/** Ambient — the switcher's job is to work while nothing is unlocked. */
export const listVaults = () => call<VaultRef[]>('list_vaults');

/**
 * Points the app at another vault — R-22.
 *
 * **Locks and zeroizes the outgoing vault first**, then moves, and leaves the state `locked`:
 * it is given no password, so it cannot and does not unlock. Callers re-read `vaultStatus`
 * rather than assuming, which is what puts the lock screen up.
 */
export const switchVault = (path: string) => call<void>('switch_vault', { path });

/**
 * Removes a vault from the list. **The file is untouched** — this is "Leave vault".
 *
 * Confusing this with `deleteVault` would be the worst bug in the application, which is why
 * they are kept apart in the command name, the confirmation, and the words on the button.
 */
export const forgetVault = (path: string) => call<void>('forget_vault', { path });

/**
 * Erases a vault file — R-18, R-22.
 *
 * `confirmName` must equal the `displayName` `listVaults` reports for that path, and **the
 * host checks it**, rejecting with `confirmation_mismatch` and deleting nothing. That is the
 * asymmetry with `deleteItem`, whose confirmation is only in the UI: a wrong item delete costs
 * one entry, and a wrong vault delete costs everything with no undo anywhere in the product.
 */
export const deleteVault = (path: string, confirmName: string) =>
  call<void>('delete_vault', { path, confirmName });

/* ---- Vault-class — §6.3 -------------------------------------------------- */

export const unlock = (path: string, password: string) => call<void>('unlock', { path, password });

export const lock = () => call<void>('lock');

/**
 * Labels the vault with its owner and saves — §6.4, D-70.
 *
 * There is no `getProfile`: the read path is `profile` on `vaultStatus`, because both come out
 * of the same body and are `null` in exactly the same state.
 *
 * Both strings are trimmed by the host, and **an unchanged profile is not a save** — the dialog
 * calls this whether or not anything was typed, and the host is what decides not to rewrite the
 * file. Nothing here needs to compare before calling.
 */
export const setProfile = (name: string, email: string) =>
  call<void>('set_profile', { name, email });

export const listItems = () => call<ItemSummary[]>('list_items');

export const getItem = (itemId: string) => call<ItemDetail>('get_item', { itemId });

/**
 * Ranks the vault against a palette query — §6.5, R-16, D-46.
 *
 * The matching runs in Rust, against plaintext that never leaves the core, and what comes back
 * is the same elided summary the list gets — **no field values, not even the one that matched**.
 * The alternative was to hold every username and URL in this heap and filter here: permitted by
 * §6.1 and still the wrong trade, which is why it took a decision rather than a preference.
 *
 * A **secret field's value is never matched**, so this cannot be used to confirm a guessed
 * password through the order of the rows.
 */
export const searchItems = (query: string, limit: number) =>
  call<ItemSummary[]>('search_items', { query, limit });

/**
 * Copies a field to the clipboard. **Returns no value** — that absence is R-10.
 *
 * The secret is written to the clipboard by Rust and never enters this heap.
 */
export const copyField = (itemId: string, fieldId: string) =>
  call<{ clearsAt: number }>('copy_field', { itemId, fieldId });

/**
 * Creates an item and saves the vault — §6.4.
 *
 * Returns the identifier and nothing else; the caller re-reads through `getItem`, so there is
 * one elision path in the application rather than two that drift apart.
 */
export const addItem = (kind: ItemKind, title: string, tags: string[], fields: NewField[]) =>
  call<{ itemId: string }>('add_item', { kind, title, tags, fields });

/** Rewrites an item from an edit form and saves — §6.4. See `EditField` for the three rules. */
export const updateItem = (
  itemId: string,
  title: string,
  tags: string[],
  favourite: boolean,
  fields: EditField[],
) => call<void>('update_item', { itemId, title, tags, favourite, fields });

/**
 * Deletes an item and saves.
 *
 * The confirmation R-18 asks for is **here**, in the UI, and the contract says so: what it
 * guards against is a mis-click, and the caller is the only user. The host does not re-check.
 */
export const deleteItem = (itemId: string) => call<void>('delete_item', { itemId });

/**
 * The current code for one item — §6.6, R-20, D-45.
 *
 * **Not sanctioned, and not a way to read the seed.** The seed is a `secret: true` field and
 * comes out, if ever, through `revealField` like any other; what crosses here is a code, which
 * D-45 exempts because it is not the credential, it is single-use, and the protocol's own
 * operation is to type it into somebody else's form.
 *
 * One item at a time, the selected one. There is deliberately no batched form and no code in
 * `listItems`: a list of live codes is a list of secrets on a refresh timer.
 */
export const totpCode = (itemId: string) => call<TotpCode>('totp_code', { itemId });

/**
 * Scores every password in the vault and groups the reused ones — §6.9, R-23, R-24.
 *
 * **The local half, and it touches no network.** The breach check is the separate command
 * below, which is the shape of the S-10 promise rather than a note about it: with one combined
 * command, "zero packets when breach checking is off" would be a branch inside the host; with
 * two, it is a command nobody calls.
 *
 * It is not free — S-07a budgets 500 ms and it measures 61 ms for a thousand items — and it
 * **writes to the vault**, caching a status per item so the list can draw its pips without
 * re-scanning. Call it when the user opens Watchtower or asks for a re-check, not on every
 * render.
 */
export const watchtowerScan = () => call<WatchtowerReport>('watchtower_scan');

/**
 * Asks Have I Been Pwned about every distinct password — §6.9, R-25, R-26.
 *
 * **The only call in this file that reaches the network, and it takes no argument.** Whether
 * anything leaves the machine is read from `breachCheckEnabled` in the host; with the setting
 * off this resolves to a report with `requested: 0` and every password in `unchecked` — a
 * refusal, not a rejection, so it must not be rendered as a failure to retry.
 *
 * Minutes rather than milliseconds on a large vault (S-07b: one request per distinct value, and
 * roughly two per second), so it is called when a person asks for it and never on opening a
 * screen. `onWatchtowerProgress` is how the wait is made legible.
 */
export const watchtowerBreachCheck = () => call<BreachReport>('watchtower_breach_check');

/* ---- Sanctioned — §7 ----------------------------------------------------- */
/* Exactly four. A fifth entry in this group is a decision, not a patch.       */

/**
 * Creates the vault **in memory** and returns its recovery code, once.
 *
 * Nothing is on disk when this resolves — D-69. `commitVault` below is what writes, and it is
 * onboarding step 3's acknowledgement that calls it.
 */
export const createVault = (name: string, path: string, password: string, kdf: KdfSummary) =>
  call<{ recoveryCode: string }>('create_vault', { name, path, password, kdf });

/**
 * Writes the vault `createVault` left pending, and opens it — D-69.
 *
 * **Not sanctioned**, and it belongs beside `createVault` rather than in the group above only
 * because the two are one operation split across a screen: it returns nothing at all, which is
 * what keeps the sanctioned budget at four. The secret crossed on the way in; this is the
 * acknowledgement going back.
 */
export const commitVault = () => call<void>('commit_vault', {});

export const unlockRecoveryKit = (path: string, code: string) =>
  call<{ recoveryCode: string }>('unlock_recovery_kit', { path, code });

export const revealField = (itemId: string, fieldId: string) =>
  call<{ value: string; remaskAt: number }>('reveal_field', { itemId, fieldId });

/**
 * Mints one password in the host and returns it with its score — R-15, D-44.
 *
 * The fourth sanctioned command, and the only one returning a secret the vault has never
 * seen. It replaced a generator that ran here, in the webview: two generators with one of
 * them being "the real one" is a distinction that survives exactly as long as the person who
 * remembers it, and the host's is the one whose randomness sits on the path R-06 constrains.
 *
 * Held under the render-and-drop rule like the other three — the dialog holds it while it is
 * open and drops it on close. Copying it goes through `copyGenerated`, never
 * `navigator.clipboard`.
 */
export const generatePassword = (length: number, sets: CharSets, excludeAmbiguous = true) =>
  call<Generated>('generate_password', { length, sets, excludeAmbiguous });

/* ---- Events — §8 --------------------------------------------------------- */

export type LockReason = 'manual' | 'timeout' | 'os_sleep';

export const onVaultLocked = (handler: (reason: LockReason) => void): Promise<UnlistenFn> =>
  listen<{ reason: LockReason }>('vault-locked', (event) => handler(event.payload.reason));

export const onFieldRemasked = (
  handler: (itemId: string, fieldId: string) => void,
): Promise<UnlistenFn> =>
  listen<{ item_id: string; field_id: string }>('field-remasked', (event) =>
    handler(event.payload.item_id, event.payload.field_id),
  );

export const onClipboardCleared = (
  handler: (itemId: string, fieldId: string) => void,
): Promise<UnlistenFn> =>
  listen<{ item_id: string; field_id: string }>('clipboard-cleared', (event) =>
    handler(event.payload.item_id, event.payload.field_id),
  );

/**
 * How far a breach check has got — §8.
 *
 * Two integers and nothing else. The event names no item and carries no prefix: a progress
 * event saying which entry is being checked would be a running commentary on the vault, emitted
 * on a timer with no user action behind any of it.
 */
export const onWatchtowerProgress = (
  handler: (done: number, total: number) => void,
): Promise<UnlistenFn> =>
  listen<{ done: number; total: number }>('watchtower-progress', (event) =>
    handler(event.payload.done, event.payload.total),
  );

export const defaultVaultPath = (name: string) => call<string>('default_vault_path', { name });
