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

use crate::dto::{Profile, VaultState, VaultStatus};

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
/// these values is a secret, and `theme` has to be readable before any vault is open or the
/// lock screen renders in the wrong colours for the time it takes to unlock.
///
/// **Three of these fields are host-owned**, and which ones is not a detail: `last_vault_path`
/// (D-40) and the three `window_*` values are written by this process and only *read* by the
/// webview. [`crate::commands::settings::merge_incoming`] is where that is true or merely
/// intended — the webview sends the whole struct back on every change, and it is holding a
/// copy that was current when the settings screen opened.
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
    /// How large the whole interface draws — R-21.
    ///
    /// The one setting that moves every measurement in the application at once: `tokens.css`
    /// derives every size from `--ui-scale`, so this multiplies text, rows, controls and
    /// spacing together rather than growing the type and leaving the rows where they were.
    pub ui_scale: UiScale,
    /// Whether the OS starts TrustVault at login — R-21.
    ///
    /// **The only setting with an effect outside this process**, which is why writing it can
    /// fail: it is a desktop entry, a `LaunchAgent` or a registry value depending on the
    /// platform. [`crate::autostart`] holds the three implementations, and `set_settings`
    /// refuses to store a value the platform would not take — a toggle that lies about
    /// whether the app will start tomorrow is worse than one that reports the failure now.
    pub launch_at_login: bool,
    /// Window width in px — R-27. Host-owned: only this process ever measures the window.
    pub window_width: u32,
    /// Window height in px — R-27.
    pub window_height: u32,
    /// Whether the window was maximized when it was last closed — R-27.
    ///
    /// Kept beside the size rather than replacing it, because restoring a maximized window
    /// still needs somewhere to put it when the user un-maximizes.
    pub window_maximized: bool,
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
            ui_scale: UiScale::Default,
            launch_at_login: false,
            // The dimensions in `tauri.conf.json`, repeated here rather than read from it:
            // this default only applies on a first run, and a mismatch costs one window of the
            // wrong size once. Reading the config at runtime to save that is a dependency
            // between two files for no benefit.
            window_width: 1360,
            window_height: 864,
            window_maximized: false,
        }
    }
}

/// How large the interface draws — R-21, `MASTER.md` §10.
///
/// Three steps rather than a slider, and they are the design's own: an arbitrary percentage
/// makes every layout a distinct one to have tested, and `tokens.css` names exactly these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScale {
    /// 92 %.
    Compact,
    /// 100 %.
    #[default]
    Default,
    /// 115 %.
    Large,
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

/// One vault this installation knows about — R-22.
///
/// **Not in `Settings`**, and the contract says why: its read path is `list_vaults`, which
/// derives a `display_name` per entry, and that is work `get_settings` has no business doing
/// and the webview must not do for itself. It shares the settings *file* (D-33's one store),
/// not the settings *struct*.
///
/// The name is deliberately absent from this record. A vault's real name is inside its sealed
/// body, so storing one here would mean keeping a copy that goes stale the moment a vault is
/// renamed — and a switcher listing a name the vault no longer has is worse than one showing
/// the file stem, because the stem is at least verifiably true.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownVault {
    /// Absolute path to the `.tvault` file.
    pub path: String,
    /// When it was last opened, Unix milliseconds. `None` for one that never has been.
    pub last_opened_at: Option<i64>,
}

/// A vault that has been created but not yet written to disk — R-07, R-08, D-69.
///
/// Onboarding used to write the file at the end of step 2 and show the recovery kit on step 3,
/// which meant a user who closed the window while reading the kit owned a `.tvault` whose kit
/// had never been recorded. R-07 shows it **exactly once**, there is no command to fetch it
/// again, and the remembered path (D-40) sent the next launch to a lock screen — so the vault
/// had no recovery path at all and nothing in the product knew.
///
/// So creation stops at memory and step 3's acknowledgement is what writes. This struct is that
/// gap made explicit. It holds a decrypted vault, so it is dropped by [`AppState::lock`] exactly
/// like the open one: abandoning onboarding must not leave a key alive behind a lock screen.
#[derive(Debug)]
pub struct PendingVault {
    pub vault: Vault,
    pub path: PathBuf,
}

/// Everything the host process knows.
#[derive(Debug, Default)]
pub struct Inner {
    /// The open vault. `None` is the locked state, and it is the only representation of it.
    pub vault: Option<Vault>,
    /// A created-but-unwritten vault, between onboarding steps 2 and 3 — D-69.
    ///
    /// Deliberately **not** part of [`AppState::status`]: a pending vault is not open and not
    /// locked, and letting it reach `VaultState` would route the shell at a file that does not
    /// exist yet.
    pub pending: Option<PendingVault>,
    /// The vault file, remembered across a lock so the lock screen knows what to unlock.
    pub path: Option<PathBuf>,
    /// Settings, loaded at start-up.
    pub settings: Settings,
    /// Every vault this installation has opened, most recent first — R-22.
    pub known_vaults: Vec<KnownVault>,
    /// When a command last ran, for the idle timer. Unix milliseconds.
    pub last_activity: i64,
    /// Bumped on every lock and unlock, so a timer that fires late cannot act on a stale view.
    ///
    /// Without this, a 10-second remask scheduled before a lock would emit `field-remasked`
    /// for an item in a vault that is no longer open — harmless today, and exactly the kind of
    /// thing that stops being harmless when someone reuses the event.
    pub generation: u64,
}

impl Inner {
    /// Records that `vault` is now open at `path`, with everything that has to follow from it.
    ///
    /// One function rather than the three identical blocks that were in `create_vault_inner`,
    /// `unlock_inner` and `unlock_recovery_kit_inner`, and consolidating them was not tidying:
    /// **`known_vaults` had to be updated on every path that leaves a vault open**, and a
    /// fourth such path is exactly the thing a reader adds without noticing there were three
    /// bookkeeping lines to copy. The switcher would then be missing the vault the user is
    /// looking at.
    ///
    /// The list is **most-recently-opened first**, which is the order the switcher wants and
    /// the reason `last_opened_at` is stored at all.
    pub fn opened(&mut self, vault: Vault, path: PathBuf) {
        let now = now_ms();
        let key = path.display().to_string();

        self.generation = self.generation.wrapping_add(1);
        self.vault = Some(vault);
        // D-40: the remembered path is set here, on the path that leaves a vault open, rather
        // than in the command wrapper -- the wrapper only writes it to disk.
        self.settings.last_vault_path = Some(key.clone());
        self.path = Some(path);
        self.last_activity = now;

        self.known_vaults.retain(|known| known.path != key);
        self.known_vaults.insert(
            0,
            KnownVault {
                path: key,
                last_opened_at: Some(now),
            },
        );
    }
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
            // A vault created but not yet written (D-69) holds a decrypted key exactly like an
            // open one, so it is dropped here too. The consequence is deliberate: a lock during
            // onboarding discards the half-made vault, and the user starts again — which is the
            // right end for a vault whose recovery kit was never recorded, and the whole reason
            // the file is not written until it has been.
            inner.pending = None;
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
                // Out of the body, so `None` while locked for the same reason `item_count` is
                // — D-70. There is no key to read it with, and an empty profile would read as
                // "nobody has filled this in" rather than "you cannot know yet".
                profile: inner.vault.as_ref().map(|v| Profile {
                    name: v.profile().name.clone(),
                    email: v.profile().email.clone(),
                }),
                // Out of the sealed body like the two above, so `None` while locked — and
                // `None` while unlocked too until a scan has run. §6.9.
                last_scan_at: inner.vault.as_ref().and_then(|v| v.body().last_scan_at),
                last_breach_check_at: inner
                    .vault
                    .as_ref()
                    .and_then(|v| v.body().last_breach_check_at),
            }
        })
        .unwrap_or_else(|| VaultStatus {
            state: VaultState::NoVault,
            path: None,
            display_name: String::new(),
            item_count: None,
            profile: None,
            last_scan_at: None,
            last_breach_check_at: None,
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
