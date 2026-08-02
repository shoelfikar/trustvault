# TrustVault

A local-first desktop password manager for Linux, macOS, and Windows. Every secret lives in a
single portable encrypted `.tvault` file that you own — no account, no server, no sync.

> **Status: Phase 0 — Workbench.** Nothing is usable yet. The vault format and cryptography land
> in Phase 1. See `trustvault-state.md` for where the project actually is; the checkboxes in
> `phases/` are the authoritative record of what is done.

## Documents

Read these in order. They do not duplicate each other — each one is the single source of truth for
what it covers.

| File | What it holds |
|------|---------------|
| `trustvault-project.md` | End result, scope, out of scope, constraints, vocabulary |
| `trustvault-requirements.md` | Requirements with IDs, success criteria, the interface contract |
| `trustvault-roadmap.md` | The six phases and their exit gates |
| `trustvault-state.md` | **Read first when resuming.** Current phase, decision log, open questions, next actions |
| `phases/phase-N-*.md` | Tasks, as checkboxes. The only record of task completion |
| `design-system/password-manager/MASTER.md` | Binding for all UI work: colour, type, space, motion, components |

## Build

Requires Rust 1.85+ and Node 24+.

```bash
# Linux system dependencies (Ubuntu / Debian)
sudo apt install -y libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
  libsoup-3.0-dev libgtk-3-dev librsvg2-dev libssl-dev \
  libayatana-appindicator3-dev build-essential pkg-config curl wget file

npm install
npm run tauri dev
```

Other commands:

| Command | Does |
|---------|------|
| `npm run tauri dev` | Run the app with hot reload |
| `npm run tauri build` | Build an installable bundle |
| `npm run check` | Svelte + TypeScript type check |
| `npm run fmt` | Format the frontend |
| `cargo test --workspace` | Run the Rust tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint the Rust |

## Layout

```
crates/trustvault-core/   Vault format and cryptography. No Tauri, no UI, no network (N-02)
src-tauri/                Host process. The only place that touches both the core and the webview
src/                      Svelte 5 frontend
  lib/styles/tokens.css   Design tokens, generated from MASTER.md
  lib/fonts/              Geist Sans + Mono, vendored as woff2 (OFL)
  lib/screens/            Screens
phases/                   One document per roadmap phase
design-system/            The design system this app is built against
```

## Security posture

The webview's heap cannot be wiped, so the architecture assumes it is hostile:

- `trustvault-core` owns every plaintext byte, and CI enforces that it cannot reach Tauri, a
  webview, or a network stack.
- No IPC command returns more than one secret value.
- Copying a secret never returns it to JavaScript — the Rust side writes to the clipboard itself.
- Locking is authoritative in the core. Reloading the webview does not unlock anything.

The full contract is the *Interface contract* section of `trustvault-requirements.md`.

## Licence

GPL-3.0-or-later — see `LICENSE`. Geist Sans and Geist Mono are OFL; see
`src/lib/fonts/LICENSE-Geist.txt`.
