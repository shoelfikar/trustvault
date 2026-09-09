//! Settings and their persistence — `docs/ipc-contract.md` §6.3, D-33.
//!
//! Plain JSON in the app config directory, not in the sealed body. `theme` must be readable
//! before any vault is open or the lock screen renders in the wrong colours for as long as the
//! unlock takes; not one of the values here is a secret. What *is* sensitive — the audit log
//! itself — stays inside the vault, and only the switch that governs it lives here.
//!
//! Four fields are **host-owned**: `last_vault_path` (D-40) and the three `window_*` values
//! (R-27). They are in this file because they share its lifetime and its non-secrecy, not
//! because the webview may set them, and [`merge_incoming`] is what keeps that true.
//!
//! **`known_vaults` shares the file but not the struct** — the contract's §6.3 rule, and this
//! module is where it holds: [`Stored`] is what goes on disk, `Settings` is what crosses IPC,
//! and the list's read path is `list_vaults`, which derives a display name per entry.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, KnownVault, Settings};

/// The shape of `settings.json`: the user's settings **plus the host's own bookkeeping**.
///
/// One file, per D-33 — and one struct short of `Settings`, per the contract. `known_vaults`
/// never crosses IPC in this shape: the webview asks `list_vaults`, which turns each entry
/// into a `VaultRef` with a display name derived for it. Flattening keeps the file readable as
/// a flat object rather than nesting every setting one level deeper for the sake of one key.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Stored {
    #[serde(flatten)]
    settings: Settings,
    known_vaults: Vec<KnownVault>,
}

/// Where the settings file lives, or `None` if the platform will not say.
fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

/// Reads settings and the known-vault list from disk, falling back to the defaults.
///
/// A missing file is the normal first-run case, and a **corrupt** file is treated the same
/// way: the defaults are safe by construction — audit off, auto-lock on, a short clipboard
/// window, no vaults known — so failing back to them is a safe failure, and refusing to start
/// because a preferences file was truncated is not.
pub fn load(app: &AppHandle) -> (Settings, Vec<KnownVault>) {
    let stored: Stored = settings_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    (stored.settings, stored.known_vaults)
}

/// Writes the remembered vault path and the known-vault list out, so the **next launch** can
/// offer to unlock what this one had open.
///
/// The values themselves are recorded by the command bodies, which is where every path that
/// leaves a vault open passes; this only puts them on disk, because that is the half needing
/// an `AppHandle`. Best effort: failing to write costs the user one "Open vault file…" on the
/// next launch, not access.
pub fn remember_vault(app: &AppHandle, state: &AppState) {
    persist(app, state);
}

/// Points the host at the vault it had open last, so a relaunch lands on the lock screen.
///
/// Sets only the *path*, never a key: the result is `VaultState::Locked`, which is precisely
/// the state a relaunch should produce. Takes `&AppState` rather than an `AppHandle` so the
/// session harness can perform a relaunch without a Tauri runtime.
///
/// A remembered path that no longer resolves — a vault on an external drive, a file moved —
/// yields `no_vault` and onboarding, and the setting is **left alone** rather than cleared.
/// Forgetting the vault because a USB stick was unplugged once is the wrong direction for
/// this to fail.
pub fn restore_last_vault(state: &AppState) {
    state.with(|inner| {
        if let Some(remembered) = inner.settings.last_vault_path.as_ref() {
            let path = PathBuf::from(remembered);
            if path.is_file() {
                inner.path = Some(path);
            }
        }
    });
}

/// Writes the whole stored file. Best effort — the process keeps the values in memory anyway.
///
/// Takes the state rather than a `Settings`, which is the change that made `known_vaults`
/// safe to add: a signature that accepted only the settings would need every caller to
/// remember to write the list too, and the one that forgot would silently drop a vault.
fn persist(app: &AppHandle, state: &AppState) {
    let Some(path) = settings_path(app) else {
        return;
    };
    let Some(stored) = state.with(|inner| Stored {
        settings: inner.settings.clone(),
        known_vaults: inner.known_vaults.clone(),
    }) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(&stored) {
        let _ = fs::write(path, text);
    }
}

/// **Ambient.** The current settings.
///
/// Ambient rather than vault-class on purpose: the lock screen needs the theme, and it runs
/// before anything is unlocked.
#[tauri::command(rename_all = "snake_case")]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state
        .with(|inner| inner.settings.clone())
        .unwrap_or_default()
}

/// **Ambient.** Replaces the settings and writes them out.
///
/// Takes the whole struct rather than a patch. A patch shape needs every field optional, and
/// an optional boolean is how a setting gets silently reset by a caller that omitted it.
///
/// **One field can refuse to be set**, and it is handled before anything is stored:
/// `launch_at_login` is a write to the OS, and the contract requires that a platform which
/// will not take it yields `io` with the stored value untouched. Doing it first is what makes
/// "untouched" true — persisting and then discovering the registration failed would leave a
/// settings file claiming something about the machine that is not so.
#[tauri::command(rename_all = "snake_case")]
pub fn set_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> IpcResult<Settings> {
    let current_autostart = state.with(|inner| inner.settings.launch_at_login);
    if current_autostart != Some(settings.launch_at_login) {
        crate::autostart::set(settings.launch_at_login)
            .map_err(|_| IpcError::new(ErrorKind::Io))?;
    }

    let stored = state
        .with(|inner| {
            inner.settings = merge_incoming(&inner.settings, settings.clone());
            inner.settings.clone()
        })
        .unwrap_or(settings);
    persist(&app, &state);
    Ok(stored)
}

/// Records the window's geometry so the next launch opens where this one closed — R-27.
///
/// Host-owned, and [`merge_incoming`] is what keeps it that way: the webview holds a copy of
/// `Settings` taken when the screen opened, so a theme change after a resize would otherwise
/// send the old dimensions back and undo this.
///
/// **Not persisted here.** This runs on every frame of a window drag, and a file write per
/// frame is how a preferences file gets corrupted by a hard shutdown mid-resize. It is written
/// out by [`persist_geometry`] when the window closes, and by any `set_settings` in between.
pub fn record_geometry(state: &AppState, width: u32, height: u32, maximized: bool) {
    state.with(|inner| {
        inner.settings.window_maximized = maximized;
        // A maximized window's own dimensions are the screen's, not the size to restore to
        // when it is un-maximized. Keeping the last un-maximized size is the whole reason
        // `window_maximized` sits beside the two numbers rather than replacing them.
        if !maximized {
            inner.settings.window_width = width;
            inner.settings.window_height = height;
        }
    });
}

/// Writes the recorded geometry out. Called when the window is closing.
pub fn persist_geometry(app: &AppHandle, state: &AppState) {
    persist(app, state);
}

/// Reconciles the stored `launch_at_login` with what the OS actually has registered.
///
/// Called at start-up. A user who removed the entry through their desktop's own
/// startup-applications tool has said something, and a settings screen that goes on showing
/// "on" over it is reporting this file's memory rather than the state of the machine. The
/// world wins; the file is corrected to match it.
pub fn reconcile_autostart(state: &AppState) {
    let registered = crate::autostart::is_enabled();
    state.with(|inner| inner.settings.launch_at_login = registered);
}

/// Applies the caller's settings over the stored ones, keeping the host-owned fields.
///
/// **Four fields are host-owned**, and this function is the only thing that makes that true.
/// `Settings` is `#[serde(default)]`, so a frontend that simply does not know about a field
/// sends it absent, serde reads that as the default, and the host's value is gone — with a
/// symptom that is intermittent and a cause that is invisible. Which is why this is a named
/// function with a test rather than a line inside a closure.
///
/// - `last_vault_path` (D-40) is in `Settings` because it shares the file, not because the
///   webview may set it. Lost, the app forgets the user's vault on the next theme change.
/// - The three `window_*` values are measured by this process on every resize (R-27). The
///   webview holds a copy of `Settings` taken when the screen it is on opened, so **resize the
///   window and then change the theme** and the stale copy comes back — the window would snap
///   to its old size on the next launch, having been resized in between.
fn merge_incoming(current: &Settings, incoming: Settings) -> Settings {
    Settings {
        last_vault_path: current.last_vault_path.clone(),
        window_width: current.window_width,
        window_height: current.window_height,
        window_maximized: current.window_maximized,
        ..incoming
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Theme;

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("trustvault-{name}-{}.tvault", std::process::id()))
    }

    #[test]
    fn a_relaunch_lands_on_the_lock_screen_when_the_vault_is_still_there() {
        // D-40, and the reason the whole setting exists: the host keeps nothing across a
        // quit, so without the remembered path a user who made a vault yesterday is shown
        // onboarding today, with no way back to the file they made.
        let path = scratch("restore");
        fs::write(&path, b"not a real vault, but a real file").unwrap();

        let state = AppState::default();
        state.with(|inner| {
            inner.settings.last_vault_path = Some(path.display().to_string());
        });
        restore_last_vault(&state);

        assert_eq!(state.status().state, crate::dto::VaultState::Locked);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_vault_that_has_moved_is_not_forgotten_but_is_not_offered_either() {
        // The external-drive case. Onboarding is the right screen when the file is not
        // there, and clearing the setting would mean unplugging a USB stick once loses the
        // vault permanently -- the wrong direction for this to fail.
        let state = AppState::default();
        state.with(|inner| {
            inner.settings.last_vault_path = Some("/nowhere/gone.tvault".into());
        });
        restore_last_vault(&state);

        assert_eq!(state.status().state, crate::dto::VaultState::NoVault);
        assert_eq!(
            state.with(|inner| inner.settings.last_vault_path.clone()),
            Some(Some("/nowhere/gone.tvault".into())),
            "still remembered for the next time the drive is plugged in"
        );
    }

    #[test]
    fn changing_the_theme_does_not_make_the_app_forget_the_vault() {
        // The trap `merge_incoming` exists for: the frontend's Settings type does not carry
        // last_vault_path, `#[serde(default)]` turns that absence into None, and a theme
        // toggle would otherwise erase it.
        let current = Settings {
            last_vault_path: Some("/home/someone/personal.tvault".into()),
            ..Settings::default()
        };
        let from_webview = Settings {
            theme: Theme::Dark,
            last_vault_path: None,
            ..Settings::default()
        };

        let merged = merge_incoming(&current, from_webview);
        assert_eq!(merged.theme, Theme::Dark, "the user's change is applied");
        assert_eq!(
            merged.last_vault_path, current.last_vault_path,
            "and the host-owned field survives it"
        );
    }

    #[test]
    fn resizing_the_window_and_then_changing_a_setting_does_not_undo_the_resize() {
        // The R-27 half of the same trap, and the one that actually bites in use: the webview
        // is holding a `Settings` from when the settings screen opened. Resize the window,
        // then flip a toggle, and without this the stale dimensions travel back and the next
        // launch opens at the size the window had two changes ago.
        let current = Settings {
            window_width: 1600,
            window_height: 900,
            window_maximized: false,
            ..Settings::default()
        };
        let from_webview = Settings {
            theme: Theme::Dark,
            ..Settings::default() // carrying the *default* 1360x864 it was handed at open
        };

        let merged = merge_incoming(&current, from_webview);
        assert_eq!(merged.theme, Theme::Dark);
        assert_eq!((merged.window_width, merged.window_height), (1600, 900));
    }

    #[test]
    fn maximizing_keeps_the_size_to_restore_to() {
        // A maximized window's own dimensions are the screen's. Recording them would mean
        // un-maximizing lands on a window the size of the display, which is not a restore.
        let state = AppState::default();
        record_geometry(&state, 1600, 900, false);
        record_geometry(&state, 3840, 2160, true);

        let settings = state.with(|inner| inner.settings.clone()).unwrap();
        assert!(settings.window_maximized);
        assert_eq!(
            (settings.window_width, settings.window_height),
            (1600, 900),
            "the last un-maximized size is what un-maximizing has to restore to"
        );
    }

    #[test]
    fn corrupt_settings_fall_back_to_the_safe_defaults() {
        // Not a new property, but nothing pinned it: the defaults are safe by construction
        // (audit off, auto-lock on, a short clipboard window), so falling back to them is a
        // safe failure. Refusing to start because a preferences file was truncated is not.
        let broken: Result<Settings, _> = serde_json::from_str("{ not json");
        assert!(broken.is_err());
        assert!(!Settings::default().audit_log_enabled);
        assert_eq!(Settings::default().auto_lock_seconds, 300);
    }
}
