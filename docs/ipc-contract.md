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
  three times in this document, and the audit harness counts them.

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

There are **three** sanctioned commands and there will not be a fourth without a decision log entry:
`create_vault`, `unlock_recovery_kit`, and `reveal_field`. Adding one is the change this whole
document exists to make expensive.

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
      | "locked" | "no_such_item" | "no_such_field" | "not_secret" | "clipboard" | "io" | "internal";
  message: string;   // already localized for display; never contains a secret or a field value
};
```

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
  (`x-kde-passwordManagerHint`) are advisory. Whether the UI may say "Clears in 12s" or must say
  something weaker is the open question against the Phase 2 gate. The contract is unaffected either
  way; only the copy is.

```ts
lock(): void
```

Zeroizes the master key, drops the body, emits `vault-locked`. Idempotent: locking a locked vault
succeeds and does nothing, because the failure mode of a lock command that errors is a user
hammering it during a panic.

```ts
get_settings(): Settings
set_settings(patch: Partial<Settings>): Settings
```

```ts
type Settings = {
  theme: "system" | "light" | "dark";     // R-28
  auto_lock_seconds: number;              // R-09
  clipboard_clear_seconds: number;        // R-14
  audit_log_enabled: boolean;             // R-13, default false — D-31
};
```

> **TBD — open question.** *Where* settings are stored is not decided. `theme` must be readable
> while locked, so it cannot live in the sealed body; `audit_log_enabled` arguably should. None of
> the four values is secret, which points at one plain config file under the OS config directory,
> but that has not been decided and is not decided here. The command shape above does not depend on
> the answer, which is why the contract can be written now. Resolve before the first settings
> command is written.

## 7. Sanctioned commands

The three commands that may carry plaintext. Each returns exactly one `Secret`.

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
3. Only the three sanctioned commands produce a response containing a `Secret` at all.
4. `copy_field`'s response, and every event payload, contain no value from the vault.
5. A scripted session driving the whole shell — create, lock, unlock, list, select, reveal, copy —
   produces a log whose only secret values are the ones explicitly revealed.
6. Every vault-class and sanctioned command returns `locked` when the vault is locked, exercised by
   calling all of them against a locked core.

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
| 4 — `copy_field` and events carry no value | **shape-level only.** `copy_field` writes to the real system clipboard, which a CI runner may not have, so the command body is not driven. `Copied` has one field and it is an integer |
| 5 — scripted whole-shell session | **not automated.** It needs the shell, which does not exist yet. This is Phase 2 gate evidence and it is not yet produced |
| 6 — every vault command refuses while locked | automated |
| N-07 — no wildcard origin in the CSP | automated |

> **Finding, not yet resolved.** R-10's acceptance criterion reads "enforced by a **core** test", but
> `trustvault-core` must not depend on Tauri (N-02) and therefore cannot see a command at all. The
> enforceable version of that criterion is this harness, in `src-tauri`. Either R-10's acceptance
> text is corrected to say so, or the criterion stays unmeetable as written.

## 10. What later phases add

Recorded now so the extension points are honest about what they can absorb.

- **Phase 3** adds mutation (`add_item`, `update_item`, `delete_item`), the generator, TOTP, tags,
  and multi-vault. `generate_password` returns a `Secret` and would be a **fourth** sanctioned
  command — which is a decision log entry, per §2.1, not a patch. The alternative shape, generating
  into the clipboard only, is worth considering first.
- **Phase 4** adds Watchtower. Nothing it returns is a secret: a breach check sends a 5-character
  SHA-1 prefix and receives a list, and the scoring happens in Rust. If any Watchtower command ends
  up wanting a plaintext password in the webview, the design is wrong.
- **TOTP** returns a 6-digit code, which is a secret with a 30-second lifetime. It is not covered
  here and needs its own decision: a code that is worthless in 30 seconds may not warrant the
  sanctioned-command budget, but "it expires soon" is the argument that ends with secrets in lists.
