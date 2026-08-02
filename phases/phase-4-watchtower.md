# Phase 4 — Watchtower

> **Provisional.** Written at kickoff as a forecast, not a plan. Run the entry check below before
> the first task, then delete this notice. — *Delete these two lines once the phase is current.*

Status: not started
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

The audit view backed by real data: zxcvbn strength scoring, cross-item reuse detection, and opt-in
HIBP breach checking using k-anonymity with response padding. Requirements covered: R-23…R-26, and
the measurement of S-07 and S-10.

This is the only phase that adds a network call to an application whose entire premise is that it
does not make any. The gate is written accordingly.

## Entry check — before the first task

- [ ] **Still in scope** — traceable to `trustvault-project.md`
- [ ] **Requirements still live** — R-23…R-26
- [ ] **Dependencies passed their gates** — read the gates table in `trustvault-state.md`. Depends
      on: Phase 3 gate
- [ ] **External dependencies met** — HIBP range API still free, unauthenticated, and still supports
      `Add-Padding`. Verify against the live service, not against this document
- [ ] **Task list re-checked** against what Phase 3 actually taught
- [ ] **Exit gate still measurable** as written

Entry check completed: —

## Exit gate

- [ ] A known-pwned password is reported correctly against **both** a mocked endpoint and the live
      HIBP service — R-25
- [ ] Packet capture confirms only a 5-character SHA-1 prefix leaves the machine, and that responses
      contain 800–1000 rows (padding honoured) — R-25
- [ ] With breach checking off — the default — `tcpdump` on the app's PID shows **zero** packets
      across 10 minutes of active use — S-10, R-26
- [ ] Full scan of the reference vault completes within 10 s — S-07
- [ ] Capture evidence recorded in the session log of `trustvault-state.md`, not just asserted

→ **G-C** is crossed after this gate: S-01…S-11 in `trustvault-requirements.md` filled in with
measured results before Phase 5 opens.

## Tasks

### Scoring

- [ ] zxcvbn integrated in the core, scoring every password in the vault — R-24
- [ ] Crack-time estimates rendered in words, matching zxcvbn's own phrasing — R-24
- [ ] Reuse detection: group items by password hash, report every member of a group of ≥ 2 — R-23
- [ ] Weak-password detection thresholds defined by zxcvbn score, not by length

### Breach checking

- [ ] HIBP range client: SHA-1, first 5 characters, `Add-Padding: true` — R-25
- [ ] Suffix matching done locally against the returned range
- [ ] Rate limiting and backoff, so a 1000-item vault is a polite caller
- [ ] Opt-in setting, off by default, with copy that says plainly what leaves the machine — R-26
- [ ] Offline and error paths: a failed check reports "not checked", never "safe"

### View

- [ ] Four stat tiles matching the design
- [ ] Breached / Reused / Weak groups with per-row actions routing to the item
- [ ] Status colours never carry meaning alone — icon and label always present (`MASTER.md` §2)
- [ ] Empty state for a clean vault that reads as reassurance, not as a blank screen

### Verification

- [ ] Packet capture harness scripted and repeatable
- [ ] Mocked HIBP fixtures committed so the test suite works offline

Total: 0/15.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Scoring and reuse detection | `crates/trustvault-core/src/watchtower.rs` |
| HIBP client | `src-tauri/src/hibp.rs` |
| Watchtower view | `src/lib/screens/Watchtower.svelte` |
| Capture evidence | `trustvault-state.md` session log |

## Notes

The HIBP client belongs in `src-tauri`, not in `trustvault-core` — N-02 keeps the core free of I/O,
and a vault-format crate that can open sockets is a vault-format crate that will eventually be asked
to.
