//! Settings and their persistence — `docs/ipc-contract.md` §6.3, D-33.
//!
//! Plain JSON in the app config directory, not in the sealed body. `theme` must be readable
//! before any vault is open or the lock screen renders in the wrong colours for as long as the
//! unlock takes; not one of the values here is a secret. What *is* sensitive — the audit log
//! itself — stays inside the vault, and only the switch that governs it lives here.
//!
//! One field is **host-owned**: `last_vault_path` (D-40). It is in this file because it shares
//! the file's lifetime and its non-secrecy, not because the webview may set it, and
//! [`merge_incoming`] is what keeps that true.

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

/// Writes the remembered vault path out, so the **next launch** can offer to unlock it.
///
/// The value itself is recorded by the command bodies, which is where every path that leaves
/// a vault open passes; this only puts it on disk, because that is the half needing an
/// `AppHandle`. Best effort: failing to write it costs the user one "Open vault file…" on the
/// next launch, not access.
pub fn remember_vault(app: &AppHandle, state: &AppState) {
    if let Some(settings) = state.with(|inner| inner.settings.clone()) {
        persist(app, &settings);
    }
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
#[tauri::command(rename_all = "snake_case")]
pub fn set_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> IpcResult<Settings> {
    let stored = state
        .with(|inner| {
            inner.settings = merge_incoming(&inner.settings, settings.clone());
            inner.settings.clone()
        })
        .unwrap_or(settings);
    persist(&app, &stored);
    Ok(stored)
}

/// Applies the caller's settings over the stored ones, keeping the host-owned fields.
///
/// Only one field is host-owned today, and it is the reason this function exists rather than
/// an assignment: `last_vault_path` is in `Settings` because it shares the file, not because
/// the webview may set it. `Settings` is `#[serde(default)]`, so a frontend that simply does
/// not know about the field sends it as absent, serde reads that as `None`, and the app
/// forgets the user's vault the first time they change the theme. The symptom would be
/// intermittent and the cause invisible, which is why the rule is a named function with a
/// test rather than a line inside a closure.
fn merge_incoming(current: &Settings, incoming: Settings) -> Settings {
    Settings {
        last_vault_path: current.last_vault_path.clone(),
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
