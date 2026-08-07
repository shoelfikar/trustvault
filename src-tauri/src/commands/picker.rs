//! Choosing a file — `docs/ipc-contract.md` §5, D-59.
//!
//! Two commands, both **ambient**, and both returning a **path and nothing else**. That is the
//! whole point of the module rather than a property it happens to have: the alternative every
//! web-shaped app reaches for is `<input type="file">`, which hands the webview the file's
//! *bytes*. For the Bitwarden import those bytes are a foreign vault's plaintext, and
//! `CLAUDE.md`'s one rule is that a secret in the JS heap can never be wiped. The picker
//! therefore lives here, the host reads the file (`commands/import.rs`), and the only thing
//! that crosses IPC is a string naming where it is.
//!
//! A path is not a secret. It is chosen by the user in an OS dialog this process cannot script,
//! it is already shown on screen by the switcher and the settings pane, and neither command
//! reads a byte of what it points at.
//!
//! **The webview cannot open a dialog by itself.** `tauri-plugin-dialog` is registered in
//! `lib.rs` but `capabilities/default.json` grants `core:default` alone, so `dialog:allow-open`
//! is denied and the plugin's own `open`/`save`/`message` commands are unreachable from the
//! frontend. These two commands are the only doors, they take no arguments, and each hard-codes
//! its own title and filter — so there is no call the frontend can make that widens what a
//! dialog is for. `ipc_audit.rs` asserts the capability file has not grown.
//!
//! Both are `async fn` deliberately. Tauri runs a **synchronous** command on the main thread,
//! and `blocking_pick_file` deadlocks there; an `async fn` command is spawned on the async
//! runtime instead, which is the arrangement upstream's own example uses.

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt as _;

use crate::error::IpcResult;

/// **Ambient.** Asks the user for a Bitwarden export, and returns its path — R-29, D-59.
///
/// `None` means the user closed the dialog, which is not an error and must not be reported as
/// one: a cancelled picker is the most common outcome of opening a picker.
#[tauri::command(rename_all = "snake_case")]
pub async fn pick_import_file(app: AppHandle) -> IpcResult<Option<String>> {
    Ok(pick(
        app,
        "Choose a Bitwarden export",
        "Bitwarden export",
        &["json"],
    ))
}

/// **Ambient.** Asks the user for a vault file, and returns its path — R-22, D-59.
///
/// The switcher passes what comes back to `switch_vault`, which already accepts a path it has
/// never seen and adds it to the list. Nothing here checks the file is a vault: `unlock` is the
/// thing that finds out, and it fails closed for a file that is not one (R-03) — a check here
/// would be a second, weaker opinion about the same question, and it would have to open the
/// file to hold it.
#[tauri::command(rename_all = "snake_case")]
pub async fn pick_vault_file(app: AppHandle) -> IpcResult<Option<String>> {
    Ok(pick(
        app,
        "Open vault file",
        "TrustVault vault",
        &["tvault"],
    ))
}

/// Opens a single-file picker and returns the chosen path as a string.
///
/// The filter is a hint, not a gate — every platform's dialog lets the user switch to "all
/// files", and a picker that could enforce a suffix still would not tell us what is inside.
/// It is here so the common case shows the file the user is looking for rather than their
/// whole home directory.
///
/// A `FilePath::Url` is dropped rather than converted. On desktop the picker only ever yields
/// `FilePath::Path`; the URL variant exists for Android `content://` URIs, which name a stream
/// this build has no way to open. Returning `None` for one is the honest answer, and it reads
/// on screen exactly like a cancelled dialog.
fn pick(app: AppHandle, title: &str, label: &str, extensions: &[&str]) -> Option<String> {
    app.dialog()
        .file()
        .set_title(title)
        .add_filter(label, extensions)
        .blocking_pick_file()
        .and_then(|picked| picked.as_path().map(|path| path.display().to_string()))
}
