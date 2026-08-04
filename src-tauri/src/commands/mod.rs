//! The Tauri command layer, implementing `docs/ipc-contract.md`.
//!
//! The core stays unaware of Tauri (N-02): nothing in `trustvault-core` imports anything from
//! this crate, and the dependency arrow points one way only. What lives here is the boundary
//! itself — eliding, guarding, and the two timers the frontend must not be trusted to run.
//!
//! Every command belongs to exactly one class from the contract's §2.1, and the class is
//! written above it. There are **three** sanctioned commands, the ones that may return
//! plaintext, and adding a fourth is a decision log entry rather than a patch.

pub mod items;
pub mod settings;
pub mod strength;
pub mod vault;

use crate::error::{IpcError, IpcResult};
use crate::state::{AppState, Inner};
use trustvault_core::Vault;

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
