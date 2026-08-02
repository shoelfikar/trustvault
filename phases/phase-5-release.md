# Phase 5 — Release

> **Provisional.** Written at kickoff as a forecast, not a plan. Run the entry check below before
> the first task, then delete this notice. — *Delete these two lines once the phase is current.*

Status: not started
Scope and requirements: `trustvault-project.md`, `trustvault-requirements.md`. Plan:
`trustvault-roadmap.md`. Progress narrative: `trustvault-state.md`.

## What this phase carries

Cross-platform builds, code signing on macOS and Windows, notarization, the release workflow,
published checksums, and install documentation. Requirements covered: N-10 and the distribution
matrix in `trustvault-requirements.md`.

## Entry check — before the first task

- [ ] **Still in scope** — traceable to the Distribution section of `trustvault-project.md`
- [ ] **Requirements still live** — N-10; the distribution matrix still lists the same four artifacts
- [ ] **Dependencies passed their gates** — read the gates table in `trustvault-state.md`. Depends
      on: **G-C** (S-01…S-11 all filled in with measured results)
- [ ] **External dependencies met** — Apple Developer account active; Azure Trusted Signing account
      approved. **Both were started in Phase 3** precisely so they are not discovered here
- [ ] **Task list re-checked** against what Phase 4 actually taught
- [ ] **Exit gate still measurable** as written

Entry check completed: —

## Exit gate

- [ ] A tagged release produces signed `.deb`, `.AppImage`, `.dmg`, and `.msi` artifacts
- [ ] SHA-256 checksums published for every asset — N-10
- [ ] The release workflow installs **each** artifact in a clean container or fresh VM and asserts
      the installed binary reports the release tag. A release where any target fails to install is
      not a release
- [ ] `docs/install.md` covers all three platforms, including what to do about the macOS Gatekeeper
      and Windows SmartScreen prompts a new signing identity will produce

## Tasks

### Signing

- [ ] Apple Developer account active; Developer ID Application certificate in CI secrets
- [ ] macOS build signed and notarized; stapling verified — budget 2–5 min per build
- [ ] Azure Trusted Signing configured; Windows `.msi` signed via the HSM-backed cert
- [ ] Signing key handling documented in `docs/release.md` — the process, never the keys
- [ ] Local `.p12` and base64 intermediates deleted after upload to secrets

### Build & release

- [ ] `.github/workflows/release.yml` triggered by a version tag
- [ ] Linux `.deb` and `.AppImage`
- [ ] macOS `.dmg` for arm64
- [ ] Windows `.msi` for x86-64
- [ ] Checksums generated and attached to the release
- [ ] Unsupported targets (linux arm64, darwin x86-64) fail with a message naming what is supported

### Smoke tests

- [ ] `.deb` installed in a clean `ubuntu:26.04` container; binary reports the tag
- [ ] `.AppImage` run in clean `ubuntu:24.04` and `fedora:42` containers
- [ ] `.dmg` installed on a `macos-15` runner; binary reports the tag
- [ ] `.msi` installed on a `windows-2025` runner; binary reports the tag

### Documentation

- [ ] `docs/install.md`
- [ ] `docs/vault-format.md` linked from the README as the format users are trusting
- [ ] CHANGELOG for the first release
- [ ] Distribution matrix in `trustvault-requirements.md` updated with actual built/smoke-tested status

Total: 0/19.

## Deliverables

| Deliverable | Location |
|-------------|----------|
| Release workflow | `.github/workflows/release.yml` |
| Release process notes | `docs/release.md` |
| Install guide | `docs/install.md` |
| First tagged release | GitHub Releases |

## Notes

Auto-update is deliberately not in this phase. An updater is a signed channel that can push code onto
a machine holding decrypted secrets — it needs its own threat model, and bolting it onto a release
workflow is how that threat model gets skipped.
