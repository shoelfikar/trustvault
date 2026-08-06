# The IPC contract, version 1

This document specifies every command and event crossing the boundary between `trustvault-core`
(which owns plaintext) and the webview (which is not trusted with it). It is written **before** the
command layer exists, for the same reason `docs/vault-format.md` was written before any
cryptography: a contract written afterwards documents whatever was built instead of constraining it.

Where this document and `src-tauri/src/commands/` disagree, **the code is wrong** — unless the
disagreement is with `trustvault-requirements.md` § *Interface contract*, which outranks both.

`src-tauri/tests/ipc_audit.rs` is the enforcement. Every rule below stated as MUST is a rule that
harness checks; a rule nobody can check is a comment, and it is marked as one.

Requirements referenced here live in `trustvault-requirements.md`. Decisions referenced as `D-nn`
live in the decision log in `trustvault-state.md`.

## 1. Notation

- Payload types are written in TypeScript syntax because the webview's half of the boundary is
  TypeScript. `number` is always an integer unless stated; there are no floats in this contract.
- `Millis` is Unix milliseconds UTC, matching `created_at`/`updated_at` in the vault body.
- `Uuid` is the lowercase hyphenated form, as `serde` writes it.
- **`Secret`** is a marker, not a type: it means *this string is plaintext secret material and this
  is the one place in the response it may appear*. It is `string` on the wire. It appears exactly
  **four** times in this document — three since Phase 2, and a fourth added by D-44 — and the audit
  harness counts them.
- **Argument names are `snake_case` on the wire**, exactly as printed below, and `src/lib/ipc.ts`
  converts to them on the way out. This is not cosmetic and it is not free: Tauri v2's
  `#[tauri::command]` renames arguments to **camelCase** by default and looks the resulting key up
  exactly, with no fallback, so every command carries
  `#[tauri::command(rename_all = "snake_case")]` to make the host agree with this document. Found
  the hard way on 2026-08-06 — see D-47 — and now asserted by the harness on every command, not
  only the ones with a two-word argument today.
- **`// planned`** on a declaration means the command is specified here and **not yet registered**.
  It exists so this document can keep being written before the code, which is the practice that
  earned its keep twice (D-25, and four findings in Phase 2). It is not a comment: the harness
  reads it, and asserts both directions — a planned command that *is* registered fails the build
  just as an unplanned command that is not. So implementing one means deleting its marker in the
  same commit, and the marker cannot be used to park a command that quietly shipped.

## 2. The rule everything follows

The webview's heap cannot be wiped (`tauri-apps/tauri` discussion #10852). JavaScript strings are
immutable and garbage-collected: once a secret is in the webview, nothing in this codebase can
remove it, and it stays until the process exits. Every rule below is a consequence.

1. **At most one `Secret` per invocation** — R-10. Not one per item, not one per field batch: one
   per command return.
2. **Only on explicit user action.** No command returning a `Secret` may be called on render, on
   hover, on focus, on selection, or on any timer. The three that can are wired to a click or a
   keystroke and nothing else.
3. **Lists elide.** `list_items` and `get_item` carry metadata and masks, never values — §6.
4. **`copy_field` never returns the value at all.** Rust owns the clipboard write. The secret does
   not enter JS in any form, which is the only version of "copy a password" that is actually safe
   on this platform.
5. **The core's lock state is authoritative.** Every command that touches vault state re-checks it.
   The frontend is told to clear its stores on lock and is never trusted to have done so.

### 2.1 Three classes of command

Every command belongs to exactly one, and the class is part of the contract:

| Class | May touch the vault | May return a `Secret` | Requires unlocked |
|-------|--------------------|-----------------------|-------------------|
| **Ambient** | no | no | no |
| **Vault** | yes | no | yes |
| **Sanctioned** | yes | **yes, exactly one** | yes |

There are **four** sanctioned commands and there will not be a fifth without a decision log entry:
`create_vault`, `unlock_recovery_kit`, `reveal_field`, and — since **D-44** — `generate_password`.
Adding one is the change this whole document exists to make expensive, and D-44 is what paying that
price looks like: the argument for the fourth is in the decision log with the alternative it beat,
not in a commit message.

### 2.2 Locking is not a class

Two commands are callable in **any** lock state, and the exception is principled rather than
convenient: `lock` and `switch_vault` only ever *remove* plaintext from memory. A command that
cannot disclose anything cannot be made safer by refusing to run, and a lock command that errors
is a user hammering a button during a panic. Both are idempotent.

This is the complete list. Every other vault-class and sanctioned command refuses while locked, and
`tests/ipc_audit.rs` check 6 exercises exactly the complement of these two — an exemption list that
lives in the harness and is justified here, so it cannot grow quietly.

## 3. Naming and shape

- Commands are `snake_case`, named verb-first: `reveal_field`, not `field_reveal` or `getField`.
- Arguments are always a single named object, never positional. A boolean in position three is how
  `copy_field(id, field, true)` eventually means something nobody remembers.
- A command that changes state returns `void` unless the caller genuinely cannot proceed without a
  value. `unlock` returns nothing: the frontend learns the vault is open from `vault_status`, which
  keeps one source of truth for lock state instead of two.

## 4. Errors

The error payload is:

```ts
type IpcError = {
  kind: "not_a_vault" | "unsupported_version" | "unreadable" | "malformed_recovery_code"
      | "locked" | "no_such_item" | "no_such_field" | "not_secret" | "clipboard" | "io" | "internal"
      // Phase 3
      | "malformed_totp_secret" | "confirmation_mismatch" | "not_importable";
  message: string;   // already localized for display; never contains a secret or a field value
};
```

The three Phase 3 kinds are all safe to distinguish, and each for the same reason
`malformed_recovery_code` is: they are decided **before** any key material or vault content is
involved. `malformed_totp_secret` is a seed the user is typing that will not base32-decode;
`confirmation_mismatch` is a name typed into a delete dialog, compared against a display name the
UI is already showing; `not_importable` is a file that is not a Bitwarden JSON export. None of them
tells the caller anything about a vault it could not open.

`not_importable` is deliberately coarse — malformed JSON, wrong schema, and an encrypted export all
return it. The `message` may say which, because an import is a file the user chose and none of it is
ours to protect; this is the opposite of `unreadable`, and the difference is that nothing here is
guarding a password guess.

Three rules, all of them load-bearing:

**`unreadable` is deliberately ambiguous** — R-03. It means *wrong password* or *corrupt file* and
the caller cannot tell which. `trustvault_core::Error::Unreadable` maps to it unchanged. Do not
improve this message. Do not add a `wrong_password` kind. The core spends equal work on both paths
(`docs/vault-format.md` §7, steps 6–7) and a helpful error at this layer throws that away for free.

**The command layer must not reintroduce a timing difference the core removed.** No early return
before calling the core on any unlock path: not a "file doesn't exist" check, not a length check on
the password, not a cached-failure short-circuit. Read the file and call the core, always. This is
the one rule in this document that a reviewer must check by reading, because a harness cannot see an
`if` that was never written.

**`malformed_recovery_code` is safe to distinguish**, and only because of when it happens: it is a
transcription error caught before any key material exists (`trustvault_core::Error::
MalformedRecoveryCode`), so reporting it leaks nothing about the vault. A *correctly formed* code
that fails to open the vault returns `unreadable` like everything else.

`internal` covers `Encode`, `Entropy`, and `KdfParams` — all three are bugs or a broken machine, not
user errors, and none of them should reach a user with a distinguishing message.

## 5. Ambient commands

Callable with no vault, and before unlock. None of them touches vault state.

```ts
build_info(): { version: string; format_version: number; extension: string }
```
Already exists from Phase 0. Carries no vault state, which is why it was safe to ship before the
boundary did.

```ts
vault_status(): {
  state: "no_vault" | "locked" | "unlocked";
  path: string | null;         // absolute path of the vault file
  display_name: string;        // see below
  item_count: number | null;   // null unless unlocked
}
```

`display_name` has a wrinkle worth stating rather than discovering on the lock screen: **the vault's
real name lives inside the sealed body**, so while the vault is locked it cannot be read. When
`state` is `locked`, `display_name` is the file stem — `personal.tvault` shows as "personal". When
`unlocked` it is `Vault::name()`. The lock screen must not imply it is showing the name the user
typed at onboarding, because until they unlock, it isn't.

```ts
default_vault_path({ name: string }): string
```

Where a vault called `name` would go if the user does not say otherwise — R-08.

**Not a native file picker.** A picker means `tauri-plugin-dialog`, and a plugin is widened attack
surface in a process holding decrypted secrets; the manifest's standing rule is that a plugin
arrives when a requirement needs one and not before. A resolved default plus an editable path
satisfies "name & location". Revisit with a decision log entry if the typed path turns out to be
what users get wrong.

```ts
score_password({ password: string; inputs: string[] }): { score: 0|1|2|3|4; label: string; crack_time: string }
```

Scores a password with zxcvbn (D-12) for onboarding's strength meter — R-08. Ambient because it
needs no vault: it scores a password that does not exist yet.

The password crosses **inbound**, which deserves stating rather than glossing. That direction is not
the one this contract defends: the user typed it into the webview, so it is already in a heap that
cannot be wiped, and nothing here changes that. What matters is that it is never sent back, never
stored, and never logged. The frontend debounces rather than scoring every keystroke — not for
safety, which debouncing does not buy, but because the KDF-adjacent work is not free.

```ts
calibrate_kdf(): KdfSummary
```
Wraps `KdfParams::calibrate`, for onboarding step 2 — R-02. Takes seconds and holds the thread; the
UI shows progress. The result is written into the header of the vault about to be created, not into
a setting.

The measured wall-clock time is deliberately **not** in the return, because
`KdfParams::calibrate() -> Self` does not currently expose it. If the onboarding screen wants to
show "unlock will take ~510 ms on this machine" — which is a good thing to tell someone choosing a
password — that is a small addition to the core, not something to reconstruct by timing the command
from JS. A timing measured across the IPC boundary measures the IPC boundary.

```ts
copy_generated({ password: string }): { clears_at: Millis }   // planned
```

Writes a **not-yet-stored** password to the clipboard and schedules the same clear `copy_field`
does — the generator's copy button, and the reason D-37's two closed copy paths can open in this
phase. Ambient because there is nothing to look up: the caller already holds the value.

That is also the whole of its safety argument, and it is worth stating plainly because the command
looks alarming. It takes a secret **inbound** and returns none; the string it copies is one the
webview already has, minted by `generate_password` moments earlier or typed by the user. It reads
nothing, so it cannot disclose anything the caller did not supply. What it adds is the one thing the
webview cannot do for itself: a clear scheduled in Rust, which is exactly what D-37 said was missing
when it closed these paths.

Constraints:

- It MUST NOT log, store, or retain the value beyond the clipboard write. There is no host-side
  "last generated password", because a host-side copy of a secret with no vault around it is a
  lock-state hole with no lock.
- The clear interval is `clipboard_clear_seconds` from settings, the same one `copy_field` uses. Two
  clipboard timers with different durations is how one of them ends up wrong.

```ts
totp_preview({ secret: string }): { code: string; expires_at: Millis; period: number; digits: number }   // planned
```

One code from a seed the user is **currently typing** into the Add dialog — the live preview R-20's
surface needs. Ambient because it needs no vault: the item does not exist yet.

It doubles as the seed's validator, which is the point. A base32 seed that will not decode is caught
while the user is looking at the field, not discovered a month later at a login prompt with the
phone already wiped. A malformed seed returns `malformed_totp_secret` (§4).

`code` is not marked `Secret` — see **D-45**, and the reasoning is in §7.1 rather than here so that
it sits next to the commands it is an exception to.

## 6. Vault commands

All require `state == "unlocked"` and return `locked` otherwise, re-checked in the core on every
call. None may return a `Secret`.

### 6.1 Elision

This is the shape that keeps secrets out of the list, so it is specified exactly.

```ts
type FieldSummary = {
  id: Uuid;
  label: string;
  kind: "text" | "username" | "password" | "url" | "email" | "otp" | "note" | "date";
  secret: boolean;
  value: string | null;   // the real value if `secret` is false; null if `secret` is true
  mask: string | null;    // null if `secret` is false; otherwise §6.2
  custom: boolean;        // user- or import-added, rather than one of the type's own — D-43
};

type ItemSummary = {
  id: Uuid;
  kind: "login" | "api_key" | "card" | "note" | "wifi" | "ssh_key" | "identity";
  title: string;
  tags: string[];
  status: "unknown" | "strong" | "weak" | "reused" | "breached" | "expired";
  favourite: boolean;
  created_at: Millis;
  updated_at: Millis;
};

type ItemDetail = ItemSummary & { fields: FieldSummary[] };
```

A non-secret field's value crosses freely, and that is correct rather than a compromise: `secret` is
**stored, never inferred** (`docs/vault-format.md` §6.3), so a field whose `secret` is false is one
the user declared is not a secret. The username on a login row is metadata the design draws in the
list, and treating it as a secret would mean the list could not be rendered at all.

`history` is **not** in either shape and there is no command that returns it in Phase 2. It holds
previous values of secret fields, which makes it as sensitive as the fields themselves — a list of a
user's last five passwords for one site is worse than any single one of them. When the history UI
arrives it reveals one entry at a time through a sanctioned command, or it does not arrive.

### 6.2 The mask is a fixed width, not the real length

`mask` is **12 bullet characters, always**, regardless of the secret's true length.

The obvious implementation returns `"•".repeat(value.len())` so the dots look right. That puts the
exact length of every password in the vault into a heap that cannot be wiped, for every item in the
list, without anyone revealing anything. Password length is not catastrophic on its own — but it is
the one piece of a secret that is free to leak by accident, it narrows a search space, and it buys
nothing except dots of a pleasing width. A fixed 12 is a deliberate lie, and the UI must not present
it as a length. — **D-32**

### 6.3 The commands

```ts
list_items(): ItemSummary[]
get_item({ item_id: Uuid }): ItemDetail
```

```ts
copy_field({ item_id: Uuid; field_id: Uuid }): { clears_at: Millis }
```

Writes the field's value to the system clipboard **from Rust** and schedules the clear — R-10, R-14.
The value is not in the return, is not in an event, and is not in the error. `clears_at` is the
wall-clock instant the clear is scheduled for, so the UI can draw its countdown without owning the
timer; the countdown is cosmetic and the clear happens whether or not the webview is alive to see
it.

Two honesty constraints on this command, both from the prior-art survey:

- Clipboard **ownership** on X11 and Wayland belongs to the copying process. If TrustVault exits
  before the timer fires, the clipboard content goes with it — which is safe, but means the UI must
  not claim the clear is guaranteed after a quit.
- Clipboard **managers** (GPaste, Klipper, CopyQ) keep their own copy and the platform hints
  (`x-kde-passwordManagerHint`) are advisory. **Measured 2026-08-05** against GPaste 45.3 on
  GNOME/Wayland with `track-changes` on: it recorded the value *with* the hint set, and still held
  it after `clipboard::clear()` ran — `src-tauri/tests/clipboard_manager.rs`. The contract is
  unaffected, as expected; the UI copy is not, and now says TrustVault clears **its own** copy.

### 6.4 Mutation — R-17, R-18

```ts
type NewField = {
  label: string;
  kind: FieldSummary["kind"];
  value: string;          // inbound plaintext; the user typed it
  secret: boolean;
  custom: boolean;        // D-43
};

add_item({ kind: ItemSummary["kind"]; title: string; tags: string[]; fields: NewField[] }): { item_id: Uuid }
```

Returns the identifier and nothing else — the caller re-reads through `get_item`, which keeps one
elision path instead of two. Saves the vault before returning, so "it is in the vault" and "the
command succeeded" are the same event; an add that lived only in memory until some later save is how
a crash loses the item the user just carefully typed.

```ts
type EditField = {
  id: Uuid | null;        // null creates; an existing id edits
  label: string;
  kind: FieldSummary["kind"];
  value: string | null;   // null means UNCHANGED — see below
  secret: boolean;
  custom: boolean;
};

update_item({ item_id: Uuid; title: string; tags: string[]; favourite: boolean; fields: EditField[] }): void
delete_item({ item_id: Uuid }): void
```

**`value: null` means "leave it alone", and it is the single most load-bearing detail in this
section.** The edit form never received the secret values — §6.1 elides them, which is the whole
architecture. So a form that sends back what it is holding is sending back masks. Without this rule,
renaming an item would overwrite every password in it with `"••••••••••••"`, and the previous values
would land in `history` where no v1 surface can reach them. A field the user did not touch carries
`null`, and the core keeps what it has.

Two more rules with the same shape:

- **Omission deletes.** A field whose `id` is absent from `fields` is removed. This is stated
  because the alternative — a separate `remove_field` command — makes an edit two round trips that
  can half-succeed.
- **The list is the order.** `fields` is written in the order given, so drag-to-reorder needs no
  command of its own.

`delete_item` is guarded by the dialog R-18 requires, and that guard is in the **UI**, not here.
Stated rather than left implicit: a confirmation the host does not verify is a confirmation, because
the thing it protects against is a mis-click, not a hostile caller — the caller is the only user.
`delete_vault` is the opposite case and §6.7 says why.

### 6.5 Search and tags — R-16

```ts
search_items({ query: string; limit: number }): ItemSummary[]   // planned
list_tags(): { tag: string; count: number }[]   // planned
```

**The matching runs in Rust — D-46.** R-16 asks the palette to search titles, usernames, URLs and
tags, and only two of those four are in `ItemSummary`. The obvious way to get the other two is to
put every username and URL in the vault into the webview so JavaScript can filter them; that is a
permitted crossing under §6.1, and it is still the wrong trade. It would place the entire
identifying surface of the vault into a heap that cannot be wiped, on every list render, for a
feature used a few times a day — to save an IPC round trip on a budget (S-04, ≤ 50 ms p95) that has
room for one.

So the query crosses inbound, the matching happens against plaintext that never leaves the core, and
what comes back is the same elided summary the list already gets. **Results carry no field values**,
including the value that matched — if the design turns out to need the matched username shown under
the row, that is a bounded addition covering the visible results only, and it gets its own decision
rather than arriving as a widened return type.

`list_tags` counts across the vault, for the sidebar's tag list and the Add dialog's chips. A tag is
metadata: it is drawn in the item list already.

### 6.6 TOTP — R-20

```ts
totp_code({ item_id: Uuid }): { code: string; expires_at: Millis; period: number; digits: number }   // planned
```

The current code for the item's `otp` field. Rules, each of which is a way this command could go
wrong:

- **The seed never crosses.** `totp_code` returns a code; the seed is a field with `secret: true`
  and it comes out, if ever, through `reveal_field` like any other secret. A command that returned
  both would be a sanctioned command pretending not to be one.
- **One item, the selected one.** It MUST NOT be batched, and no code appears in `list_items`. A
  list of live codes is a list of secrets refreshed on a timer, which is the shape §2 exists to
  prevent.
- `no_such_field` when the item has no `otp` field, so the detail pane's ring is drawn from a
  successful call rather than from a guess about the item type.

### 6.7 Vaults — R-22, R-18

```ts
type VaultRef = { path: string; display_name: string; last_opened_at: Millis | null };

list_vaults(): VaultRef[]   // planned
switch_vault({ path: string }): void   // planned
forget_vault({ path: string }): void   // planned
delete_vault({ path: string; confirm_name: string }): void   // planned
```

`display_name` carries the §5 wrinkle unchanged and it bites harder here: the real name is inside
the sealed body, so **every vault in this list except the open one shows its file stem**. The
switcher must not imply otherwise.

`switch_vault` **locks and zeroizes the outgoing vault first**, then points at the new path, and
leaves the state `locked` — it does not and cannot unlock, because it is given no password. Callable
in any lock state (§2.2). R-22's acceptance criterion is exactly the first half of that sentence.

`forget_vault` removes the entry from the list. **The file is untouched** — this is the "Leave
vault" flow, and confusing it with the next one would be the worst bug in the application.

`delete_vault` erases the file, and the confirmation is enforced **here, in Rust**: `confirm_name`
must equal the `display_name` this command would report for that path, or it returns
`confirmation_mismatch` and deletes nothing. This is the one confirmation the host verifies rather
than trusting the UI with, and the asymmetry with `delete_item` above is deliberate — a wrong
`delete_item` costs one entry that `history` may still hold, and a wrong `delete_vault` costs
everything, with no undo anywhere in the product. R-18 asks for the typed name; a typed name checked
only in JavaScript is checked by the layer this document does not trust.

Deleting the **open** vault locks it first, in that order, so the key is zeroized before the bytes go.

### 6.8 Import — R-29, D-42

```ts
type Refusal   = { item_title: string; field: string; reason: string };
type Converted = { item_title: string; field: string; note: string };

type ImportReport = {
  total: number;
  per_kind: { kind: ItemSummary["kind"]; count: number }[];
  tags_created: string[];
  tags_merged: string[];
  converted: Converted[];
  refusals: Refusal[];
};

import_preview({ path: string }): ImportReport
import_commit({ path: string }): ImportReport
```

R-29 is met only when **every field is either mapped or named in a refusal** — the documented
failure of every importer surveyed for D-42 is a field dropped in silence, so the report is the
requirement and not a courtesy. Three outcomes, not two:

| Outcome | Meaning |
|---------|---------|
| **mapped** | Landed in a field of the target item, unchanged. Counted in `per_kind`, listed nowhere. |
| **converted** | Landed, but with a shape change worth telling the user about — a Bitwarden boolean custom field stored as the text `"true"`. In `converted`. |
| **refused** | Has no home. In `refusals`, with the reason. |

**Neither a `Refusal` nor a `Converted` may carry a field's value** — the title and the label are
metadata that already cross in the item list; the value is the thing the vault exists to hold. A
report that quoted the values it could not import would be a plaintext dump of the parts of the
foreign vault we understood least.

`import_preview` commits nothing. `import_commit` is **one transaction**: it either lands whole or
leaves the vault untouched, so a malformed entry two thirds of the way through a 400-item export
does not produce a half-imported vault nobody can reason about.

`import_commit` **re-reads and re-parses the file** rather than holding the preview's result in host
memory. Holding it would keep a full plaintext copy of a foreign vault alive in the host for as long
as the user reads the preview — outside the vault, and so outside everything that locks. The cost is
one extra file read and a real race: a file edited between the two calls imports as it is at commit,
not as previewed. That is why `import_commit` returns a report too, and why the report shown *after*
an import is the authoritative one.

The file is read once per call into zeroized buffers, and **TrustVault never writes a copy of it
anywhere** — no backup, no temp file, no log line. R-29 says so and it is the easiest half of the
requirement to lose to a debugging aid.

```ts
lock(): void
```

Zeroizes the master key, drops the body, emits `vault-locked`. Idempotent: locking a locked vault
succeeds and does nothing, because the failure mode of a lock command that errors is a user
hammering it during a panic.

```ts
get_settings(): Settings
set_settings({ settings: Settings }): Settings
```

```ts
type Settings = {
  theme: "system" | "light" | "dark";     // R-28
  auto_lock_seconds: number;              // R-09
  clipboard_clear_seconds: number;        // R-14
  audit_log_enabled: boolean;             // R-13, default false — D-31
  sidebar_width: number;                  // px, clamped 180–320 — MASTER.md §4
  list_width: number;                     // px, clamped 240–460
  last_vault_path: string | null;         // host-owned, read-only to the webview — D-40
  ui_scale: "compact" | "default" | "large";   // R-21; 92 % / 100 % / 115 %
  launch_at_login: boolean;               // R-21
  window_width: number;                   // px — R-27
  window_height: number;                  // px — R-27
  window_maximized: boolean;              // R-27
};
```

The five fields below `last_vault_path` are Phase 3's. Two of them are not merely stored:

- **`ui_scale` scales the root `rem`**, which `tokens.css` already implements through
  `[data-ui-scale]`; what was missing was somewhere to keep the choice. Because every token is
  derived from `rem`, this is the one setting that moves every measurement in the application at
  once, and `MASTER.md` §10 asks for it end to end.
- **`launch_at_login` is the only setting with an effect outside this process** — a desktop entry,
  a `LaunchAgent`, or a registry value, one per platform. It is a `boolean` here and three
  implementations behind that, and on a platform where the write fails it MUST report `io` and
  leave the stored value alone rather than showing a toggle that lies.

Window geometry lives here rather than in a separate window-state file, for D-40's reason
unchanged: one store, one format, one migration story.

**`known_vaults` is deliberately not in this struct.** It is host-owned bookkeeping in the same
settings file, and its read path is `list_vaults` (§6.7), which derives a `display_name` per entry —
work `get_settings` has no business doing and the webview must not do for itself.

`set_settings` takes the **whole struct**, not a patch: a patch shape needs every field optional,
and an optional boolean is how a setting gets silently reset by a caller that omitted it.

`last_vault_path` is the one field the webview may read and MUST NOT set. The host overwrites
whatever arrives in it with what it already had, because `Settings` deserializes with defaults —
a frontend that does not know the field sends it absent, which reads as `null`, which would erase
the user's vault on the next theme change. Storage is decided: plain JSON in the OS app-config
directory, all of it, per **D-33**.

## 7. Sanctioned commands

The four commands that may carry plaintext. Each returns exactly one `Secret`.

```ts
type KdfSummary = { m_cost: number; t_cost: number; p_cost: number };

create_vault({ name: string; path: string; password: string; kdf: KdfSummary }): { recovery_code: Secret }
```

The recovery code is a secret and it crosses IPC, because R-07 requires it to be shown to the user
exactly once and there is nowhere else it can be shown. This is the least avoidable secret in the
product: it must be read by a human, off a screen, and written down. Constraints:

- The webview MUST NOT store it in any reactive state, any store, or any variable that outlives the
  recovery-kit screen. It is rendered and dropped.
- Navigating away from step 3 is the last time it exists. There is no command to fetch it again;
  reissuing (`Vault::reissue_recovery_code`) invalidates the old one and is a Phase 3 surface.
- `password` crosses **inbound**, which is unavoidable — the user types it into the webview. Inbound
  is a different risk from outbound: it is already in the heap the moment the keystroke lands, and
  nothing this contract does changes that. The mitigation is that it is never sent back.

```ts
unlock({ path: string; password: string }): void
unlock_recovery_kit({ path: string; code: string }): { recovery_code: Secret }
```

`unlock` is not sanctioned — it takes a secret inbound and returns nothing.

`unlock_recovery_kit` is sanctioned for a reason that is easy to miss: unlocking with a recovery kit
means the kit has been used, and the design's recovery flow issues a fresh one on the spot. That
fresh code is a `Secret` outbound, under the same render-and-drop rule as `create_vault`.

```ts
reveal_field({ item_id: Uuid; field_id: Uuid }): { value: Secret; remask_at: Millis }
```

Returns one field's plaintext — R-12. Rules:

- The field's `secret` must be true. Revealing a non-secret field returns `not_secret` rather than
  succeeding, so that "reveal" always means the same thing in the audit log and in the UI.
- The core starts a 10-second timer and emits `field-remasked` when it expires. The **core** owns
  the timer, not the frontend: a frontend timer is cleared by a reload, and a reload must not extend
  a reveal.
- The frontend drops its copy on that event. It is trusted to do so and it cannot be verified —
  this is the one place in the contract where the security depends on the untrusted side behaving,
  which is exactly why the window is 10 seconds and not 60.
- Appends an audit entry when `audit_log_enabled` — R-13, D-31. The entry holds the timestamp, the
  item id, and the field id. It MUST NOT hold the value, the label, or the item title.

```ts
type CharSets = { lowercase: boolean; uppercase: boolean; digits: boolean; symbols: boolean };

type Generated = { password: Secret; score: 0|1|2|3|4; label: string; crack_time: string };

generate_password({ length: number; sets: CharSets; exclude_ambiguous: boolean }): Generated   // planned
```

**The fourth sanctioned command — D-44.** `length` is 8–64 and at least one set must be true, or the
command returns `internal`; every selected set appears in the output, which is what R-15's
acceptance criterion asks for. `exclude_ambiguous` drops `0 O 1 l I` and defaults on, per
`MASTER.md` §3.

Three things about this command are worth stating, because each is where the alternative was.

**Why it returns the password at all.** The alternative §10 named — generate straight into the
clipboard, never crossing IPC — was rejected on the surface the design actually draws: the generator
shows the value, with a regenerate button beside it, because the user is deciding whether to accept
this password. A generator whose output can only be pasted, never seen, makes the length slider and
the character-set chips into theatre, and it cannot fill the New-item dialog's password field at all.

**Why that is not a widening.** A password being generated is not yet a stored secret. It exists
nowhere but this response, it protects nothing yet, and if the user rejects it, it protected nothing
ever. The comparison that settles it: an item created by typing a password by hand puts exactly the
same string in exactly the same unwipeable heap, and no rule here has ever stopped that — §5 already
says inbound is a direction this contract does not defend. The marginal exposure of generating in
Rust instead of in JavaScript is zero, and what is bought with it is the next point.

**Why not keep minting it in the webview**, as D-37 does today with `crypto.getRandomValues`. That
was the right call for a dialog with no command behind it, and it stops being right the moment the
value can be saved. Randomness for stored credentials belongs to the one path R-06 and D-23 already
constrain — `getrandom`, called from a single file, with a CI grep enforcing that no seedable RNG
exists in the crate. Two generators, one of which is "the real one", is a distinction that survives
exactly as long as the person who remembers it.

The scoring rides along rather than taking a second call, and that is a safety property and not a
convenience: routing the generated value back through `score_password` would send it across the
boundary a second time, inbound, for a number the generator's own side already has the inputs for.

Under the render-and-drop rule, like the other three: the webview holds it for as long as the dialog
is open and drops it when the dialog closes. Copying it goes through `copy_generated` (§5), never
`navigator.clipboard` — that is D-37's constraint honoured rather than repealed, because what D-37
actually objected to was a copy nothing would ever clear.

### 7.1 A TOTP code is not a `Secret` — D-45

§10 left this open with the right warning attached: *"it expires soon" is the argument that ends
with secrets in lists*. So the decision is recorded here, next to the commands it is an exception
to, rather than in the section that raised it.

A TOTP code is **not** marked `Secret`, and the line is drawn at the seed instead. The reasoning is
not that the code is short-lived — that argument would also license returning a password about to
be rotated:

- **It is not the credential.** The seed is. A code cannot be run backwards to the seed, so a code
  in the webview heap forever discloses one 30-second window that has already passed.
- **It is already published by design.** The protocol's own operation is to type the code into a
  remote party's form, over the network. A secret whose intended use is transmission to a third
  party is not the kind of secret §2 was written for.
- **It is single-use and self-invalidating**, which a password is not.

What the exception does **not** license, stated so it cannot be read as broader than it is:
`totp_code` returns codes for **one item at a time, the selected one** — no batching, and none in
any list (§6.6). The seed itself stays a `secret: true` field, elided in `get_item` and revealable
only through `reveal_field`. If a future surface wants codes for many items at once, it is a new
decision, and this one does not cover it.

## 8. Events

Core → webview, one direction. An event MUST NOT carry a `Secret`; there is no user action behind
an event, so rule 2 of §2 could not be satisfied by one.

```ts
"vault-locked":    { reason: "manual" | "timeout" | "os_sleep" }   // R-09
"field-remasked":  { item_id: Uuid; field_id: Uuid }               // R-12
"clipboard-cleared": { item_id: Uuid; field_id: Uuid }             // R-14
```

`vault-locked` carries its reason so the lock screen can say why, which is the difference between a
user thinking the app crashed and a user knowing the timeout fired.

## 9. What the audit harness checks

`src-tauri/tests/ipc_audit.rs`, against an instrumented build that logs every value crossing the
boundary in both directions:

1. Every registered command appears in this document, and every command in this document is
   registered. A command that exists but is undocumented is the failure mode this check exists for.
2. No response contains more than one `Secret` — R-10.
3. Only the **four** sanctioned commands produce a response containing a `Secret` at all — the
   count is asserted against a constant, so D-44's fourth had to change a number a reviewer sees.
4. `copy_field`'s response, and every event payload, contain no value from the vault. From Phase 3
   this extends to `copy_generated` — which takes a secret inbound and must not echo it, including
   into an error `message` — and to `import_preview`/`import_commit`, whose reports name fields and
   never quote them (§6.8).
5. A scripted session driving the whole shell — create, lock, unlock, list, select, reveal, copy —
   produces a log whose only secret values are the ones explicitly revealed.
6. Every vault-class and sanctioned command returns `locked` when the vault is locked, exercised by
   calling all of them against a locked core — **except `lock` and `switch_vault`**, whose exemption
   is §2.2 and whose list the harness holds explicitly so that adding a third requires editing it.
7. *(Phase 3)* `totp_code` and `search_items` return no field value: the first returns a code and
   never a seed (§6.6), the second returns summaries and never the text that matched (§6.5). Both
   are the shapes D-45 and D-46 chose over an easier one, so both are pinned rather than trusted.

Check 6 is worth more than it looks. It is the regression test for a webview reload: reload leaves
the frontend's stores empty and its lock state whatever the core says, and the bug it prevents is a
command that reads a cached handle instead of re-checking.

The harness calls the **real command bodies**, through the `_inner` function each command wraps. A
harness that reimplemented the boundary would prove only that the reimplementation is safe.

**What is automated and what is not**, stated here rather than implied, because a check that quietly
does not run is worse than one documented as not running:

| Check | Status |
|-------|--------|
| 1 — registered set == documented set | automated; parses `generate_handler!` and this document |
| 2 — at most one `Secret` per response | automated for `list_items`, `get_item`, `reveal_field` |
| 3 — only the three sanctioned commands | automated |
| 4 — `copy_field` and events carry no value | automated in `tests/ipc_session.rs`, which drives the real body and records **whichever** outcome the machine gives it. A runner with no clipboard produces an error payload, and an error payload is asserted against too — composing a message out of the failing value is a classic way to leak it |
| 5 — scripted whole-shell session | automated in `tests/ipc_session.rs`. It scripts launch → onboarding → quit → relaunch → wrong password → unlock → list → open → reveal → copy → lock → recovery unlock, records every crossing, and reads the transcript. Two limits, named: it drives command bodies rather than a live webview, and the item it reveals is seeded through the core's API because Phase 2 ships no mutation command (D-38). The transcript is written to `target/ipc-session.log` as gate evidence |
| 6 — every vault command refuses while locked | automated |
| N-07 — no wildcard origin in the CSP | automated |

> **Resolved 2026-08-05 — D-39.** R-10's acceptance criterion read "enforced by a **core** test",
> which `trustvault-core` cannot do: N-02 forbids it from depending on Tauri, so it cannot see a
> command at all. `trustvault-requirements.md` now names `src-tauri/tests/ipc_audit.rs` instead,
> and says why, so the correction cannot be read later as a weakening.

## 10. What later phases add

Recorded now so the extension points are honest about what they can absorb.

- ~~**Phase 3** adds mutation, the generator, TOTP, tags, and multi-vault.~~ **Written into the
  sections above on 2026-08-05, before the code**, which is the whole reason this document exists;
  Phase 1 and Phase 2 both had a contract written first falsified within the hour, and both times
  that was cheaper than the bug. The three questions this section left open are now answered where
  they belong: `generate_password` is the fourth sanctioned command (**D-44**, §7), a TOTP code is
  not a `Secret` (**D-45**, §7.1), and palette search matches in Rust (**D-46**, §6.5). The import
  path is §6.8.
- **Phase 4** adds Watchtower. Nothing it returns is a secret: a breach check sends a 5-character
  SHA-1 prefix and receives a list, and the scoring happens in Rust. If any Watchtower command ends
  up wanting a plaintext password in the webview, the design is wrong.
- **Item history** still has no command, and §6.1 still says what it would take: one entry at a
  time through a sanctioned command, or it does not arrive. Phase 3's `update_item` writes to
  `history` (a changed value pushes the old one) without any way to read it back, which is the
  correct asymmetry and not an oversight.
