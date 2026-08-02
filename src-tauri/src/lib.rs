//! Tauri host process for TrustVault.
//!
//! This layer is the *only* thing allowed to hold both a `trustvault_core` handle and a
//! channel to the webview. The rules it enforces are specified in the *Interface contract*
//! section of `trustvault-requirements.md`; the commands that implement them arrive in
//! Phase 2 (`phases/phase-2-shell-unlock.md`).
//!
//! Phase 0 wires nothing but a window. The single command below exists so the IPC path is
//! proven end to end before anything sensitive travels over it.

use trustvault_core::{EXTENSION, FORMAT_VERSION};

/// Build and format information, for the About surface and for bug reports.
///
/// Carries no vault state, which is why it is safe to expose before the security boundary
/// exists.
#[derive(serde::Serialize)]
pub struct BuildInfo {
    /// Application version, from `Cargo.toml`.
    pub version: &'static str,
    /// On-disk vault format version this build reads and writes.
    pub format_version: u16,
    /// Vault file extension.
    pub extension: &'static str,
}

#[tauri::command]
fn build_info() -> BuildInfo {
    BuildInfo {
        version: env!("CARGO_PKG_VERSION"),
        format_version: FORMAT_VERSION,
        extension: EXTENSION,
    }
}

/// Start the application.
///
/// # Panics
///
/// Panics if the Tauri runtime cannot start, which means the webview or the windowing system
/// is unavailable. There is no meaningful recovery: without a window there is no way to tell
/// the user anything, and continuing headless would leave a process that can decrypt a vault
/// with no visible owner.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![build_info])
        .run(tauri::generate_context!())
        .expect("Tauri runtime failed to start");
}
