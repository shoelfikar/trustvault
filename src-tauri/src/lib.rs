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
pub mod autostart;
pub mod clipboard;
pub mod commands;
pub mod dto;
pub mod error;
// The only socket in the application — R-25, D-83. Registered as a module before any command
// calls it, which is where the N-02 boundary is visible: `trustvault-core` is a dependency of
// this crate and cannot see this file, and this file cannot see a vault.
pub mod hibp;
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
        // The only plugin in this application — D-59, and it is registered without being
        // *granted*. `capabilities/default.json` lists `core:default` alone, so the three JS
        // commands this adds (`open`, `save`, `message`) are denied to the webview and the two
        // commands in `commands::picker` are the only way a dialog opens. Note what it does
        // do to the frontend regardless of the ACL: its init script replaces `window.alert`
        // and `window.confirm`, and the replacement `confirm` returns a **promise**, so
        // `if (confirm(…))` would be true always. Nothing here calls either, and `ipc_audit.rs`
        // keeps it that way.
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            // Settings before anything else: the lock screen needs the theme, and reading it
            // after the window paints is how a themed app flashes the wrong colours.
            let (loaded, known) = commands::settings::load(&handle);
            let state = app.state::<AppState>();
            state.with(|inner| {
                inner.settings = loaded;
                inner.known_vaults = known;
                inner.last_activity = state::now_ms();
            });
            // A relaunch must land on the lock screen, not on onboarding — D-40. Nothing but
            // the path is restored: no key, no body, so the state this produces is `locked`,
            // which is the honest description of a vault the host knows about and cannot read.
            commands::settings::restore_last_vault(&state);
            // The world wins over the file: a user who removed the login entry through their
            // desktop's own startup tool has said something the settings screen must not
            // contradict.
            commands::settings::reconcile_autostart(&state);
            restore_geometry(&handle, &state);
            // S-04's instrument, and the only thing the `measure` feature does. The criterion is
            // keystroke-to-render in the **shipping** build, `CommandPalette.svelte` records it
            // with `performance.measure`, and a release build has no console to read it from.
            //
            // Opened rather than merely enabled, because "right-click and find Inspect Element"
            // is a per-platform detail that turns a measurement into a hunt.
            //
            // It is also the compile-time proof that the feature still reaches Tauri:
            // `open_devtools` exists only under `debug_assertions` or `tauri/devtools`, so a
            // `measure` build that had quietly stopped enabling devtools fails here instead of
            // launching without a console and wasting the sitting it was built for.
            #[cfg(feature = "measure")]
            if let Some(window) = handle.get_webview_window("main") {
                window.open_devtools();
            }
            autolock::start(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            let state = window.state::<AppState>();
            match event {
                // Recorded on every frame of a drag and written out on close — see
                // `record_geometry`, which says why the two are separate.
                tauri::WindowEvent::Resized(size) => {
                    let scale = window.scale_factor().unwrap_or(1.0);
                    let logical = size.to_logical::<f64>(scale);
                    commands::settings::record_geometry(
                        &state,
                        logical.width as u32,
                        logical.height as u32,
                        window.is_maximized().unwrap_or(false),
                    );
                }
                tauri::WindowEvent::CloseRequested { .. } => {
                    commands::settings::persist_geometry(window.app_handle(), &state);
                }
                _ => {}
            }
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
            commands::generator::copy_generated,
            commands::totp::totp_preview,
            // The picker — D-59. Ambient because it reads no vault, and it returns a path
            // rather than the file, which is the whole reason it is a host command.
            commands::picker::pick_import_file,
            commands::picker::pick_vault_file,
            commands::picker::pick_new_vault_path,
            // Multi-vault — ambient, because the switcher's whole job is to be usable while
            // nothing is unlocked. None of the four returns anything from inside a vault.
            commands::vault::list_vaults,
            commands::vault::switch_vault,
            commands::vault::forget_vault,
            commands::vault::delete_vault,
            // Vault-class — require an unlocked vault, return no secret.
            commands::vault::unlock,
            commands::vault::lock,
            commands::vault::set_profile,
            commands::items::list_items,
            commands::items::get_item,
            commands::items::copy_field,
            commands::items::add_item,
            commands::items::update_item,
            commands::items::delete_item,
            commands::search::search_items,
            commands::totp::totp_code,
            commands::import::import_preview,
            commands::import::import_commit,
            // Watchtower — both halves (D-76). The second is the one command in this
            // application that opens a socket, and it opens none until `breach_check_enabled`
            // says so: with the setting off it returns before a client is constructed, which is
            // what makes S-10 a command nobody called rather than a branch nobody took.
            commands::watchtower::watchtower_scan,
            commands::watchtower::watchtower_breach_check,
            // Sanctioned — exactly four, each returning exactly one secret.
            // The fourth arrived with D-44; a fifth is a decision log entry, not a patch.
            commands::vault::create_vault,
            commands::vault::commit_vault,
            commands::vault::unlock_recovery_kit,
            commands::items::reveal_field,
            commands::generator::generate_password,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri runtime failed to start");
}

/// Reopens the window where the last session left it — R-27.
///
/// Best effort throughout, and deliberately so: every failure here costs a window of the
/// default size, and refusing to start because a monitor was unplugged is not a trade this
/// application makes. The **position** is not restored, only the size — a remembered position
/// on a display that is no longer attached opens the window off-screen, which looks exactly
/// like the app failing to launch and cannot be fixed by the user without editing a file.
fn restore_geometry(app: &tauri::AppHandle, state: &AppState) {
    let Some(settings) = state.with(|inner| inner.settings.clone()) else {
        return;
    };
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.set_size(tauri::LogicalSize::new(
        f64::from(settings.window_width),
        f64::from(settings.window_height),
    ));
    if settings.window_maximized {
        let _ = window.maximize();
    }
}
