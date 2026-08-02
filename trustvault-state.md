# TrustVault — State

Answers one question: where is this project now, and what happens next. It holds no requirements, no
calculations, and no task text — those live in `trustvault-requirements.md`, `docs/vault-format.md`,
and `phases/`.

Last updated: 2026-08-02

## Overall progress

| | |
|---|---|
| Current phase | 0 — Workbench, **gate passed**. Phase 1 is not open yet |
| Phase document | `phases/phase-0-workbench.md` |
| Phases passed | 1 of 6 |
| Last gate passed | Phase 0, on 2026-08-02 |
| Next gate | Phase 1 — opens once its entry check has been run |
| Status | on track |

## Current phase

**Phase 0 — Workbench.** Get a repository, a CI pipeline, and a window that opens, so that Phase 1
can be about cryptography and nothing else.

- Entry check: not applicable — this is the first phase and it opened at kickoff
- Tasks: **23 of 23 done**, as of 2026-08-02. Snapshot only — the checkboxes in the phase document
  are authoritative.
- In progress right now: nothing. Phase 1 does not open until its entry check has been run
- Blocked: nothing. Every Phase 0 criterion has evidence behind it — the specimen was confirmed on
  screen, the bundle identifier is fixed at `id.sulfikardi.trustvault`, and CI is green across all
  seven jobs including the three platform builds

## Gates

Three states only, and a gate the project passed through without actually running is **not ticked** —
it is marked `not run — retrofitted` with the risk carried written next to it.

- [x] **Phase 0** — window opens with the token specimen in both themes; all CI quality gates green. Passed 2026-08-02, run 30747639101
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
| 2026-08-02 | **D-20** `src-tauri/icons/` stays committed although it is derivative | `90-MOC/Gitignore Standard.md`'s single test says ignore it — one command regenerates it. Two things override that. It is a shipped artifact, so building it would let a `tauri` CLI upgrade change the user-visible icon with no diff to review; and measured on the day, `tauri icon` is **not byte-deterministic** — the same input yields a different `icon.icns` each run, so ignoring it would add unreviewable churn to every CI run rather than removing 380 KB of noise. `scripts/make-icon.py` was added so `icon-source.png` is itself reproducible byte-for-byte, which is what the artifact was missing | Ignoring `src-tauri/icons/` and generating during the build (churn, and an unreviewed icon change on every CLI bump); leaving `icon-source.png` as a binary nobody could regenerate, which is the weakest possible reason to commit something |
| 2026-08-02 | **D-18** Workspace MSRV raised from 1.85 to 1.88 | 1.85 was picked as a conservative default and turned out to be actively harmful: it pinned `cargo update` to versions still carrying RUSTSEC-2026-0009 (`time`) and RUSTSEC-2026-0194/0195 (`quick-xml` via `plist`). The patched releases require 1.88. An MSRV below what the security patches need is an MSRV that blocks them | Staying on 1.85 and ignoring three real DoS advisories; `--ignore-rust-version` in CI, which fixes the lockfile while leaving the manifest lying about what the project needs |
| 2026-08-02 | **D-19** `.cargo/audit.toml` ignores 17 advisories individually, with reasons | The GTK3 binding crates, `glib`'s unsoundness, and the `unic-*`/`proc-macro-error` crates all arrive through Tauri's Linux backend and cannot be fixed from here. Leaving CI permanently red on them trains everyone to ignore CI, which costs more than the advisories do. Each entry names why it is unfixable and what retires it, and anything not listed still fails | A blanket `--ignore-warnings`, which would hide new findings too; leaving the job red, which makes the signal worthless; dropping the audit job entirely, which is how N-04 quietly dies |
| 2026-08-02 | **D-17** The repository is public: `github.com/shoelfikar/trustvault` | Two reasons. A password manager asking for trust should be auditable, and a public repository gets unlimited GitHub Actions minutes — which matters because CI builds Linux, macOS, and Windows on every push, and the macOS runner bills at a 10× multiplier on private repositories. Consequence carried: every commit message, the design system, and anything pushed by mistake are permanently public, so the pre-push secret scan becomes a habit rather than a one-off | Private — safer default and trivially flipped to public later, rejected for the Actions cost and because the audit argument only works if the code is actually visible |
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

1. **Run the Phase 1 entry check** in `phases/phase-1-vault-core.md` and record the result in this
   session log. Phase 1 does not open until that is done — Phase 0's gate passing is not the same
   event as Phase 1 opening.
2. **Settle the Argon2id parameters by measuring them**, per open question 3. This blocks the Phase 1
   gate and is the first thing in that phase that cannot be guessed.
3. **Write `docs/vault-format.md` before any crypto code.** A format extracted from an
   implementation afterwards is a description of whatever the implementation happened to do.

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

### 2026-08-02 (later)

Repository published at `github.com/shoelfikar/trustvault` (public, D-17) and CI ran for the first
time. **All three platform builds passed on the first attempt** — Linux, macOS 15, and Windows 2025
— which was the sequencing risk most likely to bite, since none of those runners had ever executed.

The audit job failed, for two unrelated reasons that looked like one:

1. `rustsec/audit-check` could not create its check run — `Resource not accessible by integration`.
   The default `GITHUB_TOKEN` is read-only, so the job needs `permissions: checks: write`. This is a
   reporting failure, not a finding, and it masked the real result.
2. Three genuine vulnerabilities, all denial-of-service, all transitive through Tauri:
   RUSTSEC-2026-0009 (`time`, stack exhaustion) and RUSTSEC-2026-0194/0195 (`quick-xml`, quadratic
   parse and unbounded allocation, reached via `plist`).

The interesting part is why they were not simply patched away: `cargo update` refused to move,
reporting "Locking 0 packages to latest Rust 1.85 compatible versions". The workspace `rust-version`
had been set to 1.85 as a conservative default, and the patched releases need 1.88 — so a setting
chosen for caution was holding three security fixes out of the build. Raised to 1.88 (D-18), after
which `time` went 0.3.45 → 0.3.55 and `quick-xml` 0.38.4 → 0.41.0. Clippy and the tests still pass.

The remaining 17 findings — 16 unmaintained gtk-rs GTK3 crates plus `glib`'s `VariantStrIter`
unsoundness — arrive through Tauri's Linux backend and cannot be fixed here. They are now ignored
individually in `.cargo/audit.toml` with a reason and a retirement condition each (D-19), rather
than left to make the audit job permanently red.

Phase 0 gate passed later the same day. The audit fixes went green on run 30747639101 — all seven
jobs, including the three platform builds. The last open task, "no network font requests", was
closed by checking the built bundle rather than a devtools session: every `@font-face` source
resolves to a local `/assets/*.woff2`, there is no `@import` or font-CDN reference anywhere in
`dist/`, and the CSP pins `font-src 'self'`. That is reproducible in CI later, which an eyeballed
network tab is not.

Phase 0 is closed at 23/23. **Phase 1 is not open** — its entry check has not been run, and running
it is the first next action.

Audited `.gitignore` against `90-MOC/Gitignore Standard.md`. Three real findings, none of them a
wrong pattern:

- The repo's `.gitignore` carried `.DS_Store`, `Thumbs.db`, `.idea/`, and `*.swp`. Per the
  standard those are a statement about somebody's editor, not about this project, and belong in
  `~/.config/git/ignore` — which did not exist and was not configured. Created it, set
  `core.excludesFile`, and removed the lines from the repo.
- `!.env.example` had no comment. The standard requires a reason on every `!` line, because an
  exception without one gets deleted by the next person tidying up and the effect surfaces much
  later.
- `*.cer` was grouped with secrets. A `.cer` is a public certificate; the line was dropped and
  `*.pem` and `*.key` added in its place, which are the ones that actually matter.

Two things were checked and found **correct**, contrary to first impressions. The KAT-vector
exception `!crates/trustvault-core/tests/vectors/*.tvault` works — `git check-ignore -v` prints
negated matches too, so its output initially read as a failure; `git add --dry-run` shows the
vectors are committable while `personal.tvault` is refused. And nothing derivative was tracked by
accident: `git status --ignored` lists only `dist/`, `node_modules/`, `target/`, and
`src-tauri/gen/`.

The icon question is written up as D-20. It is the one place this repo knowingly departs from the
standard's default, so the reason is recorded rather than left to be rediscovered.
