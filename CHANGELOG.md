# Changelog

All notable changes to TrustVault are documented here. The project follows Semantic Versioning.

## [Unreleased]

## [0.1.0] - 2026-09-13

Built locally, not published: `docs/release.md` reserves published releases for the tagged CI
workflow, and no tag was cut for this. The version exists so an installed package can be upgraded
in place rather than reinstalled over itself.

### Added

- A **Password generator** section in Settings with an **Include symbols** switch, off by default.
  It is the starting state for the generator dialog's symbol chip and the only character-set
  choice the one-click generate in the New item and Edit item dialogs has. The dialog's chip still
  overrides it for one password.

### Changed

- Generated passwords no longer contain symbols unless that switch is on. Existing settings files
  gain the key with its default the first time settings are written; nothing else about an
  installed vault or its preferences changes.

## [1.0.0] - TBD

### Added

- Portable, local-first encrypted `.tvault` files with Argon2id and XChaCha20-Poly1305.
- Recovery kits, seven item types, version history, search, password generation, and TOTP.
- Bitwarden JSON import with explicit handling for every source item type.
- Watchtower strength, reuse, and opt-in HIBP range checks.
- Linux x86-64 `.deb` and `.AppImage` release packaging. macOS and Windows packaging is deferred.

[Unreleased]: https://github.com/shoelfikar/trustvault/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/shoelfikar/trustvault/releases/tag/v1.0.0
[0.1.0]: https://github.com/shoelfikar/trustvault/compare/v0.0.0...v0.1.0
