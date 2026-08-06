//! Vault lifecycle: status, creation, unlock, lock — `docs/ipc-contract.md` §5 and §7.

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Manager as _, State};
use trustvault_core::{EXTENSION, FORMAT_VERSION, KdfParams, RecoveryCode, Vault};

use crate::dto::{BuildInfo, KdfSummary, VaultStatus};
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, LockReason};

/// **Ambient.** Build and format information, for the About surface and bug reports.
#[tauri::command(rename_all = "snake_case")]
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: env!("CARGO_PKG_VERSION"),
        format_version: FORMAT_VERSION,
        extension: EXTENSION,
    }
}

/// **Ambient.** Where a vault called `name` would go if the user does not say otherwise.
///
/// Deliberately **not** a native file picker. A picker means `tauri-plugin-dialog`, and the
/// manifest's rule is that a plugin is added when a requirement needs it and not before — a
/// plugin is widened attack surface in a process that holds decrypted secrets. R-08 asks for
/// "name & location", which a resolved default and an editable path satisfies. Revisit with a
/// decision log entry if the typed path proves to be the thing users get wrong.
#[tauri::command(rename_all = "snake_case")]
pub fn default_vault_path(app: AppHandle, name: String) -> String {
    // A filename, not a path: everything that could traverse or escape is dropped rather than
    // escaped, because the safe subset is small and obvious and the unsafe one is not.
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let stem = stem.trim_matches('-');
    let stem = if stem.is_empty() { "vault" } else { stem };

    let directory = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .unwrap_or_else(|_| PathBuf::from("."));

    directory
        .join(format!("{}.{EXTENSION}", stem.to_lowercase()))
        .display()
        .to_string()
}

/// **Ambient.** Open, locked, or nothing chosen — the shape the whole frontend routes on.
#[tauri::command(rename_all = "snake_case")]
pub fn vault_status(state: State<'_, AppState>) -> VaultStatus {
    state.status()
}

/// **Ambient.** Measures this machine and returns Argon2id parameters for a new vault (R-02).
///
/// Takes seconds and holds the thread, which is why onboarding shows progress while it runs.
/// The measured wall-clock time is not returned: `KdfParams::calibrate` does not expose it,
/// and timing this command from JS would measure the IPC boundary rather than the KDF.
#[tauri::command(rename_all = "snake_case")]
pub fn calibrate_kdf() -> KdfSummary {
    let params = KdfParams::calibrate();
    KdfSummary {
        m_cost: params.m_cost,
        t_cost: params.t_cost,
        p_cost: params.p_cost,
    }
}

/// **Sanctioned.** Creates a vault and returns its recovery code, once (R-07, R-08).
///
/// The recovery code is the least avoidable secret in the product: it exists to be read by a
/// human off a screen and written down, so it must cross. There is no command to fetch it
/// again — the webview renders it on step 3 and drops it.
#[tauri::command(rename_all = "snake_case")]
pub fn create_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    path: String,
    password: String,
    kdf: KdfSummary,
) -> IpcResult<RecoveryKit> {
    let kit = create_vault_inner(&state, name, path.clone(), password, kdf)?;
    crate::commands::settings::remember_vault(&app, &state);
    Ok(kit)
}

/// The body of [`create_vault`], reachable without a Tauri runtime.
///
/// Split for the same reason `items.rs` splits its commands: `tests/ipc_session.rs` drives the
/// **real** command bodies through a whole-shell session, and a harness that drove a
/// reimplementation would prove only that the reimplementation is safe.
pub fn create_vault_inner(
    state: &AppState,
    name: String,
    path: String,
    password: String,
    kdf: KdfSummary,
) -> IpcResult<RecoveryKit> {
    let params = KdfParams {
        m_cost: kdf.m_cost,
        t_cost: kdf.t_cost,
        p_cost: kdf.p_cost,
    };
    let path = PathBuf::from(path);
    let (mut vault, recovery) = Vault::create(name, &password, params)?;
    vault.save_to(&path)?;

    let code = recovery.display().to_string();
    // Every path that leaves a vault open goes through `Inner::opened`, which is where the
    // remembered path (D-40) and the known-vaults list (R-22) are both kept.
    state.with(|inner| inner.opened(vault, path));

    Ok(RecoveryKit {
        recovery_code: code,
    })
}

/// The one-time recovery kit. The only field is the secret.
#[derive(Debug, serde::Serialize)]
pub struct RecoveryKit {
    /// 24 characters in six groups of four. Shown once and never stored.
    pub recovery_code: String,
}

/// **Vault-class inbound, returns nothing.** Opens a vault with the master password.
///
/// Deliberately returns `()`: the frontend learns the vault is open by calling `vault_status`,
/// which keeps one source of truth for lock state instead of two that can disagree.
///
/// **No early return before the core is called.** Not a "does the file exist" check, not a
/// length check on the password, not a cached-failure short-circuit. The core spends equal
/// work on a wrong password and a corrupt file (R-03), and any check added here that fails
/// faster than the KDF hands that property back.
#[tauri::command(rename_all = "snake_case")]
pub fn unlock(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> IpcResult<()> {
    unlock_inner(&state, path.clone(), password)?;
    crate::commands::settings::remember_vault(&app, &state);
    Ok(())
}

/// The body of [`unlock`], reachable without a Tauri runtime.
pub fn unlock_inner(state: &AppState, path: String, password: String) -> IpcResult<()> {
    let path = PathBuf::from(path);
    let vault = Vault::open_file(&path, &password)?;
    // Every path that leaves a vault open goes through `Inner::opened`, which is where the
    // remembered path (D-40) and the known-vaults list (R-22) are both kept.
    state.with(|inner| inner.opened(vault, path));
    Ok(())
}

/// **Sanctioned.** Opens a vault with the recovery kit and issues a fresh one (R-07).
///
/// Sanctioned for a reason the name does not give away: using a recovery kit **spends** it, so
/// the flow issues a replacement on the spot, and that replacement is a secret travelling
/// outbound. It is counted in the budget of three rather than smuggled in as part of unlock.
///
/// The new kit is saved before it is returned. If the save fails the caller gets an error and
/// the old kit still works, which is the right way for this to fail — the alternative is a
/// user holding a code that opens nothing.
#[tauri::command(rename_all = "snake_case")]
pub fn unlock_recovery_kit(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    code: String,
) -> IpcResult<RecoveryKit> {
    let kit = unlock_recovery_kit_inner(&state, path.clone(), code)?;
    crate::commands::settings::remember_vault(&app, &state);
    Ok(kit)
}

/// The body of [`unlock_recovery_kit`], reachable without a Tauri runtime.
pub fn unlock_recovery_kit_inner(
    state: &AppState,
    path: String,
    code: String,
) -> IpcResult<RecoveryKit> {
    let path = PathBuf::from(path);
    let parsed = RecoveryCode::parse(&code)?;
    let bytes = std::fs::read(&path).map_err(|_| IpcError::new(ErrorKind::Io))?;
    let mut vault = Vault::open_with_recovery(&bytes, &parsed)?;

    let reissued = vault.reissue_recovery_code()?;
    vault.save_to(&path)?;
    let fresh = reissued.display().to_string();

    // Every path that leaves a vault open goes through `Inner::opened`, which is where the
    // remembered path (D-40) and the known-vaults list (R-22) are both kept.
    state.with(|inner| inner.opened(vault, path));

    Ok(RecoveryKit {
        recovery_code: fresh,
    })
}

/// **Vault-class.** Locks on demand — R-09.
///
/// Idempotent: locking a locked vault succeeds and does nothing. The failure mode of a lock
/// command that can error is a user hammering it during a panic.
#[tauri::command(rename_all = "snake_case")]
pub fn lock(app: AppHandle, state: State<'_, AppState>) -> IpcResult<()> {
    lock_now(&app, &state, LockReason::Manual);
    Ok(())
}

/// Locks the vault, flushing any buffered audit entries first, and emits `vault-locked`.
///
/// The flush is what D-31 means by "on save": reveals buffer in memory so that reading a
/// password does not rewrite the vault file, and lock is the natural moment to write the tail.
/// A crash before this point loses it, which the decision accepted.
pub fn lock_now(app: &AppHandle, state: &AppState, reason: LockReason) {
    state.with(|inner| {
        if let (Some(vault), Some(path)) = (inner.vault.as_mut(), inner.path.as_ref())
            && vault.has_unflushed_audit()
        {
            // Best effort. A failed flush must not prevent the lock: an unlockable vault is a
            // worse outcome than a lost audit tail, and refusing to lock because a disk is
            // full would leave the master key in memory.
            let _ = vault.save_to(path);
        }
    });

    if state.lock() {
        let _ = app.emit("vault-locked", LockEvent { reason });
    }
}

/// Payload of `vault-locked`.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct LockEvent {
    /// Why it locked, so the lock screen can say.
    pub reason: LockReason,
}

/* ---- Multi-vault — R-22, R-18, contract §6.7 ------------------------------ */

/// One row of the switcher.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct VaultRef {
    /// Absolute path to the file.
    pub path: String,
    /// The real name **only for the open vault**; the file stem for every other row.
    pub display_name: String,
    /// Unix milliseconds, or `None` for a vault that has never been opened here.
    pub last_opened_at: Option<i64>,
}

/// **Ambient.** Every vault this installation knows about — R-22.
///
/// Ambient rather than vault-class, and it has to be: the switcher's whole job is to be usable
/// while nothing is unlocked. Nothing in the response is secret — a list of file paths on the
/// user's own disk, which the file manager shows them anyway.
///
/// **`display_name` is the file stem for every row except the open one**, and the switcher must
/// not imply otherwise. The real name lives inside the sealed body; with no key there is
/// nothing to read it with, so the stem is not a fallback but the only knowable thing.
#[tauri::command(rename_all = "snake_case")]
pub fn list_vaults(state: State<'_, AppState>) -> Vec<VaultRef> {
    list_vaults_inner(&state)
}

/// The body of [`list_vaults`], reachable without a Tauri runtime.
pub fn list_vaults_inner(state: &AppState) -> Vec<VaultRef> {
    state
        .with(|inner| {
            let open = inner.path.as_ref().map(|path| path.display().to_string());
            inner
                .known_vaults
                .iter()
                .map(|known| VaultRef {
                    display_name: display_name_for(inner, &known.path, open.as_deref()),
                    path: known.path.clone(),
                    last_opened_at: known.last_opened_at,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The name `list_vaults` reports for one path — and therefore the string `delete_vault`
/// checks the confirmation against.
///
/// Factored out rather than written twice on purpose: the contract defines the confirmation as
/// "equal to the `display_name` this command would report", so two implementations that drifted
/// would make a vault undeletable through its own dialog, with the user typing exactly what is
/// on their screen and being told it does not match.
fn display_name_for(inner: &crate::state::Inner, path: &str, open: Option<&str>) -> String {
    let is_open = open == Some(path) && inner.vault.is_some();
    match (is_open, inner.vault.as_ref()) {
        (true, Some(vault)) => vault.name().to_owned(),
        _ => std::path::Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default(),
    }
}

/// **Ambient.** Points the app at another vault, locking the current one first — R-22.
///
/// The order in that sentence is the acceptance criterion: **lock and zeroize the outgoing
/// vault, then move.** Reversed, there is a window in which the state names the new path while
/// the old vault's master key is still in memory, and any command arriving in it would answer
/// from a vault the user believes they have left.
///
/// It leaves the state `locked` and **cannot** unlock, because it is given no password. That
/// is why it is ambient: the switcher has to work from the lock screen, which is where a user
/// who wants a different vault most often is.
#[tauri::command(rename_all = "snake_case")]
pub fn switch_vault(app: AppHandle, state: State<'_, AppState>, path: String) -> IpcResult<()> {
    lock_now(&app, &state, LockReason::Manual);
    switch_vault_inner(&state, path);
    crate::commands::settings::remember_vault(&app, &state);
    Ok(())
}

/// The body of [`switch_vault`] after the lock, reachable without a Tauri runtime.
///
/// The lock is the caller's, not this function's, because emitting `vault-locked` needs an
/// `AppHandle` — and a switch that zeroized without telling the frontend would leave the
/// window showing an unlocked shell over a vault that is gone.
pub fn switch_vault_inner(state: &AppState, path: String) {
    state.with(|inner| {
        let target = PathBuf::from(&path);
        inner.settings.last_vault_path = Some(path.clone());
        inner.path = Some(target);
        // Listed even though it has not been opened here yet, so the switcher shows the vault
        // the user just chose. `last_opened_at` stays `None` until an unlock actually happens
        // -- claiming a time for it would make the sort order a small lie.
        if !inner.known_vaults.iter().any(|known| known.path == path) {
            inner.known_vaults.push(crate::state::KnownVault {
                path,
                last_opened_at: None,
            });
        }
    });
}

/// **Ambient.** Removes a vault from the list. **The file is untouched** — R-22.
///
/// This is the "Leave vault" flow, and confusing it with `delete_vault` would be the worst bug
/// in the application: one forgets a path, the other destroys the only copy of everything.
/// They are kept apart in three places — different command names, different confirmation
/// requirements, and different words on the buttons.
///
/// Forgetting the **open** vault is allowed and does not close it. The user has said "stop
/// listing this", not "get out of it", and locking them out of a vault they are working in to
/// satisfy a bookkeeping request is the wrong reading of a mild instruction.
#[tauri::command(rename_all = "snake_case")]
pub fn forget_vault(app: AppHandle, state: State<'_, AppState>, path: String) -> IpcResult<()> {
    forget_vault_inner(&state, &path);
    crate::commands::settings::remember_vault(&app, &state);
    Ok(())
}

/// The body of [`forget_vault`], reachable without a Tauri runtime.
pub fn forget_vault_inner(state: &AppState, path: &str) {
    state.with(|inner| {
        inner.known_vaults.retain(|known| known.path != path);
        // Forgetting the vault that a relaunch would offer must clear that offer too, or the
        // next launch re-adds the entry the user just removed and the button does nothing.
        if inner.settings.last_vault_path.as_deref() == Some(path) && inner.vault.is_none() {
            inner.settings.last_vault_path = None;
            inner.path = None;
        }
    });
}

/// **Ambient.** Erases a vault file, and **verifies the typed name here rather than trusting
/// the UI** — R-18, R-22.
///
/// This is the one confirmation the host checks. The asymmetry with `delete_item`, which
/// deliberately does not, is the whole of the reasoning: a wrong `delete_item` costs one entry
/// that `history` may still hold, and a wrong `delete_vault` costs everything, with no undo
/// anywhere in the product. R-18 asks for the typed name; a typed name checked only in
/// JavaScript is checked by the layer this contract does not trust.
///
/// `confirm_name` must equal what [`list_vaults`] would report for that path — the string the
/// user can actually see — and both come from `display_name_for` so they cannot drift.
///
/// **Deleting the open vault locks it first, in that order**, so the master key is zeroized
/// before the bytes go.
#[tauri::command(rename_all = "snake_case")]
pub fn delete_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    confirm_name: String,
) -> IpcResult<()> {
    // The whole sequence lives in `_inner` on purpose, and this command is the one where that
    // matters most: D-47 found that the wrapper is exactly the half no harness can drive, and
    // the confirmation R-18 asks for is the single check in this product whose *success* is
    // irreversible. A check reachable only from a running webview is a check no test has read.
    let was_open = delete_vault_inner(&state, &path, &confirm_name)?;
    if was_open {
        // The zeroizing already happened inside. What is left is telling the window, which
        // needs an `AppHandle` -- and a delete that zeroized without saying so would leave an
        // unlocked shell drawn over a vault that no longer exists.
        let _ = app.emit(
            "vault-locked",
            LockEvent {
                reason: LockReason::Manual,
            },
        );
    }
    crate::commands::settings::remember_vault(&app, &state);
    Ok(())
}

/// The body of [`delete_vault`], reachable without a Tauri runtime. Returns whether a vault was
/// open and therefore locked, so the caller knows to emit `vault-locked`.
///
/// The order here **is** the specification — see §6.7 — and each step is placed against a
/// specific way this could go wrong:
///
/// 1. The name is checked first, before the lock. A typo must not cost the user their session.
/// 2. The open vault is zeroized before its bytes are touched.
/// 3. The file goes before the bookkeeping. If the remove fails the entry stays in the list,
///    which is the honest state — a vault still on disk that the switcher stopped showing is a
///    file the user can no longer reach from inside the app and has not been told about.
pub fn delete_vault_inner(state: &AppState, path: &str, confirm_name: &str) -> IpcResult<bool> {
    let expected = state
        .with(|inner| {
            let open = inner.path.as_ref().map(|p| p.display().to_string());
            display_name_for(inner, path, open.as_deref())
        })
        .unwrap_or_default();
    if confirm_name != expected {
        return Err(IpcError::new(ErrorKind::ConfirmationMismatch));
    }

    let is_open = state
        .with(|inner| {
            inner
                .path
                .as_ref()
                .is_some_and(|open| open.display().to_string() == path)
        })
        .unwrap_or(false);
    // Deliberately not `lock_now`: that flushes the buffered audit tail to the vault file
    // first, and this file is about to stop existing. Writing to it would be a save whose only
    // effect is to make the delete slower.
    let was_open = is_open && state.lock();

    match std::fs::remove_file(path) {
        Ok(()) => {}
        // Already gone is the desired state reached. Erroring would leave the entry in the
        // list with no way to remove it: the file it names cannot be deleted twice.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(IpcError::new(ErrorKind::Io)),
    }
    forget_vault_inner(state, path);
    Ok(was_open)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trustvault_core::KdfParams;

    /// A real vault file on disk, opened. `KdfParams::TESTING` because these tests are about
    /// bookkeeping, not about the KDF.
    fn vault_at(name: &str) -> (AppState, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "trustvault-{name}-{}-{:?}.tvault",
            std::process::id(),
            std::thread::current().id()
        ));
        let state = AppState::default();
        create_vault_inner(
            &state,
            "Personal Vault".into(),
            path.display().to_string(),
            "correct horse battery staple".into(),
            KdfSummary {
                m_cost: KdfParams::TESTING.m_cost,
                t_cost: KdfParams::TESTING.t_cost,
                p_cost: KdfParams::TESTING.p_cost,
            },
        )
        .expect("the test parameters are valid");
        (state, path)
    }

    #[test]
    fn opening_a_vault_lists_it_and_only_once() {
        // `Inner::opened` is the one place `known_vaults` grows, and the property that makes
        // it safe to call from four command bodies is that it does not accumulate duplicates:
        // a user who unlocks the same vault every morning must not have a switcher that grows
        // a row a day.
        let (state, path) = vault_at("list");
        let key = path.display().to_string();

        assert_eq!(list_vaults_inner(&state).len(), 1);
        unlock_inner(&state, key.clone(), "correct horse battery staple".into()).unwrap();
        unlock_inner(&state, key.clone(), "correct horse battery staple".into()).unwrap();

        let listed = list_vaults_inner(&state);
        assert_eq!(
            listed.len(),
            1,
            "re-opening moves an entry, it does not add one"
        );
        assert_eq!(listed[0].path, key);
        assert!(listed[0].last_opened_at.is_some());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn only_the_open_vault_is_listed_under_its_real_name() {
        // §6.7's wrinkle, and the switcher must not imply otherwise: a vault's real name is
        // inside the sealed body, so with no key there is nothing to read it with. The stem is
        // not a fallback -- it is the only knowable thing.
        let (state, path) = vault_at("names");
        assert_eq!(list_vaults_inner(&state)[0].display_name, "Personal Vault");

        state.lock();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        assert_eq!(
            list_vaults_inner(&state)[0].display_name,
            stem,
            "locked, the file stem is all there is"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn switching_zeroizes_before_it_moves() {
        // R-22's acceptance criterion. The order is the whole of it: reversed, there is a
        // window in which the state names the new path while the old vault's master key is
        // still in memory, and any command arriving in it answers from a vault the user
        // believes they have left.
        let (state, path) = vault_at("switch");
        assert_eq!(state.status().state, crate::dto::VaultState::Unlocked);

        state.lock();
        switch_vault_inner(&state, "/elsewhere/other.tvault".into());

        let status = state.status();
        assert_eq!(status.state, crate::dto::VaultState::Locked);
        assert_eq!(status.path.as_deref(), Some("/elsewhere/other.tvault"));
        assert_eq!(
            list_vaults_inner(&state).len(),
            2,
            "the vault just switched to is listed, so the switcher can switch back"
        );
        assert!(
            list_vaults_inner(&state)
                .iter()
                .any(|entry| entry.path == "/elsewhere/other.tvault"
                    && entry.last_opened_at.is_none()),
            "and it claims no open time, because it has not been opened"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn forgetting_a_vault_leaves_the_file_alone() {
        // The distinction that would be the worst bug in the application: one forgets a path,
        // the other destroys the only copy of everything. Asserted rather than trusted to the
        // two commands having different names.
        let (state, path) = vault_at("forget");
        let key = path.display().to_string();

        forget_vault_inner(&state, &key);

        assert!(list_vaults_inner(&state).is_empty());
        assert!(path.is_file(), "forget removes a list entry, never a file");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn deleting_needs_the_name_the_user_can_actually_see() {
        // R-18, verified in Rust rather than in the dialog. The name checked against is the one
        // `list_vaults` reports, because that is the string on the user's screen -- two
        // implementations that drifted would make a vault undeletable through its own dialog.
        let (state, path) = vault_at("delete-name");
        let key = path.display().to_string();

        let refused = delete_vault_inner(&state, &key, "personal vault");
        assert_eq!(
            refused.unwrap_err().kind,
            ErrorKind::ConfirmationMismatch,
            "the check is exact -- a near miss is a miss, because the cost of a wrong one \
             is everything with no undo anywhere in the product"
        );
        assert!(path.is_file(), "and nothing was deleted");
        assert_eq!(
            state.status().state,
            crate::dto::VaultState::Unlocked,
            "a typo must not cost the user their open session either"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn deleting_the_open_vault_zeroizes_before_the_bytes_go() {
        let (state, path) = vault_at("delete-open");
        let key = path.display().to_string();

        let was_open = delete_vault_inner(&state, &key, "Personal Vault").unwrap();

        assert!(
            was_open,
            "the caller is told to announce the lock it did not ask for"
        );
        assert!(!path.exists(), "the file is gone");
        assert!(list_vaults_inner(&state).is_empty(), "and so is its entry");
        assert_eq!(
            state.status().state,
            crate::dto::VaultState::NoVault,
            "not `locked` -- a lock screen offering to unlock a file that no longer exists is \
             a dead end the user cannot get out of"
        );
    }

    #[test]
    fn deleting_a_vault_whose_file_has_already_gone_still_clears_the_entry() {
        // Erroring here would leave an entry in the list with no way to remove it: the file it
        // names cannot be deleted twice, so the only exit would be editing settings.json.
        let (state, path) = vault_at("delete-twice");
        let key = path.display().to_string();
        state.lock();
        std::fs::remove_file(&path).unwrap();

        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        delete_vault_inner(&state, &key, &stem).expect("already gone is the desired state");
        assert!(list_vaults_inner(&state).is_empty());
    }
}
