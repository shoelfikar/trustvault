//! The error type that crosses IPC, specified in `docs/ipc-contract.md` §4.
//!
//! The whole job of this module is to carry the core's errors outward **without widening what
//! they distinguish**. `trustvault_core::Error::Unreadable` deliberately covers both a wrong
//! master password and a corrupted file (R-03); a helpful `wrong_password` kind added here
//! would throw that away for free, at a layer where it looks like an improvement.

use serde::Serialize;
use trustvault_core::Error as CoreError;

/// What went wrong, in the vocabulary `docs/ipc-contract.md` §4 defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The file is not a vault.
    NotAVault,
    /// The file is a vault from a newer format version.
    UnsupportedVersion,
    /// Wrong password **or** damaged file — deliberately indistinguishable (R-03).
    Unreadable,
    /// The recovery code is not 24 characters of the RFC 4648 base32 alphabet.
    MalformedRecoveryCode,
    /// No vault is open.
    Locked,
    /// No item with that identifier.
    NoSuchItem,
    /// No field with that identifier.
    NoSuchField,
    /// The field is not marked secret, so there is nothing to reveal.
    NotSecret,
    /// The TOTP seed will not decode, or carries parameters this build cannot generate.
    MalformedTotpSecret,
    /// The system clipboard refused the write.
    Clipboard,
    /// The file offered for import is not an unencrypted Bitwarden JSON export.
    NotImportable,
    /// The name typed into the vault-deletion dialog is not the vault's name — R-18.
    ConfirmationMismatch,
    /// A file this application had to read or write would not. The vault file, or — since
    /// `launch_at_login` — the OS's own login-time launcher entry.
    Io,
    /// A bug in this application or a broken machine.
    Internal,
}

/// The error payload as the webview receives it.
#[derive(Debug, Clone, Serialize)]
pub struct IpcError {
    /// The machine-readable kind.
    pub kind: ErrorKind,
    /// Display text. Never contains a secret, a field value, or a field label.
    pub message: String,
}

impl IpcError {
    /// Builds an error with the message that belongs to `kind`.
    ///
    /// Messages are fixed per kind rather than composed from the failure. A composed message
    /// is how the value that failed to parse ends up in a log file.
    pub fn new(kind: ErrorKind) -> Self {
        let message = match kind {
            ErrorKind::NotAVault => "That file is not a TrustVault vault.",
            ErrorKind::UnsupportedVersion => {
                "This vault was written by a newer version of TrustVault."
            }
            // One message for two causes, matching one error type for two causes. Reworded,
            // this sentence must keep saying both — a user who mistyped and a user with a
            // damaged file are told the same thing on purpose.
            ErrorKind::Unreadable => {
                "Could not unlock. Check the password, or the file may be damaged."
            }
            ErrorKind::MalformedRecoveryCode => {
                "That recovery code is not in the right format. It is 24 characters in six groups of four."
            }
            ErrorKind::Locked => "The vault is locked.",
            ErrorKind::NoSuchItem => "That item no longer exists.",
            ErrorKind::NoSuchField => "That field no longer exists.",
            ErrorKind::NotSecret => "That field is not hidden, so there is nothing to reveal.",
            // Said while the user is still looking at the field, which is the only moment it
            // can be acted on. It names both shapes we accept, because the commonest cause is
            // a paste of the wrong half of a setup page.
            ErrorKind::MalformedTotpSecret => {
                "That is not a valid 2FA secret. Paste the base32 key or the whole otpauth:// link."
            }
            ErrorKind::Clipboard => "Could not write to the clipboard.",
            // The only error whose cause the user can do something about by going back to the
            // other application, so it says which application and which export.
            // Says what to type rather than only that it was wrong. The user is looking at the
            // name on the same screen, so the failure is almost always a typo or the wrong
            // vault selected -- and this is the one error in the product whose *success* is
            // irreversible, so being unhelpful here has no upside.
            ErrorKind::ConfirmationMismatch => {
                "That is not this vault's name. Type it exactly as it is shown above."
            }
            ErrorKind::NotImportable => {
                "That file is not an unencrypted Bitwarden JSON export. In Bitwarden, choose                  Export vault and the .json format, without a password."
            }
            // Said "the vault file" until 2026-08-06, when `launch_at_login` became the first
            // `io` that has nothing to do with a vault: the write it fails on is a desktop
            // entry or a registry value. A password manager telling a user their vault file
            // could not be written, when the vault is fine and a login toggle is what failed,
            // is the same class of wrong copy D-49 found in the delete dialog — one sentence,
            // read at the moment it matters, describing something that did not happen.
            ErrorKind::Io => "Could not read or write a file on this computer.",
            ErrorKind::Internal => "Something went wrong inside TrustVault.",
        };
        Self {
            kind,
            message: message.to_owned(),
        }
    }

    /// The vault is locked — the answer every vault-class command gives when it is.
    pub fn locked() -> Self {
        Self::new(ErrorKind::Locked)
    }
}

impl From<CoreError> for IpcError {
    fn from(error: CoreError) -> Self {
        // Exhaustive on purpose, with no catch-all arm: `CoreError` is `#[non_exhaustive]`,
        // so a new variant there must be routed here deliberately. A `_ => Internal` arm
        // would silently swallow a future variant that deserved its own answer.
        let kind = match error {
            CoreError::NotAVault => ErrorKind::NotAVault,
            CoreError::UnsupportedVersion { .. } => ErrorKind::UnsupportedVersion,
            CoreError::Unreadable => ErrorKind::Unreadable,
            CoreError::MalformedRecoveryCode => ErrorKind::MalformedRecoveryCode,
            CoreError::NoSuchItem => ErrorKind::NoSuchItem,
            CoreError::NoSuchField => ErrorKind::NoSuchField,
            CoreError::NotSecret => ErrorKind::NotSecret,
            CoreError::MalformedTotpSecret => ErrorKind::MalformedTotpSecret,
            CoreError::NotImportable => ErrorKind::NotImportable,
            CoreError::Io(_) => ErrorKind::Io,
            // Encode, Entropy and KdfParams are bugs or a broken machine, not user errors,
            // and none of them should reach a user with a distinguishing message.
            _ => ErrorKind::Internal,
        };
        Self::new(kind)
    }
}

/// Result alias for every Tauri command.
pub type IpcResult<T> = Result<T, IpcError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wrong_password_and_a_corrupt_file_map_to_one_kind() {
        // R-03 at this layer. The core makes them indistinguishable; the only way this
        // property dies is if someone splits them here, so it is pinned by a test.
        assert_eq!(
            IpcError::from(CoreError::Unreadable).kind,
            ErrorKind::Unreadable
        );
        let message = IpcError::new(ErrorKind::Unreadable).message;
        assert!(message.contains("password"), "names one cause");
        assert!(message.contains("damaged"), "and the other");
    }

    #[test]
    fn a_malformed_recovery_code_stays_distinguishable() {
        // Safe, and only because of when it happens: caught before any key material exists,
        // so reporting it leaks nothing about the vault.
        assert_eq!(
            IpcError::from(CoreError::MalformedRecoveryCode).kind,
            ErrorKind::MalformedRecoveryCode
        );
    }

    #[test]
    fn internal_failures_do_not_get_their_own_message() {
        for error in [CoreError::Encode, CoreError::Entropy, CoreError::KdfParams] {
            assert_eq!(IpcError::from(error).kind, ErrorKind::Internal);
        }
    }
}
