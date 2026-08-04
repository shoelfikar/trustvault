//! Settings and their persistence — `docs/ipc-contract.md` §6.3, D-33.
//!
//! Plain JSON in the app config directory, not in the sealed body. `theme` must be readable
//! before any vault is open or the lock screen renders in the wrong colours for as long as the
//! unlock takes; none of the four values is a secret. What *is* sensitive — the audit log
//! itself — stays inside the vault, and only the switch that governs it lives here.

use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::error::IpcResult;
use crate::state::{AppState, Settings};

/// Where the settings file lives, or `None` if the platform will not say.
fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

/// Reads settings from disk, falling back to the defaults.
///
/// A missing file is the normal first-run case, and a **corrupt** file is treated the same
/// way: the defaults are safe by construction — audit off, auto-lock on, a short clipboard
/// window — so failing back to them is a safe failure, and refusing to start because a
/// preferences file was truncated is not.
pub fn load(app: &AppHandle) -> Settings {
    settings_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Writes settings to disk. Best effort — the process keeps the values in memory regardless.
fn persist(app: &AppHandle, settings: &Settings) {
    let Some(path) = settings_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, text);
    }
}

/// **Ambient.** The current settings.
///
/// Ambient rather than vault-class on purpose: the lock screen needs the theme, and it runs
/// before anything is unlocked.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.with(|inner| inner.settings).unwrap_or_default()
}

/// **Ambient.** Replaces the settings and writes them out.
///
/// Takes the whole struct rather than a patch. A patch shape needs every field optional, and
/// an optional boolean is how a setting gets silently reset by a caller that omitted it.
#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> IpcResult<Settings> {
    let stored = state
        .with(|inner| {
            inner.settings = settings;
            inner.settings
        })
        .unwrap_or(settings);
    persist(&app, &stored);
    Ok(stored)
}
