# TrustVault — working rules

## Before changing anything

Read `trustvault-state.md` first. It names the current phase, the gate that has not passed, and the
next actions. Then open the phase document it names and read the unticked boxes — the counter in
`state.md` can be a session stale, the checkboxes cannot.

Do not tick a gate for work that happened without it. Mark it `not run — retrofitted` with the risk
named, or run it now if it is cheap.

## The one rule that shapes everything

**The webview cannot be trusted with plaintext.** JavaScript strings are immutable and
garbage-collected; there is no way to wipe a secret out of the webview heap once it is there. Every
architectural decision below follows from that.

- `crates/trustvault-core` owns every plaintext byte. It must never depend on Tauri, a webview, a UI
  crate, or a network stack (N-02). CI fails the build if it does.
- No Tauri command returns more than **one** secret value per invocation (R-10).
- Item lists cross IPC with secrets **elided** — masked placeholders and metadata only.
- `copy_field` never returns the value. Rust writes to the clipboard and schedules the clear.
- Lock state lives in the core and is authoritative. A webview reload must not unlock anything, and
  every command re-checks the lock rather than trusting the frontend to have cleared its stores.

If a change would make any of these simpler to violate, it is the wrong change — say so rather than
making it.

## UI work

`design-system/password-manager/MASTER.md` is binding, not advisory. Its ban list is the point:

- No indigo, violet, or emerald accents. No blue-cast slate neutrals. Brass `--accent` is
  **brand and interaction only** — it never means "good" or "warning".
- No gradients, no mesh blobs, no glassmorphism except the lock-screen scrim.
- No radii above 6px outside avatars. No shadows on rows, cards, sidebar, or toolbar — hairline
  borders do the work.
- No emoji as icons. Lucide only, 16px grid, 1.5px stroke.
- Status is never colour alone: always an icon **and** a text label.
- Every secret renders in `--font-mono` with tabular numerals and disambiguated `0/O`, `1/l/I`. This
  is a correctness requirement for hand-transcription, not a style preference.

Never hardcode a colour, size, or duration. Everything comes from `src/lib/styles/tokens.css`, and a
token is only added if a line in `MASTER.md` justifies it.

The prototype (Claude Design project "Brankas App Design" → `TrustVault App.dc.html`) is the
authority for pixel values; `MASTER.md` is the authority for the rules behind them. Where they
disagree, `MASTER.md` wins and the disagreement goes in the decision log.

## Rust

- `trustvault-core` is Tier 1: `forbid(unsafe_code)`, `unwrap`/`expect`/`panic` denied by lint,
  ≥ 90 % coverage. A defect there is unrecoverable data loss.
- Errors must fail closed. A wrong password and a corrupted file are deliberately indistinguishable
  in both error type and timing (R-03) — do not "improve" the error message to tell them apart.
- Key material is wrapped in `Zeroizing`/`Secret` and zeroized on drop (N-01).
- Nonces come from the OS CSPRNG, never a counter (R-06).

## Frontend

Svelte 5 with runes (`$state`, `$derived`, `$effect`) — not the Svelte 4 store syntax. Plain Vite,
no SvelteKit: this is one window with screens, not a site with routes.

`style-src 'unsafe-inline'` is present in the CSP because Svelte injects component styles. That is
the only relaxation; do not widen `script-src`, `connect-src`, or `font-src` without a decision log
entry.

## Process

- Every library, framework, or key component choice needs prior art *before* adoption: maintenance,
  licence, platform support, resource cost, exit path. Log the alternatives rejected and what for.
- Every decision goes in the decision log in `trustvault-state.md` **with the alternatives
  rejected** — what is needed later is the constraint that produced the choice.
- Update `trustvault-state.md` before the session ends, especially the next-actions section.
- Commit messages in English, always. No AI attribution trailers.
