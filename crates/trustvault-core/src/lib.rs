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
//! # The format
//!
//! `docs/vault-format.md` is the specification and was written before this code. It is not a
//! description of what the implementation happens to do, and where the two disagree the document
//! is authoritative — with the known-answer vectors in `tests/vectors/` as the tie-breaker.
//!
//! # Example
//!
//! ```
//! use trustvault_core::{ItemKind, KdfParams, Vault};
//!
//! // Test parameters. Real vaults calibrate — see `KdfParams::calibrate`.
//! let (mut vault, recovery) = Vault::create("Personal", "correct horse", KdfParams::TESTING)?;
//! let id = vault.add_item(ItemKind::Login, "GitHub");
//! vault.item_mut(id).map(|item| item.set_field("Password", "hunter2", true));
//!
//! let bytes = vault.to_bytes()?;
//! drop(vault);
//!
//! // The password opens it, and so does the recovery kit, independently.
//! let reopened = Vault::open(&bytes, "correct horse")?;
//! assert_eq!(reopened.items().count(), 1);
//! assert!(Vault::open_with_recovery(&bytes, &recovery).is_ok());
//! assert!(Vault::open(&bytes, "wrong").is_err());
//! # Ok::<(), trustvault_core::Error>(())
//! ```

#![forbid(unsafe_code)]

mod aead;
mod format;
mod kdf;
mod model;
mod recovery;
mod secret;
mod vault;

pub use format::{HEADER_LEN, Header, WRAP_AAD_LEN};
pub use kdf::KdfParams;
pub use model::{
    AuditEntry, Field, FieldId, FieldKind, HistoryEntry, Item, ItemId, ItemKind, ItemStatus,
    VaultBody,
};
pub use recovery::RecoveryCode;
pub use secret::{SecretBytes, SecretString};
pub use vault::Vault;

use thiserror::Error;

/// Magic bytes at the head of every `.tvault` file.
///
/// Fixed now so that a file written by any future version is recognisable as ours, and so a
/// file that is *not* ours fails fast rather than being fed to the KDF.
pub const MAGIC: &[u8; 4] = b"TVLT";

/// On-disk format version.
///
/// Bumped whenever the byte layout changes in a way an older reader cannot handle. The layout
/// itself is specified in `docs/vault-format.md`, which was written before the implementation.
pub const FORMAT_VERSION: u16 = 1;

/// The file extension used for vault files.
pub const EXTENSION: &str = "tvault";

/// Algorithm identifier for Argon2id v0x13, stored at header offset 6.
pub const KDF_ID_ARGON2ID: u8 = 1;

/// Algorithm identifier for XChaCha20-Poly1305, stored at header offset 7.
pub const AEAD_ID_XCHACHA20POLY1305: u8 = 1;

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

    /// The recovery code is not 24 characters of the RFC 4648 base32 alphabet.
    ///
    /// Distinct from [`Error::Unreadable`] on purpose: this is a transcription mistake found
    /// before any key material exists, so reporting it leaks nothing about the vault. A
    /// *correctly formed* code that does not open the vault still returns `Unreadable`.
    #[error("recovery code is malformed")]
    MalformedRecoveryCode,

    /// The vault body could not be serialized. A bug in this crate, not a user error.
    #[error("vault body could not be encoded")]
    Encode,

    /// The operating system's random number generator refused to produce entropy.
    ///
    /// There is no fallback and there must not be one: a PRNG standing in for the OS CSPRNG is
    /// exactly the failure R-06 exists to prevent.
    #[error("the operating system's random number generator is unavailable")]
    Entropy,

    /// The KDF parameters are outside the bounds in `docs/vault-format.md` §3.3.
    #[error("Argon2id parameters are out of range")]
    KdfParams,

    /// No item in this vault has that identifier.
    ///
    /// Safe to distinguish, and it has to be: the caller is the local UI acting on a list it
    /// was just given, so the only way to reach this is a stale selection or a bug. It says
    /// nothing an attacker could not learn by opening the vault they already opened.
    #[error("no such item")]
    NoSuchItem,

    /// The item exists; it has no field with that identifier.
    #[error("no such field")]
    NoSuchField,

    /// The field exists and is not marked secret, so there is nothing to reveal.
    ///
    /// Revealing a non-secret field is refused rather than quietly succeeding, so that
    /// "reveal" means one thing in the audit log, in the IPC audit harness, and in the UI.
    /// A field whose value is already in the item list is not revealed by asking again.
    #[error("field is not secret")]
    NotSecret,

    /// An I/O failure while reading or writing the vault file.
    #[error("vault file I/O failed")]
    Io(#[from] std::io::Error),
}

/// Result alias for vault operations.
pub type Result<T> = core::result::Result<T, Error>;
