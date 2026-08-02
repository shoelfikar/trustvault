# TrustVault

A local-first desktop password manager for Linux, macOS, and Windows that stores every secret in a
single portable encrypted `.tvault` file, opens from the dock in under a second, and can be operated
end to end without touching the mouse.

Status: Phase 0 — see `trustvault-state.md`. Requirements: `trustvault-requirements.md`.
Plan: `trustvault-roadmap.md`.

## Why this exists

Cloud password managers put the entire secret store on someone else's server, and the two most
common local alternatives force a trade: KeePassXC has the right threat model and a UI from 2009,
while the polished products all require an account. There is room for a tool that keeps the vault as
a file you own and still feels like a machined instrument rather than a Java form.

If it isn't solved, nothing catastrophic happens — the honest driver is that this is a tool the
author wants to use daily, and building it is worth the effort on its own. That matters for how much
rigor it deserves: **the cryptography and the file format get full rigor because a bug there loses
data irrecoverably; the UI gets iterated.**

## Scope

What this project delivers. Each line is claimed by exactly one phase in the roadmap.

- A versioned `.tvault` file format: Argon2id key derivation, XChaCha20-Poly1305 AEAD, atomic writes
- A headless Rust vault core, usable and testable without any UI
- Seven item types: login, API key, card, secure note, Wi-Fi, SSH key, identity
- Onboarding (vault creation), lock/unlock, and a printable offline recovery kit
- The three-pane desktop shell drawn in the design: sidebar, item list, detail pane
- Per-field secret reveal (time-boxed) and copy with best-effort clipboard auto-clear
- A password generator with live strength scoring and crack-time estimates
- A ⌘K command palette as the primary navigation surface
- TOTP code generation (RFC 6238) with the countdown ring
- Watchtower: reuse detection, zxcvbn strength scoring, and opt-in HIBP breach checking
- Multiple vaults with a switcher, independent lock state, and leave/delete flows
- Two first-class themes, a UI-scale setting, and full keyboard operation
- Signed, installable builds for Linux, macOS, and Windows produced by CI

## Out of scope

What this project deliberately does **not** do — the most valuable section in this file, because it
is the one that gets quietly renegotiated at 2 a.m. A line moves out of here only through a decision
log entry in `trustvault-state.md`.

| Not doing | Why | Revisit when |
|-----------|-----|--------------|
| Cloud sync / multi-device | Sync means conflict resolution, a server, and a key-exchange protocol — each is a project. The design's "Synced 2 min ago" label was resolved to offline-only (decision D-03). | v1 has been daily-driven for 3 months |
| Browser extension / autofill | Native messaging plus a per-browser surface, and autofill is where password managers historically leak. | After v1 ships and the IPC boundary has proven stable |
| Mobile (iOS / Android) | Different threat model, different unlock story, and Tauri mobile is not ready to hold secrets. | Never for this project — it would be a separate one |
| Team / shared vaults | Requires asymmetric key exchange and an identity system. | Never |
| Biometric / OS-keyring unlock | Adds a platform-specific trusted path per OS; the master password must work first and alone. | Phase 6 candidate, after G-C |
| SSH agent integration | The vault stores SSH material in v1; *serving* it as an agent is a daemon with its own attack surface. | Never for v1 |
| Command-line client | The GUI must earn its keep first, and a CLI doubles the surface that touches plaintext. | After v1 |
| Custom window chrome on Windows/Linux | `MASTER.md` §6: costs more bugs than it buys. Native decorations, with `titleBarStyle: "Overlay"` on macOS only. | Never |
| Plugin / extension API | Third-party code in a process that holds decrypted secrets. | Never |

**Open, not decided:** importing from other password managers (KeePass CSV/XML, 1Password, Bitwarden,
browser exports). A vault you cannot migrate into is a vault nobody adopts, but it was not in the
agreed v1 scope. Tracked as an open question in `trustvault-state.md` — it must be resolved before
the Phase 3 gate, not silently dropped.

## Users and stakeholders

| Who | What they need from it |
|-----|------------------------|
| Primary user (the author) | A vault that opens 15× a day, is driven by keyboard, and never phones home |
| Future self, 3 years on | To open a `.tvault` written today with whatever version exists then — format versioning is a user requirement, not an implementation detail |
| A reviewer of the crypto | A file format spec and known-answer test vectors that can be checked without reading the UI |

## Hard constraints

| Constraint | Value | Source |
|------------|-------|--------|
| Target platforms | Linux x86-64, macOS arm64, Windows x86-64 | Kickoff decision |
| Network egress at rest | Zero. Only Watchtower may make an outbound request, and only when opted in | Design intent; `MASTER.md` §1 |
| Fonts | Bundled `.woff2`, no remote font origin | `MASTER.md` §3; Tauri CSP |
| Minimum window | 720 × 520 px | `MASTER.md` §6 |
| macOS signing | Apple Developer account, USD 99/yr | Apple; required for notarization |
| Windows signing | HSM-backed cert (Azure Trusted Signing) — exportable `.p12` no longer issued | CA/Browser Forum, June 2023 |
| Master password | Never persisted to disk in any form, including swap-visible allocations | Threat model |

## Tracks

- [ ] Hardware → vault: `90-MOC/PCB Design Standard.md`
- [ ] Firmware → vault: `90-MOC/Firmware Development Standard.md`
- [x] Software → vault: `90-MOC/Software Development Standard.md`

Single-track software project. The hardware and firmware gates (G-A pin mapping, G-B bring-up) do
not exist here; `trustvault-roadmap.md` names what replaces them.

## Distribution

| | |
|---|---|
| Channel | Native installers per platform, published as GitHub release assets |
| Platforms | linux amd64 (`.deb` + `.AppImage`), darwin arm64 (`.dmg`), windows amd64 (`.msi`) — matrix in `trustvault-requirements.md` |
| Users have sudo | Linux: yes (`.deb`); AppImage needs none. macOS/Windows: standard installer privileges |

**Deviation from `90-MOC/Release Installer Standard.md`:** that standard governs single-binary CLI
apps shipped via `curl … | sh`. TrustVault is a GUI desktop app with a webview runtime, so it cannot
build to one binary and a shell installer is the wrong shape. The parts of the standard that still
bind are kept and moved into the Phase 5 gate: **checksums published for every asset**, and **CI
installs each artifact in a clean container/VM and asserts the installed binary reports the release
tag**. Logged as decision D-08.

## Vocabulary

| Term | Meaning in this project |
|------|-------------------------|
| Vault | One `.tvault` file plus its in-memory decrypted state. Locking discards the state, not the file |
| Item | One record of one of the seven types. Has fields, tags, a status, and version history |
| Field | A labelled value inside an item. A field is either plain or **secret** |
| Secret | A field value that is masked by default and never crosses IPC unless explicitly requested |
| Reveal | A time-boxed unmask of one secret field. Emits an audit-log entry; auto-remasks after 10 s |
| Recovery kit | Six 4-character base32 groups that unwrap the master key without the master password |
| Watchtower | The read-only audit view: reuse, weakness, and breach findings across one vault |
| Locked | Master key zeroized, decrypted state dropped, UI showing the lock screen |
| Vault core | The `trustvault-core` Rust crate. Knows nothing about Tauri, the UI, or the OS clipboard |
