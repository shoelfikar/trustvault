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

/// **Ambient.** Asks the user where a vault that does not exist yet should go — R-08, D-60.
///
/// The third door, and the only one that opens a **save** dialog. Onboarding's *Change* button
/// has been drawn-and-disabled since D-36 with "a file picker would mean adding a plugin" in its
/// `title`; the plugin arrived with D-59 and the reason died with it, which is what the D-36
/// sweep is for. Typing an absolute path into a text field still works and is still the fallback
/// — this is the surface, not the mechanism.
///
/// `suggested` pre-fills the file name, and it is the one argument any picker here takes. What
/// D-59 made load-bearing is that **the frontend cannot change what a dialog is for**, and that
/// still holds: the title and the filter are fixed in Rust, and the suggestion is reduced to its
/// own `file_name` component, so a value like `../../etc/passwd` reaches the dialog as
/// `passwd` and an absolute path reaches it as its last segment. It is a suggestion in a field
/// the user reads and confirms, not a destination.
///
/// **The dialog may name a file that already exists**, and every platform's save dialog asks
/// before it does. Nothing is overwritten here — this command returns a string — but
/// `create_vault` does write it, so the confirmation the OS shows is the only one there is.
/// That is unchanged from typing the same path by hand, which onboarding has always allowed.
#[tauri::command(rename_all = "snake_case")]
pub async fn pick_new_vault_path(app: AppHandle, suggested: String) -> IpcResult<Option<String>> {
    let name = std::path::Path::new(&suggested)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "vault.tvault".to_owned());

    Ok(app
        .dialog()
        .file()
        .set_title("Choose where the vault goes")
        .add_filter("TrustVault vault", &["tvault"])
        .set_file_name(name)
        .blocking_save_file()
        .and_then(|picked| picked.as_path().map(|path| path.display().to_string())))
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
