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

export interface VaultStatus {
  state: VaultState;
  path: string | null;
  /** While locked this is the **file stem**, not the name typed at onboarding — §5. */
  displayName: string;
  itemCount: number | null;
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

/* ---- Sanctioned — §7 ----------------------------------------------------- */
/* Exactly four. A fifth entry in this group is a decision, not a patch.       */

export const createVault = (name: string, path: string, password: string, kdf: KdfSummary) =>
  call<{ recoveryCode: string }>('create_vault', { name, path, password, kdf });

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

export const defaultVaultPath = (name: string) => call<string>('default_vault_path', { name });
