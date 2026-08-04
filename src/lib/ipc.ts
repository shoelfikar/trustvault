/**
 * The webview's half of `docs/ipc-contract.md`.
 *
 * Every command in the app goes through this file, and nothing else calls `invoke` directly.
 * That is not tidiness — it is the only place a reviewer has to read to answer "what can reach
 * plaintext from here", and a stray `invoke` elsewhere would make that answer wrong.
 *
 * The three functions that can return a secret are grouped and labelled at the bottom. If that
 * group grows, the contract's budget of three has been spent without anyone deciding to.
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

export type Theme = 'system' | 'light' | 'dark';

export interface Settings {
  theme: Theme;
  autoLockSeconds: number;
  clipboardClearSeconds: number;
  /** Off by default — D-31. */
  auditLogEnabled: boolean;
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

function snake(value: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(
    Object.entries(value).map(([key, entry]) => [
      key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`),
      entry !== null && typeof entry === 'object' && !Array.isArray(entry)
        ? snake(entry as Record<string, unknown>)
        : entry,
    ]),
  );
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

export const getSettings = () => call<Settings>('get_settings');

export const setSettings = (settings: Settings) => call<Settings>('set_settings', { settings });

/* ---- Vault-class — §6.3 -------------------------------------------------- */

export const unlock = (path: string, password: string) => call<void>('unlock', { path, password });

export const lock = () => call<void>('lock');

export const listItems = () => call<ItemSummary[]>('list_items');

export const getItem = (itemId: string) => call<ItemDetail>('get_item', { itemId });

/**
 * Copies a field to the clipboard. **Returns no value** — that absence is R-10.
 *
 * The secret is written to the clipboard by Rust and never enters this heap.
 */
export const copyField = (itemId: string, fieldId: string) =>
  call<{ clearsAt: number }>('copy_field', { itemId, fieldId });

/* ---- Sanctioned — §7 ----------------------------------------------------- */
/* Exactly three. A fourth entry in this group is a decision, not a patch.     */

export const createVault = (name: string, path: string, password: string, kdf: KdfSummary) =>
  call<{ recoveryCode: string }>('create_vault', { name, path, password, kdf });

export const unlockRecoveryKit = (path: string, code: string) =>
  call<{ recoveryCode: string }>('unlock_recovery_kit', { path, code });

export const revealField = (itemId: string, fieldId: string) =>
  call<{ value: string; remaskAt: number }>('reveal_field', { itemId, fieldId });

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
