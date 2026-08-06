//! Launch at login — R-21, `docs/ipc-contract.md` §6.3.
//!
//! `launch_at_login` is a `bool` in `Settings` and **three implementations behind it**, one per
//! platform, because there is no cross-platform place to put this. The contract states the
//! failure mode rather than leaving it to the implementation: if the platform will not take
//! the write, the command reports `io` and the stored value is left alone. A toggle that shows
//! "on" for a registration that never happened is a promise the user only discovers is empty
//! on the morning they were relying on it.
//!
//! **Written by hand rather than taken from `tauri-plugin-autostart` or `auto-launch`** —
//! D-56. Both were read first; the reasoning and what each was rejected for is in the decision
//! log rather than repeated here. What each platform's mechanism is:
//!
//! | Platform | Mechanism |
//! |----------|-----------|
//! | Linux | XDG autostart: a `.desktop` file in `$XDG_CONFIG_HOME/autostart` |
//! | macOS | A `LaunchAgent` plist in `~/Library/LaunchAgents` with `RunAtLoad` |
//! | Windows | A value under `HKCU\…\CurrentVersion\Run`, written with `reg.exe` |
//!
//! All three are **per-user**, never system-wide: registering a password manager for every
//! account on the machine needs a privilege this application never asks for, and it would
//! start the app for users who have no vault.
//!
//! Two properties hold on every platform and are what the tests pin:
//!
//! - **Enabling twice is enabling once.** The entry is written, not appended to.
//! - **Disabling when nothing is registered succeeds.** The desired state is "not registered",
//!   and it already holds — an error there would make a fresh install's Settings screen show a
//!   failure for a toggle nobody touched.

/// The identifier the entry is filed under, on all three platforms.
///
/// One constant rather than three literals: the enable and disable paths must agree on it, and
/// a disable that looks for a different name than enable wrote leaves the app starting at login
/// with the toggle showing "off" — the exact lie this module exists to prevent.
const ENTRY_NAME: &str = "trustvault";

/// The platform would not take the write.
///
/// Deliberately carries **nothing**. What went wrong is a platform detail the settings screen
/// has no use for, and `docs/ipc-contract.md` §4 makes every message fixed per kind for the
/// same reason: a composed message is how the thing that failed ends up in a log file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refused;

/// Registers or unregisters TrustVault with the OS's login-time launcher.
///
/// `Err(Refused)` means the platform would not take the write and the caller must **not**
/// store the value — that is the contract's rule, not this module's preference.
pub fn set(enabled: bool) -> Result<(), Refused> {
    let exe = std::env::current_exe().map_err(|_| Refused)?;
    platform::set(enabled, &exe)
}

/// Whether the OS currently starts TrustVault at login.
///
/// Read at start-up so the stored setting can be **reconciled** with the world: a user who
/// removed the entry through their desktop's own startup-applications tool has said something,
/// and a settings screen that keeps showing "on" over it is reporting its own memory rather
/// than the state of the machine.
pub fn is_enabled() -> bool {
    platform::is_enabled()
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{ENTRY_NAME, Refused};
    use std::fs;
    use std::path::PathBuf;

    /// `$XDG_CONFIG_HOME/autostart/trustvault.desktop`, per the XDG autostart spec.
    fn entry() -> Option<PathBuf> {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(
            config
                .join("autostart")
                .join(format!("{ENTRY_NAME}.desktop")),
        )
    }

    pub fn set(enabled: bool, exe: &std::path::Path) -> Result<(), Refused> {
        let path = entry().ok_or(Refused)?;
        if !enabled {
            // A missing file is the desired state already reached, not a failure.
            return match fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(_) => Err(Refused),
            };
        }
        fs::create_dir_all(path.parent().ok_or(Refused)?).map_err(|_| Refused)?;
        // `Exec` is unquoted, which the spec allows for a path with no spaces and which every
        // installed location this ships to is. Quoting rules in a desktop entry are their own
        // small specification -- backslash escapes inside quotes, reserved characters -- and
        // getting them subtly wrong produces an entry the session silently ignores at login.
        let entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=TrustVault\n\
             Comment=Offline password manager\n\
             Exec={}\n\
             Terminal=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe.display()
        );
        fs::write(&path, entry).map_err(|_| Refused)
    }

    pub fn is_enabled() -> bool {
        entry().is_some_and(|path| path.is_file())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{ENTRY_NAME, Refused};
    use std::fs;
    use std::path::PathBuf;

    /// `~/Library/LaunchAgents/trustvault.plist`.
    ///
    /// A `LaunchAgent` rather than a Login Item, because a Login Item is added through
    /// AppleScript — a subprocess running a script interpreter, from a process holding
    /// decrypted secrets, to write a file this can write directly.
    fn entry() -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        Some(
            home.join("Library")
                .join("LaunchAgents")
                .join(format!("{ENTRY_NAME}.plist")),
        )
    }

    pub fn set(enabled: bool, exe: &std::path::Path) -> Result<(), Refused> {
        let path = entry().ok_or(Refused)?;
        if !enabled {
            return match fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(_) => Err(Refused),
            };
        }
        fs::create_dir_all(path.parent().ok_or(Refused)?).map_err(|_| Refused)?;
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\t<string>{ENTRY_NAME}</string>\n\
             \t<key>ProgramArguments</key>\n\t<array>\n\t\t<string>{}</string>\n\t</array>\n\
             \t<key>RunAtLoad</key>\n\t<true/>\n\
             </dict>\n\
             </plist>\n",
            exe.display()
        );
        fs::write(&path, plist).map_err(|_| Refused)
    }

    pub fn is_enabled() -> bool {
        entry().is_some_and(|path| path.is_file())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{ENTRY_NAME, Refused};
    use std::process::Command;

    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    /// `reg.exe` rather than a registry crate.
    ///
    /// One subprocess with fixed arguments, against a binary that ships with the OS, is a
    /// smaller thing to justify than a crate that links the Windows API into a process holding
    /// decrypted secrets — and it is only reached when the user changes this one toggle. The
    /// Startup-folder alternative was rejected: a shortcut there needs COM, and a `.cmd`
    /// instead flashes a console window at every login.
    fn reg(args: &[&str]) -> Result<bool, Refused> {
        let mut command = Command::new("reg.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW: without it every toggle flashes a console window.
            command.creation_flags(0x0800_0000);
        }
        command
            .args(args)
            .output()
            .map(|output| output.status.success())
            .map_err(|_| Refused)
    }

    pub fn set(enabled: bool, exe: &std::path::Path) -> Result<(), Refused> {
        let ok = if enabled {
            // `/f` overwrites, so enabling twice is enabling once rather than an error.
            reg(&[
                "add",
                RUN_KEY,
                "/v",
                ENTRY_NAME,
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\"", exe.display()),
                "/f",
            ])?
        } else {
            // A value that is not there is the desired state already reached. `reg delete`
            // reports failure for it, so absence is checked rather than inferred from status.
            if !is_enabled() {
                return Ok(());
            }
            reg(&["delete", RUN_KEY, "/v", ENTRY_NAME, "/f"])?
        };
        if ok { Ok(()) } else { Err(Refused) }
    }

    pub fn is_enabled() -> bool {
        reg(&["query", RUN_KEY, "/v", ENTRY_NAME]).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two properties that hold on every platform, exercised on the one CI runs.
    ///
    /// Deliberately not asserting *where* the entry lands: the path is the platform's business
    /// and pinning it here would make the test a copy of the implementation. What is asserted
    /// is the pair of round-trips the settings screen depends on being true.
    #[test]
    #[cfg_attr(
        not(target_os = "linux"),
        ignore = "writes into the real user profile; Linux is the CI platform"
    )]
    fn enabling_is_idempotent_and_disabling_an_absent_entry_succeeds() {
        // The starting state is whatever the developer's machine is in, and it is restored at
        // the end: a test that leaves the author's own session starting an app is a rude test.
        let was = is_enabled();

        assert!(set(true).is_ok());
        assert!(is_enabled(), "enabling registers the entry");
        assert!(set(true).is_ok(), "enabling twice is enabling once");
        assert!(is_enabled());

        assert!(set(false).is_ok());
        assert!(!is_enabled(), "disabling removes it");
        assert!(
            set(false).is_ok(),
            "disabling what is not registered is the desired state already reached, \
             not a failure -- otherwise a fresh install shows an error for an untouched toggle"
        );

        let _ = set(was);
    }
}
