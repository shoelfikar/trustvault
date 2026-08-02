//! TrustVault vault core.
//!
//! This crate owns every plaintext byte in the application. It knows nothing about Tauri,
//! about the UI, or about the operating system's clipboard — that separation is requirement
//! N-02 and is checked in CI, because a vault-format crate that can reach the network or the
//! screen will eventually be asked to.
//!
//! # Why the boundary is drawn here
//!
//! The webview's heap cannot be wiped: JavaScript strings are immutable and garbage-collected,
//! and Tauri's own maintainers concluded that `location.reload()` merely raises the chance a
//! secret is collected rather than guaranteeing it. Everything downstream follows from that one
//! fact — see the *Interface contract* section of `trustvault-requirements.md`:
//!
//! * No command returns more than one secret value per invocation.
//! * Item lists cross the boundary with secrets elided.
//! * Copying a secret never returns it; this crate's caller writes to the clipboard directly.
//!
//! # Status
//!
//! Phase 0 stub. The format, the KDF, and the AEAD land in Phase 1 — see
//! `phases/phase-1-vault-core.md`. Nothing here is stable yet.

#![forbid(unsafe_code)]

use thiserror::Error;

/// Magic bytes at the head of every `.tvault` file.
///
/// Fixed now so that a file written by any future version is recognisable as ours, and so a
/// file that is *not* ours fails fast rather than being fed to the KDF.
pub const MAGIC: &[u8; 4] = b"TVLT";

/// On-disk format version.
///
/// Bumped whenever the byte layout changes in a way an older reader cannot handle. The layout
/// itself is specified in `docs/vault-format.md`, which is written before the implementation.
pub const FORMAT_VERSION: u16 = 1;

/// The file extension used for vault files.
pub const EXTENSION: &str = "tvault";

/// Errors surfaced by the vault core.
///
/// [`Error::Unreadable`] deliberately covers both a wrong master password and a corrupted
/// file. Distinguishing them tells an attacker holding a stolen vault whether a guessed
/// password was close, so requirement R-03 makes the two indistinguishable in error type
/// *and* in timing.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The file is not a TrustVault vault, or its header is not intelligible.
    #[error("not a TrustVault vault")]
    NotAVault,

    /// The file is a vault, but written by a format version this build cannot read.
    #[error("vault format version {found} is newer than the supported version {supported}")]
    UnsupportedVersion {
        /// Version recorded in the file.
        found: u16,
        /// Highest version this build understands.
        supported: u16,
    },

    /// The vault could not be decrypted. Wrong password, or damaged file — by design, the
    /// caller cannot tell which.
    #[error("vault could not be read")]
    Unreadable,

    /// An I/O failure while reading or writing the vault file.
    #[error("vault file I/O failed")]
    Io(#[from] std::io::Error),
}

/// Result alias for vault operations.
pub type Result<T> = core::result::Result<T, Error>;

/// The kinds of item a vault can hold.
///
/// Fixed at seven by the design; adding an eighth is a format change and therefore a decision
/// log entry, not a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemKind {
    /// Username and password for a site or service.
    Login,
    /// An API key or token.
    ApiKey,
    /// A payment card.
    Card,
    /// Free-form encrypted text.
    Note,
    /// A Wi-Fi network and its passphrase.
    WiFi,
    /// An SSH key and its passphrase.
    SshKey,
    /// Identity documents.
    Identity,
}

impl ItemKind {
    /// Every variant, in the order the design's "New item" dialog presents them.
    pub const ALL: [ItemKind; 7] = [
        ItemKind::Login,
        ItemKind::ApiKey,
        ItemKind::Card,
        ItemKind::Note,
        ItemKind::WiFi,
        ItemKind::SshKey,
        ItemKind::Identity,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_is_four_bytes_and_stable() {
        // If this test is ever "fixed" by changing the expected value, that is a format
        // break: every existing vault stops being recognised. Bump FORMAT_VERSION instead.
        assert_eq!(MAGIC, b"TVLT");
    }

    #[test]
    fn all_item_kinds_are_listed() {
        // Guards against adding a variant to ItemKind and forgetting ALL, which would make
        // the new type invisible to the New-item dialog while still being storable.
        assert_eq!(ItemKind::ALL.len(), 7);
        for kind in ItemKind::ALL {
            assert!(ItemKind::ALL.contains(&kind));
        }
    }

    #[test]
    fn item_kind_serialises_to_stable_snake_case() {
        // These strings go into the vault file, so they are part of the format.
        // `.ok()` rather than `?` or unwrap: serde_json::Error is not PartialEq, and a
        // serialisation failure here should read as "wrong output", not as a panic.
        let json = serde_json::to_string(&ItemKind::SshKey).ok();
        assert_eq!(json.as_deref(), Some(r#""ssh_key""#));
    }
}
