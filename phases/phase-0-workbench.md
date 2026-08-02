# Phase 0 — Workbench

Status: current
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

Everything that must exist before a line of vault code is worth writing: the repository baseline, the
CI pipeline across all three target platforms, the design tokens lifted out of the prototype into
real CSS, the bundled fonts, and a Tauri window that opens. Requirements covered: N-04, N-06.

## Entry check — before the first task

Not applicable. This is the first phase; it opened at kickoff on 2026-08-02 with no upstream gate to
read against.

## Exit gate

- [ ] `npm run tauri dev` opens a window on Linux rendering the token-specimen page — every colour,
      type, space, radius, and shadow token from `MASTER.md` — with the theme toggle switching both
      themes live
      *(App builds, launches, and stays alive; the dev server serves the specimen. The visual half —
      that it renders correctly and both themes switch — has not been confirmed by a human yet.)*
- [x] Locally: `cargo clippy --workspace --all-targets -D warnings`, `cargo fmt --all --check`,
      `cargo test --workspace`, `svelte-check`, and `prettier --check` all pass
- [ ] The same set passes **in CI on a clean checkout** — the workflow is written but has never run,
      because there is no remote yet
- [ ] CI produces an unsigned build artifact on all three platforms (Linux, macOS, Windows) — proving
      the runners work before Phase 5 adds signing on top

Ticking this box is the same event as ticking the corresponding gate in `trustvault-state.md`. Do
both in the same session or neither.

## Tasks

### Environment

- [x] Tauri Linux system dependencies installed and verified by `pkg-config`:
      ```
      sudo apt install -y libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
        libsoup-3.0-dev libgtk-3-dev librsvg2-dev libssl-dev \
        libayatana-appindicator3-dev build-essential pkg-config curl wget file
      ```
- [x] `cargo tauri info` reports no missing dependency — webkit2gtk-4.1 2.52.3, rsvg2 2.61.3,
      rustc 1.97.1, node 24.18.0
- [x] `scripts/dev.sh` — launcher that strips the snap environment, because the editor on this
      machine is a snap and its terminal poisons any natively-built binary it starts

### Repository baseline

- [x] Git repository on `main` with `.gitignore` covering `target/`, `node_modules/`, `dist/`,
      and — importantly — `*.tvault` so a real vault can never be committed by accident
- [x] `LICENSE` retained and referenced from `README.md`
- [x] `README.md`: what it is, how to build, where the documents live
- [x] `CLAUDE.md` for the repo: the vault-core/UI boundary, the "no secret across IPC" rule, and a
      pointer to `design-system/password-manager/MASTER.md` as binding for all UI work
- [x] First commit made on `main`
- [ ] Remote added and pushed, so CI can run for the first time

### Scaffold

- [x] Vite + Svelte 5 + TypeScript frontend that builds — 56.7 kB JS (21.6 kB gzip), 11.3 kB CSS
- [x] `src-tauri/` Tauri v2 app that opens a 1360×864 window with native decorations, minimum
      720×520 (`MASTER.md` §6)
- [x] Cargo workspace with `crates/trustvault-core` as a stub carrying `#![forbid(unsafe_code)]`
      from its first commit — N-05
- [x] CI check asserting `trustvault-core` depends on neither Tauri nor any UI crate — N-02.
      Verified locally: the core's whole tree is serde, thiserror, zeroize and their proc-macros

### Design system

- [x] `src/lib/styles/tokens.css`: every token from `MASTER.md` §2, §3, §4, §5 as CSS custom
      properties, both themes, driven by `[data-theme]` with an OS-default fallback
- [x] Geist Sans and Geist Mono bundled as local `.woff2` with `@font-face` (latin + latin-ext,
      83 KB total, OFL), vendored rather than depended on
- [ ] Zero network font requests confirmed with devtools offline — bundling is done, the
      verification is not
- [x] `@lucide/svelte` wired, 16px grid, 1.5px stroke as the default
- [x] Token specimen screen rendering all colours with their **computed** contrast ratios, the full
      type scale, the space scale, radii, and shadows — this is the gate's evidence, and it stays in
      the repo as a regression surface
- [ ] Bundle identifier confirmed before it ships — currently `id.sulfikardi.trustvault`, a guess,
      and it cannot change after the first release without orphaning installed copies

### Quality gates

- [x] `.github/workflows/ci.yml`: fmt, clippy, `cargo audit`, `cargo test`, `svelte-check`,
      `prettier --check`, and a build on `ubuntu-latest`, `macos-15`, `windows-2025` — N-04, N-06
- [x] CI job asserting `trustvault-core` has no Tauri, UI, or network dependency — N-02
- [x] Prettier configured for Svelte 5 and TypeScript. **No ESLint** — N-06 lists four gates
      (clippy, cargo fmt, svelte-check, prettier) and `svelte-check` already covers what an ESLint
      Svelte config would catch here
- [x] `rustfmt.toml` and `clippy.toml` committed so local and CI agree

Total: 20/23. Removing a task from the *current* phase needs a line in the decision log.

Outstanding: the offline font check, the bundle identifier, and pushing to a remote so CI runs.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Frontend scaffold | `src/`, `vite.config.ts`, `svelte.config.js` |
| Tauri app | `src-tauri/` |
| Vault core stub | `crates/trustvault-core/` |
| Design tokens | `src/lib/styles/tokens.css` |
| Bundled fonts | `src/lib/fonts/` |
| Token specimen | `src/lib/screens/Specimen.svelte` |
| CI pipeline | `.github/workflows/ci.yml` |
| Repo instructions | `CLAUDE.md`, `README.md` |

## Notes

The prototype at `TrustVault App.dc.html` is the authority for pixel values, and
`design-system/password-manager/MASTER.md` is the authority for the rules behind them. Where they
disagree, `MASTER.md` wins and the disagreement gets logged — the prototype is a rendering of the
rules, not a second source of them.
