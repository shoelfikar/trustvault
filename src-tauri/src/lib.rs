//! Tauri host process for TrustVault.
//!
//! This layer is the *only* thing allowed to hold both a `trustvault_core` handle and a
//! channel to the webview. The rules it enforces are specified in `docs/ipc-contract.md`,
//! which was written before this module had a single command in it — the same order Phase 1
//! used for the vault format, and for the same reason.
//!
//! The dependency arrow points one way: `trustvault-core` knows nothing about Tauri (N-02),
//! and CI fails the build if that stops being true.

pub mod autolock;
pub mod clipboard;
pub mod commands;
pub mod dto;
pub mod error;
pub mod state;

use tauri::Manager as _;

use state::AppState;

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
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            // Settings before anything else: the lock screen needs the theme, and reading it
            // after the window paints is how a themed app flashes the wrong colours.
            let loaded = commands::settings::load(&handle);
            let state = app.state::<AppState>();
            state.with(|inner| {
                inner.settings = loaded;
                inner.last_activity = state::now_ms();
            });
            // A relaunch must land on the lock screen, not on onboarding — D-40. Nothing but
            // the path is restored: no key, no body, so the state this produces is `locked`,
            // which is the honest description of a vault the host knows about and cannot read.
            commands::settings::restore_last_vault(&state);
            autolock::start(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Ambient — callable with no vault open.
            commands::vault::build_info,
            commands::vault::vault_status,
            commands::vault::default_vault_path,
            commands::vault::calibrate_kdf,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::strength::score_password,
            // Vault-class — require an unlocked vault, return no secret.
            commands::vault::unlock,
            commands::vault::lock,
            commands::items::list_items,
            commands::items::get_item,
            commands::items::copy_field,
            commands::items::add_item,
            commands::items::update_item,
            commands::items::delete_item,
            commands::import::import_preview,
            commands::import::import_commit,
            // Sanctioned — exactly three, each returning exactly one secret.
            // Adding a fourth is a decision log entry, not a patch.
            commands::vault::create_vault,
            commands::vault::unlock_recovery_kit,
            commands::items::reveal_field,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri runtime failed to start");
}
