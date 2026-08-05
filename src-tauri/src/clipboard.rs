//! The clipboard, which Rust owns so the secret never enters the webview (R-10, R-14).
//!
//! Two platform facts shape everything here, both from the Phase 0 prior-art survey:
//!
//! * **Ownership on X11 and Wayland belongs to the copying process.** The clipboard is not a
//!   system buffer you write into and walk away from — the owner serves the content on
//!   request. `arboard`'s Linux backend serves it from a thread that lives as long as the
//!   `Clipboard` value, so the value is kept alive for the life of the process rather than
//!   dropped at the end of the copy. Drop it and the paste silently produces nothing.
//! * **Clipboard managers keep their own copy.** GPaste, Klipper and CopyQ record history, and
//!   the hints asking them not to are advisory. Nothing in this file can make the clear a
//!   guarantee, which is why the UI copy is an open question against the Phase 2 gate and not
//!   a promise already written.

use std::sync::{Mutex, OnceLock};

use crate::error::{ErrorKind, IpcError, IpcResult};

/// The process-wide clipboard handle.
///
/// One instance, created on first use and never dropped: on Linux, dropping it ends the thread
/// that serves the selection, so a copied password would stop being pastable the moment the
/// command returned.
static CLIPBOARD: OnceLock<Mutex<arboard::Clipboard>> = OnceLock::new();

/// Borrows the process clipboard, creating it on first use.
fn clipboard() -> IpcResult<&'static Mutex<arboard::Clipboard>> {
    // `OnceLock::get_or_init` cannot fail, so the fallible construction is done first and the
    // error surfaced rather than swallowed into a panic inside the initializer.
    if CLIPBOARD.get().is_none() {
        let created = arboard::Clipboard::new().map_err(|_| IpcError::new(ErrorKind::Clipboard))?;
        let _ = CLIPBOARD.set(Mutex::new(created));
    }
    CLIPBOARD
        .get()
        .ok_or_else(|| IpcError::new(ErrorKind::Clipboard))
}

/// Writes `value` to the system clipboard.
///
/// Takes the secret by reference and returns nothing about it. The caller — `copy_field` — has
/// the value only because the core handed it over one field at a time, and it goes out of
/// scope at the end of that command.
pub fn set(value: &str) -> IpcResult<()> {
    let handle = clipboard()?;
    let mut guard = handle
        .lock()
        .map_err(|_| IpcError::new(ErrorKind::Clipboard))?;

    // On Linux the copy carries `x-kde-passwordManagerHint: secret`, which is the most widely
    // adopted convention for asking GPaste, Klipper and CopyQ not to record it. **Advisory** —
    // nothing obliges a manager to honour it, and `tests/clipboard_manager.rs` is what tells us
    // whether one does. It was also D-11's whole reason for choosing `arboard`, and going and
    // looking is how we found the hint was not actually being sent.
    #[cfg(target_os = "linux")]
    {
        use arboard::SetExtLinux as _;
        guard
            .set()
            .exclude_from_history()
            .text(value)
            .map_err(|_| IpcError::new(ErrorKind::Clipboard))
    }
    #[cfg(not(target_os = "linux"))]
    {
        guard
            .set_text(value)
            .map_err(|_| IpcError::new(ErrorKind::Clipboard))
    }
}

/// Clears the clipboard, if this process still owns it.
///
/// Best effort by construction, and the failure is deliberately silent: if another application
/// has taken ownership since the copy, there is nothing of ours left to clear and nothing has
/// gone wrong. Reporting that as an error would train the user to ignore a real one.
pub fn clear() {
    let Ok(handle) = clipboard() else { return };
    let Ok(mut guard) = handle.lock() else { return };
    // `clear()` rather than setting an empty string: on some platforms an empty string is a
    // clipboard entry, and a clipboard manager will happily record it as one.
    let _ = guard.clear();
}
