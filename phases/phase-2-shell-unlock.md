# Phase 2 — Shell & unlock

> **Provisional.** Written at kickoff as a forecast, not a plan. Run the entry check below before
> the first task, then delete this notice. — *Delete these two lines once the phase is current.*

Status: not started
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

The Tauri command layer implementing the interface contract, then the first real screens: the three
onboarding steps, the lock screen, the recovery flow, the three-pane shell with sidebar and item
list, and the detail pane with per-field reveal and copy. Requirements covered: R-07…R-14, R-28,
N-07, N-08.

The security boundary is built here or it is never built. Every later phase adds surfaces on top of
this contract.

## Entry check — before the first task

- [ ] **Still in scope** — traceable to `trustvault-project.md`, nothing drifted into out-of-scope
- [ ] **Requirements still live** — R-07…R-14, R-28, N-07, N-08
- [ ] **Dependencies passed their gates** — read the gates table in `trustvault-state.md`, not the
      ticked boxes. Depends on: Phase 1 gate
- [ ] **External dependencies met** — the audit-log open question is resolved (it changes R-13's
      storage location, and retrofitting that means re-encrypting)
- [ ] **Task list re-checked** against what Phase 1 actually taught
- [ ] **Exit gate still measurable** as written

Entry check completed: —

## Exit gate — G-B′

- [ ] *Functional:* create a vault through onboarding, quit, relaunch, unlock, read an item — on
      Linux, against a real file on disk
- [ ] Auto-lock fires on timeout, on demand, and on OS sleep — R-09
- [ ] Clipboard clears within the configured window + 200 ms — S-11, R-14
- [ ] *Security:* an instrumented build logs every value crossing IPC; a scripted session exercising
      the whole shell produces a log with no secret value except explicitly revealed ones, never more
      than one per command, and `copy_field` never returns a value at all — R-10
- [ ] The clipboard-manager open question is answered with a real test against GPaste or Klipper, and
      the UI copy matches the truth

Ticking this box is the same event as ticking G-B′ in `trustvault-state.md`.

## Tasks

### IPC boundary

- [ ] Tauri command layer in `src-tauri/src/commands/`, wrapping `trustvault-core` — the core stays
      unaware of Tauri
- [ ] `list_items` returns metadata with secrets **elided**, never values
- [ ] `reveal_field` returns one value and starts a 10 s core-side timer, then emits
      `field-remasked` — R-12
- [ ] `copy_field` writes to the clipboard from Rust and returns nothing — R-10
- [ ] Lock state is authoritative in the core; every command re-checks it, and a webview reload
      cannot unlock anything
- [ ] IPC audit harness: an instrumented build that logs every value crossing the boundary, used by
      the gate and kept for regression
- [ ] Tauri CSP with no wildcard origins — N-07

### Onboarding & lock

- [ ] Step 1 — vault name, file, location — R-08
- [ ] Step 2 — master password with live strength meter — R-08
- [ ] Step 3 — recovery kit, shown once, printable — R-07, R-08
- [ ] Lock screen with the unlock scrim transition (`MASTER.md` §5 — the app's only expressive
      moment)
- [ ] Recovery flow: six 4-character groups, and the password alternative — R-07
- [ ] Auto-lock on idle timeout, on demand, and on OS sleep — R-09
- [ ] Audit log written on every reveal, containing no secret values — R-13

### Shell

- [ ] Three-pane layout: 232px sidebar, 300px list, flex detail; resizable and persisted
- [ ] Titlebar: vault name, search affordance, settings, lock — native decorations
- [ ] Sidebar: vault nav with counts, tags, Watchtower entry with badge, profile footer
- [ ] Item list with the 34px row, selection bar, and hover states from `MASTER.md` §7 — R-11
- [ ] Detail pane: title, type icon, status chip, field rows
- [ ] Secret field: masked by default, reveal with auto-remask, copy — R-12, R-14
- [ ] Empty states for every list — R-19 groundwork
- [ ] Theme follows the OS with a manual override — R-28
- [ ] `prefers-reduced-motion` honoured throughout — N-08

Total: 0/22.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Tauri command layer | `src-tauri/src/commands/` |
| IPC audit harness | `src-tauri/tests/ipc_audit.rs` |
| Onboarding, lock, recovery screens | `src/lib/screens/` |
| Three-pane shell | `src/lib/shell/` |
| Keyboard audit, started | `docs/keyboard-audit.md` |

## Notes

The webview heap cannot be wiped — `tauri-apps/tauri` discussion #10852. Every design choice in the
IPC layer follows from that one fact, so if a shortcut is ever proposed here ("just send the whole
item, it's simpler"), this is the note that explains why the answer is no.
