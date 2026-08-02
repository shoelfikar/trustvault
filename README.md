# TrustVault

[![CI](https://github.com/shoelfikar/trustvault/actions/workflows/ci.yml/badge.svg)](https://github.com/shoelfikar/trustvault/actions/workflows/ci.yml)

A local-first desktop password manager for Linux, macOS, and Windows. Every secret lives in a
single portable encrypted `.tvault` file that you own — no account, no server, no sync.

> **Status: Phase 0 passed, Phase 1 not yet open.** Nothing is usable yet — there is no
> cryptography in this repository at all. The vault format lands in Phase 1.

## Planning documents

The project's process record — scope, requirements, roadmap, per-phase task lists, the decision
log, and the design system — is kept privately by the author and is not published here. Where the
code refers to a requirement by ID (`R-10`, `N-02`, `S-04`) or to a phase gate, that identifier
points into those documents.

What the repository does carry, and what a reader actually needs, is below: the security posture
this codebase is built around, and — from Phase 1 — `docs/vault-format.md`, which specifies the
`.tvault` byte layout well enough to write a second implementation from.

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
  lib/styles/tokens.css   Design tokens — the only place a colour, size, or duration is defined
  lib/fonts/              Geist Sans + Mono, vendored as woff2 (OFL)
  lib/screens/            Screens
scripts/                  Icon generation, and a launcher that survives a snap-packaged editor
```

## Security posture

The webview's heap cannot be wiped, so the architecture assumes it is hostile:

- `trustvault-core` owns every plaintext byte, and CI enforces that it cannot reach Tauri, a
  webview, or a network stack.
- No IPC command returns more than one secret value.
- Copying a secret never returns it to JavaScript — the Rust side writes to the clipboard itself.
- Locking is authoritative in the core. Reloading the webview does not unlock anything.

These are not conventions to be tidied away later. They exist because the webview's heap cannot be
wiped — JavaScript strings are immutable and garbage-collected — so a secret that reaches the
frontend cannot be taken back. A change that makes any of the above easier to violate is the wrong
change.

## Licence

GPL-3.0-or-later — see `LICENSE`. Geist Sans and Geist Mono are OFL; see
`src/lib/fonts/LICENSE-Geist.txt`.
