# Phase 3 — Working surfaces

> **Provisional.** Written at kickoff as a forecast, not a plan. Run the entry check below before
> the first task, then delete this notice. — *Delete these two lines once the phase is current.*

Status: not started
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

Everything that turns a reader into a usable tool: the password generator, the ⌘K command palette,
add/edit/delete with the naming confirmation, tags, TOTP, the settings screen including UI scale, and
multi-vault switching. Requirements covered: R-15…R-22, R-27, R-30.

## Entry check — before the first task

- [ ] **Still in scope** — traceable to `trustvault-project.md`
- [ ] **Requirements still live** — R-15…R-22, R-27, R-30
- [ ] **Dependencies passed their gates** — read the gates table in `trustvault-state.md`. Depends
      on: **G-B′**
- [ ] **External dependencies met** — the **import question (R-29) is resolved** before the first
      task, not during. It changes the item model if the answer is yes
- [ ] **Task list re-checked** against what Phase 2 actually taught
- [ ] **Exit gate still measurable** as written

Entry check completed: —

## Exit gate

- [ ] All 15 surfaces from the design exist and are reachable
- [ ] S-08 passes: 100 % of surfaces operable with no pointer, verified against
      `docs/keyboard-audit.md`
- [ ] Every box in `MASTER.md` §10 is ticked
- [ ] The author has used TrustVault as their **only** password manager for 7 consecutive days
      without falling back
- [ ] R-29 resolved: either a requirement with a phase, or an out-of-scope line with a decision entry

## Tasks

### Generator

- [ ] Length 8–64, four toggleable character classes — R-15
- [ ] Four-segment strength meter with the word and the crack-time estimate (`MASTER.md` §2)
- [ ] Ambiguous glyphs excluded by default (0/O, 1/l/I) — `MASTER.md` §3
- [ ] Generated values come from the Rust CSPRNG, not `Math.random`

### Command palette

- [ ] ⌘K/Ctrl+K opens; Esc closes; arrows navigate — R-16
- [ ] Fuzzy search over titles, usernames, URLs, tags — R-16
- [ ] Actions surfaced alongside items ("Lock vault", "Generate password", "New login")
- [ ] Enter copies the password, ⇧Enter opens the item — R-16
- [ ] S-04 met: ≤ 50 ms p95 keystroke-to-render on the reference vault

### Item management

- [ ] Add dialog for all seven types with their type-specific fields — R-17
- [ ] Edit and delete — R-17
- [ ] Delete confirmation naming the item; vault deletion requires typing the vault name — R-18
- [ ] Tag assignment and filtering
- [ ] Empty state on every list — R-19

### TOTP

- [ ] RFC 6238 generation, verified against the RFC test vectors — R-20
- [ ] Countdown ring in the detail pane
- [ ] Secret entry and live preview in the Add dialog

### Settings & vaults

- [ ] UI scale (Compact 92 % / Default 100 % / Large 115 %) scaling the root `rem` — R-21
- [ ] Auto-lock interval, clipboard clear interval, reveal logging, launch at login, theme — R-21
- [ ] Settings persist and take effect without a restart — R-21
- [ ] Multi-vault switcher; switching locks and zeroizes the outgoing vault — R-22
- [ ] Leave vault and delete vault flows
- [ ] Window geometry and pane widths persisted — R-27

### Accessibility

- [ ] `:focus-visible` ring on every interactive element, distinct from selection — `MASTER.md` §7
- [ ] `docs/keyboard-audit.md` completed and passing — R-30, S-08
- [ ] Contrast audited in both themes — S-09

Total: 0/25.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Generator, palette, dialogs | `src/lib/overlays/` |
| Settings screen | `src/lib/screens/Settings.svelte` |
| TOTP support | `crates/trustvault-core/src/totp.rs` |
| Completed keyboard audit | `docs/keyboard-audit.md` |

## Notes

The 7-day daily-drive requirement in the gate is the only test that catches the things no unit test
will: the item that takes four keystrokes too many, the dialog that steals focus, the auto-lock
interval that is wrong for real life.
