# TrustVault — Roadmap

Every phase below divides the scope in `trustvault-project.md`; a phase carrying scope that isn't
written there is scope creep. The phase model and its gates come from
`90-MOC/Development Standard.md` in the vault.

This is a software-only project, so the hardware gates do not exist. **G-A** (pin mapping locked) has
no analogue and is dropped. **G-B** (bring-up passed, errata written) is replaced by
**G-B′ — the IPC security boundary holds**: the equivalent "the physical thing behaves as drawn"
moment for this project is proving that no plaintext secret escapes the core except through the one
sanctioned path. **G-C** keeps its meaning: success criteria proven with measured data.

## Phases at a glance

The *Depends on* column is what the entry check of each phase is read against, so it names a
**gate**, not a phase number alone.

| Phase | Name | Carries | Depends on | Exit gate | Document |
|-------|------|---------|------------|-----------|----------|
| 0 | Workbench | Toolchain, repo baseline, CI, design tokens, fonts, shell that opens | — | Window opens on Linux showing the token specimen in both themes; all quality gates green in CI | `phases/phase-0-workbench.md` |
| 1 | Vault core | `.tvault` format v1, crypto, item model, atomic save, recovery kit — headless | Phase 0 gate | Format spec + KAT vectors committed; fail-closed proven by mutation test; ≥ 90 % coverage; `cargo audit` clean | `phases/phase-1-vault-core.md` |
| 2 | Shell & unlock | IPC boundary, onboarding, lock, recovery, three-pane shell, detail pane, reveal/copy | Phase 1 gate | **G-B′** — create → quit → relaunch → unlock → read works, and no secret crosses IPC outside the sanctioned path | `phases/phase-2-shell-unlock.md` |
| 3 | Working surfaces | Generator, ⌘K, add/edit/delete, tags, TOTP, settings, multi-vault | G-B′ | All 15 designed surfaces exist, reachable by keyboard alone; `MASTER.md` §10 checklist ticked; daily-drivable | `phases/phase-3-surfaces.md` |
| 4 | Watchtower | zxcvbn scoring, reuse detection, HIBP k-anonymity, opt-in | Phase 3 gate | Breach detection verified live and mocked; opted out → **zero** outbound packets, proven by capture | `phases/phase-4-watchtower.md` |
| 5 | Release | macOS + Windows builds, signing, notarization, release workflow, docs | **G-C** | Tagged release produces signed artifacts for all three targets; CI installs each in a clean environment and asserts the version | `phases/phase-5-release.md` |

**G-C** is not a phase — it is the gate between Phase 4 and Phase 5: every success criterion S-01…S-11
in `trustvault-requirements.md` filled in with a measured result.

## Phase 0 — Workbench

**Carries.** Everything needed before a line of vault code is worth writing: the repository baseline,
the CI pipeline, the design tokens lifted out of the prototype, the bundled fonts, and a Tauri window
that opens. Requirements: N-04, N-06.

**Depends on.** Nothing, except the Tauri Linux system dependencies being installed.

**Exit gate.** `npm run tauri dev` opens a window on Linux rendering a token-specimen page — every
colour, type, space, radius, and shadow token from `MASTER.md` — with the theme toggle switching both
themes live. In CI: `cargo clippy -D warnings`, `cargo fmt --check`, `cargo audit`, `svelte-check`,
and `prettier --check` all pass on a clean checkout.

**Deliverables.** Git repository with a first commit, `src-tauri/` + `src/` scaffold, the
`trustvault-core` crate stub, `tokens.css`, self-hosted Geist woff2, `.github/workflows/ci.yml`,
`CLAUDE.md` for the repo.

**Not in this phase.** Any cryptography. Any real UI screen. The temptation is to start the lock
screen because it is the most fun to draw — it needs the core to exist first.

## Phase 1 — Vault core

**Carries.** The `trustvault-core` crate in full, with no UI and no Tauri dependency: the `.tvault`
binary format, Argon2id KDF, XChaCha20-Poly1305 AEAD, the item model for all seven types, CRUD,
atomic save, and recovery-kit wrap/unwrap. Requirements: R-01…R-07, N-01, N-02, N-03, N-05, N-09.

**Depends on.** Phase 0 gate.

**Exit gate.** All of:
1. `docs/vault-format.md` exists and describes the byte layout well enough to write a second
   implementation from it.
2. Known-answer test vectors committed, with the passwords and expected ciphertexts, so a future
   refactor cannot silently change the format.
3. A mutation test flips every byte of a fixture vault; every mutation fails to decrypt (R-04).
4. Wrong password and corrupt file are indistinguishable in both error type and timing (R-03).
5. 100 injected process kills during save leave a readable vault every time (R-05).
6. `cargo llvm-cov` ≥ 90 % on the crate; `cargo audit` clean; zero unsafe.
7. The Argon2id open question is closed with **measured** parameters.

**Deliverables.** `crates/trustvault-core/`, `docs/vault-format.md`, KAT fixtures, benchmark results.

**Not in this phase.** Tauri commands. The IPC design is written down in Phase 1 but implemented in
Phase 2 — writing the commands early is how core ends up importing Tauri and N-02 quietly dies.

## Phase 2 — Shell & unlock

**Carries.** The Tauri command layer implementing the interface contract, then the first real
screens: onboarding's three steps, the lock screen, the recovery flow, the three-pane shell with
sidebar and item list, and the detail pane with per-field reveal and copy. Requirements:
R-07…R-14, R-28, N-07, N-08.

**Depends on.** Phase 1 gate.

**Exit gate — G-B′.** Both halves:
- *Functional:* create a vault through onboarding, quit the app, relaunch, unlock, and read an item —
  on Linux, with the real file on disk. Auto-lock fires. Clipboard clears (S-11).
- *Security:* an instrumented build logs every value crossing IPC; a scripted session exercising the
  whole shell produces a log containing no secret value except the ones explicitly revealed, and
  never more than one per command (R-10). `copy_field` never returns a value at all.

**Deliverables.** `src-tauri/src/commands/`, the shell and lock/onboarding screens, the IPC audit
harness, `docs/keyboard-audit.md` started.

**Not in this phase.** The command palette, the generator, and Watchtower. They are the interesting
parts and they all assume a working detail pane.

## Phase 3 — Working surfaces

**Carries.** Everything that makes it a usable tool rather than a reader: password generator, ⌘K
palette, add/edit/delete with the naming confirmation, tags, TOTP, the settings screen including UI
scale, and multi-vault switching. Requirements: R-15…R-22, R-27, R-30.

**Depends on.** G-B′.

**Exit gate.** All 15 surfaces from the design exist and are reachable; S-08 (100 % keyboard
reachability) passes against `docs/keyboard-audit.md`; every box in `MASTER.md` §10 is ticked; and
the author has used it as their only password manager for **7 consecutive days** without falling back
to the old one. The import open question (R-29) is resolved — either as a requirement with a phase,
or as an out-of-scope line with a decision log entry.

**Deliverables.** The remaining screens, the settings persistence layer, a filled-in
`docs/keyboard-audit.md`.

**Not in this phase.** Any network call. Watchtower's UI may be stubbed with the design's static
data, but nothing may leave the machine.

## Phase 4 — Watchtower

**Carries.** zxcvbn strength scoring, cross-item reuse detection, and opt-in HIBP breach checking
with k-anonymity and response padding. Requirements: R-23…R-26, S-07, S-10.

**Depends on.** Phase 3 gate.

**Exit gate.** Breach detection reports a known-pwned password correctly against both a mocked and
the live HIBP endpoint; a packet capture confirms only a 5-character SHA-1 prefix is sent and that
`Add-Padding: true` is honoured; and with the feature off, `tcpdump` on the app's PID shows **zero**
packets across 10 minutes of active use (S-10).

**Deliverables.** The Watchtower view backed by real data, the HIBP client, the opt-in setting, and
the packet-capture evidence recorded in `trustvault-state.md`.

**Not in this phase.** Automatic background scanning on a schedule. v1 scans on demand; a background
scanner is a network call the user did not just ask for.

→ **G-C** is crossed here: S-01…S-11 filled in with measured results before Phase 5 opens.

## Phase 5 — Release

**Carries.** Cross-platform builds, code signing on macOS and Windows, notarization, the release
workflow, checksums, and the install documentation. Requirements: N-10, and the distribution matrix.

**Depends on.** G-C.

**Exit gate.** A tagged release produces signed `.deb`, `.AppImage`, `.dmg`, and `.msi` artifacts with
published SHA-256 checksums, and the release workflow installs **each** of them in a clean
container or fresh VM and asserts the installed binary reports the release tag. A release where any
target fails to install is not a release.

**Deliverables.** `.github/workflows/release.yml`, signing key management documented (not the keys),
`docs/install.md`, the first tagged release.

**Not in this phase.** Auto-update. Shipping an updater means shipping a signed channel that can
push code to a machine holding secrets; it deserves its own phase and its own threat model.

## Sequencing risks

| Risk | Affects phase | Fallback |
|------|---------------|----------|
| Webview heap cannot be wiped, so a "revealed" secret may persist in JS memory longer than the 10 s timer suggests | 2 | Accept and document honestly in the UI. The mitigation is architectural (one field at a time, `copy_field` never returning a value), not a promise the platform can keep |
| Clipboard managers (GPaste, Klipper, CopyQ) cache copied secrets, defeating auto-clear | 2, 4 | Set `x-kde-passwordManagerHint`; where unsupported, the Settings copy must say "best effort" rather than claiming a guarantee |
| Windows HSM signing (Azure Trusted Signing) requires an account, identity verification, and lead time | 5 | Start the account setup during Phase 3, not Phase 5 — it is paperwork with a queue, and it blocks nothing else |
| Apple notarization adds 2–5 min per build and fails in ways that are only visible on a real macOS runner | 5 | Get one unsigned macOS build through CI in Phase 0 so the runner works before signing is added |
| Argon2id parameters that feel right on the dev machine are punishing on a slower laptop | 1 | Calibrate at vault creation against a target wall-clock time and store the result in the header (R-02) |
| The import question (R-29) reopens scope late | 3 | It is a named gate blocker, not a surprise. Resolve it before Phase 3 tasks start |
| macOS/Windows are never tested until Phase 5 | 5 | CI builds all three platforms from Phase 0, even though only Linux is developed against |

## Changes to this plan

A roadmap change is a decision: record it in the decision log in `trustvault-state.md`, then edit
here. The log holds the reason; this file only ever shows the current plan.
