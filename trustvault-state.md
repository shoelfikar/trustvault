# TrustVault — State

Answers one question: where is this project now, and what happens next. It holds no requirements, no
calculations, and no task text — those live in `trustvault-requirements.md`, `docs/vault-format.md`,
and `phases/`.

Last updated: 2026-08-02

## Overall progress

| | |
|---|---|
| Current phase | 0 — Workbench |
| Phase document | `phases/phase-0-workbench.md` |
| Phases passed | 0 of 6 |
| Last gate passed | none |
| Next gate | Phase 0 — blocked by three things: a human confirming the specimen renders, a git remote so CI can actually run, and the bundle identifier |
| Status | on track |

## Current phase

**Phase 0 — Workbench.** Get a repository, a CI pipeline, and a window that opens, so that Phase 1
can be about cryptography and nothing else.

- Entry check: not applicable — this is the first phase and it opened at kickoff
- Tasks: **20 of 23 done**, as of 2026-08-02. Snapshot only — the checkboxes in the phase document
  are authoritative.
- In progress right now: closing the Phase 0 gate
- Blocked: the gate's visual half. The app builds, launches, and serves the specimen, but nobody has
  looked at the window yet — there is no screenshot tool on this machine, so it needs human eyes.
  CI has also never run, because the repository has no remote

## Gates

Three states only, and a gate the project passed through without actually running is **not ticked** —
it is marked `not run — retrofitted` with the risk carried written next to it.

- [ ] **Phase 0** — window opens with the token specimen in both themes; all CI quality gates green
- [ ] **Phase 1** — format spec + KAT vectors committed; fail-closed proven; ≥ 90 % coverage; zero unsafe
- [ ] **G-B′** — IPC security boundary holds: no secret crosses IPC outside the sanctioned path
- [ ] **Phase 3** — all 15 surfaces reachable by keyboard alone; `MASTER.md` §10 ticked; 7 days daily-driven
- [ ] **Phase 4** — breach detection verified; zero egress when opted out, proven by packet capture
- [ ] **G-C** — S-01…S-11 filled in with measured results
- [ ] **Phase 5** — signed artifacts for all three targets, each installed and version-asserted in CI

**G-A (pin mapping locked) is dropped** — software-only project, no hardware track. **G-B is replaced
by G-B′**, because the equivalent "the physical thing behaves as drawn" moment here is the IPC
security boundary, not a bring-up. Both recorded as decision D-14.

## Decision log

Every decision with the alternatives rejected — what's needed later is the constraint that produced
the choice, not the choice.

| Date | Decision | Reason | Alternatives rejected |
|------|----------|--------|-----------------------|
| pre-kickoff | Tauri v2 + Rust for a desktop app | Predates this document set; recorded in `design-system/password-manager/MASTER.md` | Electron, native per-platform — **no prior-art survey was run for this choice**, so it is a risk carried, not a considered decision |
| 2026-08-02 | **D-02** Svelte 5 + TS + Vite for the frontend | Smallest bundle and fastest cold start, which is what a dock-resident tool opened 15×/day is judged on (S-01) | React 19 — heaviest runtime, though the prototype is already React so the port is now manual work we chose to pay for. SolidJS — React-like ergonomics but a smaller ecosystem. Leptos — one language end to end, rejected for WASM payload and total prototype rewrite |
| 2026-08-02 | **D-03** Offline only; no sync in v1 | The design contradicted itself — sidebar said "Synced 2 min ago", Settings said "Offline — this device only". Resolved to offline. Sync needs conflict resolution, a server, and key exchange | Building sync now; leaving the contradiction unresolved in the UI copy |
| 2026-08-02 | **D-04** Custom `.tvault` format, Argon2id + XChaCha20-Poly1305 | Matches the design's own artefacts (`.tvault` filenames, Argon2id onboarding copy, recovery kit) and leaves room for Watchtower and audit metadata | KeePass KDBX4 (`keepass-rs`) — battle-tested and gives users an exit path, but has no native slot for status/audit metadata and no recovery-kit concept. SQLCipher — real queries, but a C dependency and a poor fit for a single portable file. `age` — file encryption, not a vault format: no per-item crypto, no key rotation story |
| 2026-08-02 | **D-05** Tauri Stronghold plugin not used | It is a secrets store with its own format — adopting it means a second vault inside the vault, with two formats to version | Using Stronghold as the storage layer; using it only for the master key |
| 2026-08-02 | **D-06** XChaCha20-Poly1305, not AES-GCM | The 192-bit nonce is large enough to be drawn randomly with no counter discipline. Aegis's format docs reject counter-based nonces outright for exactly this failure mode | AES-GCM (96-bit nonce, needs a counter or a birthday-bound argument), AES-GCM-SIV (misuse-resistant but less common in Rust and slower) |
| 2026-08-02 | **D-07** Ship Linux, macOS, and Windows from v1 | Decided at kickoff so signing costs and CI matrix are phase-0 problems rather than release-day surprises | Linux-only first — cheaper, but defers the two hardest distribution problems to the end |
| 2026-08-02 | **D-08** Native installers, not a `curl \| sh` installer | `90-MOC/Release Installer Standard.md` governs single-binary CLI apps. A GUI app with a webview runtime cannot build to one binary. The standard's checksum and clean-environment smoke-test requirements are kept and moved into the Phase 5 gate | Following the standard literally; shipping archives only with no installer |
| 2026-08-02 | **D-09** No component library | shadcn-svelte and Skeleton default to radii, shadows, and densities that `MASTER.md` explicitly bans; every component would be fought rather than used | shadcn-svelte, Skeleton, Melt UI (kept as a possible headless-primitives fallback if the dialog/focus-trap work proves fiddly) |
| 2026-08-02 | **D-10** CBOR (`ciborium`) for the vault body | Tolerates unknown fields, so a vault written by a newer version round-trips through an older one without data loss (N-09) | `postcard` — smaller and faster but schema-rigid, an unknown field is a parse error. JSON — larger, and no clean binary field. `bincode` — same rigidity problem as postcard |
| 2026-08-02 | **D-11** `arboard` (1Password's fork) for clipboard | Needs the platform MIME hints (`x-kde-passwordManagerHint`) that thinner wrappers do not expose, and the crate documents the Wayland ownership caveat that bites password managers | `tauri-plugin-clipboard-manager` — simpler but no MIME hint control; `copypasta` — less maintained |
| 2026-08-02 | **D-12** `zxcvbn` crate for strength scoring | The design's crack-time strings ("takes ~8 centuries to crack") are zxcvbn's own output format, so using anything else means reimplementing its phrasing | Entropy-only scoring — cheap but reports `Jakarta2019!` as strong, which is exactly the case Watchtower exists to catch |
| 2026-08-02 | **D-13** Six phases, none merged | The 'no plaintext across IPC' rule needs its own gate; folding it into a larger phase is how it becomes an aspiration | Merging 0 into 1; compressing to three phases |
| 2026-08-02 | **D-15** `@lucide/svelte`, not `lucide-svelte` | The package installed first emitted a deprecation notice on install: `lucide-svelte` is superseded by the scoped package. Swapped before the first commit rather than carrying a deprecated dependency into the history | Staying on `lucide-svelte`; Phosphor (the approved alternate in `MASTER.md` §8) — no reason to switch icon families, only packages |
| 2026-08-02 | **D-16** `scripts/dev.sh` strips the snap environment before launching | The editor on this machine is a snap, and its integrated terminal exports `SNAP_LIBRARY_PATH`, `LOCPATH`, `GTK_PATH`, and `GIO_MODULE_DIR` pointing into `/snap/core20/`. A natively-built binary started from that terminal loads the snap's libc and dies before `main()` with `undefined symbol: __libc_pthread_init`. The binary is fine; the environment is not | Telling the developer to always use an external terminal (works, but is a trap that will be rediscovered every few months); patching `LD_LIBRARY_PATH` only (insufficient — the GTK and GIO module paths poison it too) |
| 2026-08-02 | **D-14** G-A dropped, G-B replaced by G-B′ | Software-only project: there is no pin mapping and no board bring-up. The IPC security boundary is the structural equivalent of "the physical thing behaves as drawn" | Keeping the hardware gates as empty ticks — which would make "already checked" indistinguishable from "never considered" |

## Open questions

- [ ] **Import from other password managers** — which formats, if any, in v1? A vault nobody can
      migrate into is a vault nobody adopts, but import was not in the agreed scope. Currently in the
      out-of-scope section of `trustvault-project.md` marked *open, not decided*. Resolve as either a
      requirement with a phase, or an out-of-scope line with a decision log entry. Blocks the
      **Phase 3 gate**. — R-29
- [ ] **Audit-log retention and location** — R-13 logs every reveal, but a plaintext record of which
      secrets were revealed and when is itself sensitive. Inside the encrypted vault, or beside it?
      Retained for how long? Blocks the **Phase 2 gate**.
- [ ] **Argon2id default parameters** — the m/t/p triple that hits S-03 (≥ 500 ms) must be measured
      per platform, not guessed. Blocks the **Phase 1 gate**.
- [ ] **Does the "Clears in 12s" chip survive contact with clipboard managers?** GPaste, Klipper, and
      CopyQ cache clipboard history; CopyQ issue #2802 documents exactly this against KeePassXC. If
      the hint is not honoured, the UI copy must change from a promise to a best-effort statement.
      Blocks the **Phase 2 gate**.

## Next actions

1. **Look at the running window.** `npm run dev:app` launches it. Confirm the specimen renders, the
   theme toggle switches both themes, the UI-scale segmented control resizes everything, and the
   contrast table reports no failures. That is the visual half of the Phase 0 gate and it needs a
   human — there is no screenshot tool on this machine.
2. **Decide the bundle identifier.** `id.sulfikardi.trustvault` is a guess. It is baked into
   installed copies and cannot be changed later without orphaning them.
3. **Add a remote and push.** The first commit exists on `main`; `.github/workflows/ci.yml` has been
   verified locally but has never executed on a clean checkout, and the macOS and Windows build legs
   have never run at all.
4. Then run the Phase 1 entry check and close the Argon2id parameter question by measuring, not
   guessing.

## Session log

### 2026-08-02

Kickoff. Read the Claude Design project `Brankas App Design`
(`7d8165c1-ec9a-4424-a50e-a24f8c2ac96f`) — `TrustVault App.dc.html`, a 1339-line interactive
prototype covering 15 surfaces, plus `TrustVault Logo.dc.html`. Confirmed the remote
`uploads/MASTER.md` is byte-identical to the local
`design-system/password-manager/MASTER.md`, so the design system has one source of truth and no
reconciliation was needed.

Two contradictions found in the prototype and resolved: sync (D-03), and the "Clears in 12s"
clipboard promise (now an open question rather than a silent assumption).

Ran the prior-art survey before proposing anything. Four findings changed the plan rather than
merely informing it:

- The **webview heap cannot be wiped** (`tauri-apps/tauri` discussion #10852). This is why the
  interface contract in `trustvault-requirements.md` limits IPC to one secret per command and why
  `copy_field` never returns a value — it is an architectural constraint discovered before any code
  was written, not a hardening pass bolted on later.
- **Clipboard ownership on Wayland/X11** belongs to the copying process, and clipboard managers
  cache secrets indefinitely. Turned into an open question against the Phase 2 gate.
- **HIBP response padding is opt-in**; without `Add-Padding: true` the response *size* leaks the
  queried prefix. Now R-25.
- **Windows OV certificates must live on an HSM** since June 2023. Moved the signing-account setup
  forward into Phase 3 so the paperwork queue does not block Phase 5.

Framing answered: Svelte 5 + TS + Vite; custom `.tvault` with Argon2id + XChaCha20-Poly1305; full
v1 scope (core + TOTP + Watchtower + multi-vault); all three platforms. Six-phase breakdown
proposed and confirmed. Document set generated. Repository initialized on `main`.

Toolchain checked: Rust 1.97.1 and Node 24.18.0 present; all five Tauri Linux system dependencies
were missing. The install needs interactive sudo, which the agent could not provide, so the user
ran it mid-session — `cargo tauri info` now reports webkit2gtk-4.1 2.52.3 and rsvg2 2.61.3 clean.

Scaffold built and verified in the same session. Green locally: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (3 tests),
`svelte-check` (0 errors over 4051 files), `prettier --check`, `vite build` (56.7 kB JS / 21.6 kB
gzip, 11.3 kB CSS, fonts emitted as local assets), and the N-02 boundary check — `trustvault-core`'s
entire dependency tree is serde, thiserror, zeroize and their proc-macros.

Three things bit during the build and are worth remembering:

- **Vite 8 dropped the bundled esbuild minifier.** `minify: 'esbuild'` now fails with a missing
  package; the minifier is `oxc`.
- **`svelte-check` cannot use TypeScript 7** (peer range is `^5 || ^6`), so the toolchain is pinned
  to TS 5.9.3 even though 7.0.2 is current. Revisit when svelte-check widens its range.
- **The snap-terminal crash** (D-16), which cost the most time and looked exactly like a broken
  build until the binary was run under a scrubbed environment and started fine.

The app now compiles, launches, and serves the specimen. What is *not* verified: that the window
renders correctly and that both themes switch — there is no screenshot tool on this machine, so the
visual half of the gate needs human eyes. CI has never executed either, because the repository has
no remote yet. Neither is ticked.
