//! Host-process state: the open vault, the settings, and the lock clock.
//!
//! **This module is where "the lock state is authoritative" is either true or a slogan.** The
//! vault lives here as an `Option<Vault>`; `None` *is* locked. The webview holds no handle to
//! it, cannot construct one, and a reload leaves this struct untouched — which is the property
//! `docs/ipc-contract.md` §9 check 6 exists to regression-test.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use trustvault_core::Vault;

use crate::dto::{VaultState, VaultStatus};

/// Milliseconds since the Unix epoch, UTC. The same clock the vault body uses.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// User settings — `docs/ipc-contract.md` §6.3.
///
/// Stored as plain JSON beside the app's config, **not** in the sealed body (D-33). None of
/// these four values is a secret, and `theme` has to be readable before any vault is open or
/// the lock screen renders in the wrong colours for the time it takes to unlock.
// `Copy` was dropped when `last_vault_path` arrived: a `String` cannot be copied, and the
// alternative — a fixed-size path — is not a thing on any platform this ships to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Follow the OS, or override it — R-28.
    pub theme: Theme,
    /// Idle seconds before the vault locks itself — R-09.
    pub auto_lock_seconds: u64,
    /// Seconds a copied secret stays on the clipboard — R-14.
    pub clipboard_clear_seconds: u64,
    /// Whether reveals are recorded — R-13. **Off by default** (D-31).
    pub audit_log_enabled: bool,
    /// Sidebar width in px, clamped to `MASTER.md` §4's 180–320 by the frontend.
    pub sidebar_width: u32,
    /// Item-list width in px, clamped to §4's 240–460.
    ///
    /// Pane widths live with the settings rather than in a separate window-state file because
    /// `MASTER.md` §10 asks for them to be persisted and they are no more secret than the
    /// theme. One file, one format — D-33's argument, applied again.
    pub list_width: u32,
    /// The vault opened last, so a relaunch lands on the **lock screen** rather than
    /// onboarding — D-40.
    ///
    /// Not a secret: it is a path to a file whose whole security lies in being encrypted, and
    /// it sits in a config directory beside the theme. What it *is* is the only thing that
    /// makes "quit and relaunch" work at all — the host keeps nothing in memory across a quit,
    /// so without this the app cannot name the vault it is asking the user to unlock.
    pub last_vault_path: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            auto_lock_seconds: 300,
            clipboard_clear_seconds: 12,
            audit_log_enabled: false,
            sidebar_width: 232,
            list_width: 300,
            last_vault_path: None,
        }
    }
}

/// Theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Follow the operating system.
    #[default]
    System,
    /// Always light.
    Light,
    /// Always dark.
    Dark,
}

/// Why the vault locked, carried on the `vault-locked` event.
///
/// The reason is the difference between a user thinking the app crashed and a user knowing
/// the timeout fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LockReason {
    /// The user asked.
    Manual,
    /// The idle timer expired — R-09.
    Timeout,
    /// The machine appears to have slept — R-09, and see `auto_lock` for what "appears" means.
    OsSleep,
}

/// Everything the host process knows.
#[derive(Debug, Default)]
pub struct Inner {
    /// The open vault. `None` is the locked state, and it is the only representation of it.
    pub vault: Option<Vault>,
    /// The vault file, remembered across a lock so the lock screen knows what to unlock.
    pub path: Option<PathBuf>,
    /// Settings, loaded at start-up.
    pub settings: Settings,
    /// When a command last ran, for the idle timer. Unix milliseconds.
    pub last_activity: i64,
    /// Bumped on every lock and unlock, so a timer that fires late cannot act on a stale view.
    ///
    /// Without this, a 10-second remask scheduled before a lock would emit `field-remasked`
    /// for an item in a vault that is no longer open — harmless today, and exactly the kind of
    /// thing that stops being harmless when someone reuses the event.
    pub generation: u64,
}

/// The Tauri-managed state handle.
#[derive(Debug, Default)]
pub struct AppState {
    inner: Mutex<Inner>,
}

impl AppState {
    /// Runs `f` against the state.
    ///
    /// A poisoned mutex is treated as fatal to the *operation*, not recovered from: the lock
    /// is only ever held across small synchronous sections, so a panic inside one means an
    /// invariant already broke.
    pub fn with<T>(&self, f: impl FnOnce(&mut Inner) -> T) -> Option<T> {
        let mut guard = self.inner.lock().ok()?;
        Some(f(&mut guard))
    }

    /// Records that the user did something, for the idle timer.
    pub fn touch(&self) {
        self.with(|inner| inner.last_activity = now_ms());
    }

    /// Locks the vault, zeroizing the master key by dropping it.
    ///
    /// Returns whether anything was open, so a caller can decide whether an event is worth
    /// emitting. Locking a locked vault is a success and does nothing — the failure mode of a
    /// lock command that errors is a user hammering it during a panic.
    pub fn lock(&self) -> bool {
        self.with(|inner| {
            inner.generation = inner.generation.wrapping_add(1);
            // Dropping the Vault zeroizes the master key. The path is kept: the lock screen
            // needs to know what it is about to unlock.
            inner.vault.take().is_some()
        })
        .unwrap_or(false)
    }

    /// The status shape the whole frontend routes on.
    pub fn status(&self) -> VaultStatus {
        self.with(|inner| {
            let state = match (&inner.vault, &inner.path) {
                (Some(_), _) => VaultState::Unlocked,
                (None, Some(_)) => VaultState::Locked,
                (None, None) => VaultState::NoVault,
            };
            let display_name = match (&inner.vault, &inner.path) {
                // Open: the real name, out of the body that is now decrypted.
                (Some(vault), _) => vault.name().to_owned(),
                // Locked: the file stem, because the real name is sealed inside the body and
                // there is no key to read it with. Not a fallback — the only thing knowable.
                (None, Some(path)) => stem_of(path),
                (None, None) => String::new(),
            };
            VaultStatus {
                state,
                path: inner.path.as_ref().map(|p| p.display().to_string()),
                display_name,
                item_count: inner.vault.as_ref().map(|v| v.items().count()),
            }
        })
        .unwrap_or_else(|| VaultStatus {
            state: VaultState::NoVault,
            path: None,
            display_name: String::new(),
            item_count: None,
        })
    }
}

/// The file stem, which is all a locked vault can be called.
fn stem_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use trustvault_core::{ItemKind, KdfParams};

    fn open_vault() -> Vault {
        let (mut vault, _) = Vault::create("Personal Vault", "correct horse", KdfParams::TESTING)
            .expect("test parameters are valid");
        vault.add_item(ItemKind::Login, "GitHub");
        vault
    }

    #[test]
    fn a_fresh_state_has_no_vault() {
        let status = AppState::default().status();
        assert_eq!(status.state, VaultState::NoVault);
        assert_eq!(status.item_count, None);
    }

    #[test]
    fn a_locked_vault_is_named_by_its_file_stem_not_its_real_name() {
        // The wrinkle that only shows up on the lock screen: "Personal Vault" lives inside
        // the sealed body, so while locked the only knowable name is "work-secrets".
        let state = AppState::default();
        state.with(|inner| {
            inner.vault = Some(open_vault());
            inner.path = Some(PathBuf::from("/home/someone/work-secrets.tvault"));
        });
        assert_eq!(state.status().display_name, "Personal Vault");

        state.lock();

        let status = state.status();
        assert_eq!(status.state, VaultState::Locked);
        assert_eq!(status.display_name, "work-secrets");
        assert_eq!(
            status.item_count, None,
            "an item count cannot be known while locked"
        );
    }

    #[test]
    fn locking_is_idempotent_and_bumps_the_generation() {
        let state = AppState::default();
        state.with(|inner| inner.vault = Some(open_vault()));

        assert!(state.lock(), "something was open");
        assert!(!state.lock(), "nothing was, and that is still a success");

        let generation = state
            .with(|inner| inner.generation)
            .expect("state readable");
        assert_eq!(
            generation, 2,
            "both locks bumped it, so stale timers are stale"
        );
    }

    #[test]
    fn the_audit_setting_is_off_by_default() {
        // D-31. Written as a test because a default is exactly the kind of thing that gets
        // flipped by someone making the feature easier to demonstrate.
        assert!(!Settings::default().audit_log_enabled);
    }
}
