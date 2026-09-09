//! Importing a Bitwarden export — `docs/ipc-contract.md` §6.8, R-29, D-42.
//!
//! Both commands are **vault-class**: they need an unlocked vault and neither returns a
//! secret. That is not a claim about intent, it is a property of the type they return —
//! [`ImportReport`] holds counts, titles, field labels and sentences, and `trustvault-core`'s
//! own `a_report_carries_no_value_from_the_vault_it_describes` fails if a value ever reaches
//! one. The report is returned unwrapped rather than elided into a DTO for that reason: there
//! is nothing in it to elide, and a wrapper would only be a second place to keep in step.
//!
//! The path comes from the webview, which is untrusted. It is not sanitised here and does not
//! need to be: the file is opened read-only by the core, nothing is written next to it, and a
//! path that does not resolve is an `io` error the user reads as "that file is not there".

use tauri::State;

use crate::commands::{save_open_vault, with_vault};
use crate::error::IpcResult;
use crate::state::AppState;
use trustvault_core::ImportReport;

/// **Vault-class.** Reports what an import would do, and changes nothing — R-29.
#[tauri::command(rename_all = "snake_case")]
pub fn import_preview(state: State<'_, AppState>, path: String) -> IpcResult<ImportReport> {
    import_preview_inner(&state, &path)
}

/// The body of [`import_preview`], reachable without a Tauri runtime.
pub fn import_preview_inner(state: &AppState, path: &str) -> IpcResult<ImportReport> {
    with_vault(state, |vault, _| Ok(vault.preview_bitwarden(path)?))
}

/// **Vault-class.** Imports the export as one transaction, then saves — R-29.
///
/// It re-reads and re-parses the file rather than taking the preview's result, which is
/// §6.8's rule and not an oversight: holding the parse would keep a full plaintext copy of a
/// foreign vault alive in this process for as long as the user reads the preview — outside the
/// vault, and so outside everything that locks. The cost is a real race, and it is why the
/// report returned *here* is the authoritative one.
#[tauri::command(rename_all = "snake_case")]
pub fn import_commit(state: State<'_, AppState>, path: String) -> IpcResult<ImportReport> {
    import_commit_inner(&state, &path)
}

/// The body of [`import_commit`], reachable without a Tauri runtime.
pub fn import_commit_inner(state: &AppState, path: &str) -> IpcResult<ImportReport> {
    with_vault(state, |vault, inner| {
        let report = vault.import_bitwarden(path)?;
        // The save is inside the same call as the import for the reason every mutation is:
        // "the command succeeded" and "it is in the vault" have to be one event, or a crash
        // loses an import the user has already been told landed.
        save_open_vault(vault, inner)?;
        Ok(report)
    })
}
