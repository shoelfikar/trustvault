//! The Tauri command layer, implementing `docs/ipc-contract.md`.
//!
//! The core stays unaware of Tauri (N-02): nothing in `trustvault-core` imports anything from
//! this crate, and the dependency arrow points one way only. What lives here is the boundary
//! itself — eliding, guarding, and the two timers the frontend must not be trusted to run.
//!
//! Every command belongs to exactly one class from the contract's §2.1, and the class is
//! written above it. There are **four** sanctioned commands, the ones that may return
//! plaintext, and the fourth arrived as a decision log entry rather than as a patch — D-44,
//! `generate_password`. A fifth costs the same.

pub mod generator;
pub mod import;
pub mod items;
pub mod search;
pub mod settings;
pub mod strength;
pub mod totp;
pub mod vault;

use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, Inner};
use trustvault_core::Vault;

/// Writes the open vault to the file it came from.
///
/// **Every mutation calls this before returning**, so "the command succeeded" and "it is in
/// the vault" are the same event. A mutation that lived in memory until some later save is how
/// a crash loses the item the user just carefully typed — and the user has no way to tell the
/// two states apart, because both look like a saved item on screen.
///
/// It also flushes the buffered audit tail, which is what D-31 means by "on save".
///
/// A missing path with a vault open is not reachable from the UI — every path that opens a
/// vault records where it came from — so it is reported as a bug rather than given a message
/// that implies the user did something.
pub fn save_open_vault(vault: &mut Vault, inner: &Inner) -> IpcResult<()> {
    let Some(path) = inner.path.as_ref() else {
        return Err(IpcError::new(ErrorKind::Internal));
    };
    vault.save_to(path)?;
    Ok(())
}

/// Runs `f` against the open vault, or fails with `locked`.
///
/// **Every vault-class and sanctioned command goes through here.** That is the mechanism
/// behind "the core's lock state is authoritative": there is no other path to the `Vault`, so
/// a command cannot accidentally hold a stale handle across a lock. The contract's §9 check 6
/// calls every one of them against a locked state to prove it.
pub fn with_vault<T>(
    state: &AppState,
    f: impl FnOnce(&mut Vault, &mut Inner) -> IpcResult<T>,
) -> IpcResult<T> {
    state.touch();
    state
        .with(|inner| {
            // The vault is moved out and put back so `f` can see the rest of `Inner` too --
            // it needs the settings to know whether to audit. Taking it means a panic inside
            // `f` leaves the state locked rather than half-modified, which is the direction
            // this failure should fall.
            let Some(mut vault) = inner.vault.take() else {
                return Err(IpcError::locked());
            };
            let result = f(&mut vault, inner);
            inner.vault = Some(vault);
            result
        })
        .unwrap_or_else(|| Err(IpcError::locked()))
}
