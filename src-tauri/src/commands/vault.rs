//! Vault lifecycle: status, creation, unlock, lock — `docs/ipc-contract.md` §5 and §7.

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Manager as _, State};
use trustvault_core::{EXTENSION, FORMAT_VERSION, KdfParams, RecoveryCode, Vault};

use crate::commands::{save_open_vault, with_vault};
use crate::dto::{BuildInfo, KdfSummary, VaultStatus};
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, LockReason, PendingVault};

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
/// A default is what this returns; the field is editable and, since D-60, *Change* opens a save
/// dialog beside it. The reason there was no picker for two phases is in `docs/ipc-contract.md`
/// §5 and it is still the reason a plugin arrives late, not the reason this command exists.
///
/// **The suffix is not tidiness — it is what keeps the default from naming a vault that already
/// exists.** Since D-62 a second vault can be created from inside the shell, and the flow opens
/// with the same default name it opened with the first time; without this loop, a user who kept
/// "Personal Vault" would be handed the path of the vault they are currently using. Refusing to
/// overwrite is `create_vault`'s job and it does it, but a default that walks into the refusal
/// is a default that teaches the user the app is broken.
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

    free_path(&directory, &stem.to_lowercase())
        .display()
        .to_string()
}

/// `<directory>/<stem>.tvault`, or the first `-2`, `-3`, … that is not taken.
///
/// Bounded rather than a `loop`: at a hundred vaults called the same thing the suffix has
/// stopped being a convenience, and an unbounded search on a directory that answers "exists" to
/// everything would not return. The last candidate is returned even if it is taken, because
/// `create_vault` is what refuses — two places deciding that would be one too many.
fn free_path(directory: &std::path::Path, stem: &str) -> PathBuf {
    let mut candidate = directory.join(format!("{stem}.{EXTENSION}"));
    for n in 2..=99 {
        if !candidate.try_exists().unwrap_or(true) {
            return candidate;
        }
        candidate = directory.join(format!("{stem}-{n}.{EXTENSION}"));
    }
    candidate
}

/// **Ambient.** Open, locked, or nothing chosen — the shape the whole frontend routes on.
#[tauri::command(rename_all = "snake_case")]
pub fn vault_status(state: State<'_, AppState>) -> VaultStatus {
    state.status()
}

/// **Vault-class.** Labels the vault with its owner's name and e-mail, and saves — D-70.
///
/// The two strings go **inside the sealed body**, which is what makes them worth storing rather
/// than inventing on screen: they identify a person, and the file whose whole purpose is being
/// unreadable without a key is the right place for that. It is not an account. Nothing
/// authenticates against them, nothing is sent anywhere (D-03), and the recovery kit and the
/// sidebar footer are the only things that read them.
///
/// Vault-class rather than ambient because it writes to the body, so it goes through
/// [`with_vault`] like every other mutation and fails `locked` when nothing is open.
#[tauri::command(rename_all = "snake_case")]
pub fn set_profile(state: State<'_, AppState>, name: String, email: String) -> IpcResult<()> {
    set_profile_inner(&state, name, email)
}

/// The body of [`set_profile`], reachable without a Tauri runtime.
///
/// **A profile that did not change is not saved**, and that is the whole reason `set_profile`
/// returns a bool down in the core: the Edit-profile dialog's Save is pressed whether or not
/// anything was typed, and writing on every press would rewrite — re-encrypt, re-nonce, and
/// atomically replace — the entire vault file to store the strings it already held.
pub fn set_profile_inner(state: &AppState, name: String, email: String) -> IpcResult<()> {
    with_vault(state, |vault, inner| {
        if vault.set_profile(&name, &email) {
            save_open_vault(vault, inner)?;
        }
        Ok(())
    })
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

/// **Sanctioned.** Creates a vault **in memory** and returns its recovery code, once (R-07, R-08).
///
/// The recovery code is the least avoidable secret in the product: it exists to be read by a
/// human off a screen and written down, so it must cross. There is no command to fetch it
/// again — the webview renders it on step 3 and drops it.
///
/// **Nothing is written and nothing is opened here — D-69.** The vault waits in
/// `Inner::pending` until [`commit_vault`], which is onboarding step 3's acknowledgement. Until
/// this was split, the file was written at the end of step 2 and a user who closed the window
/// while reading the kit owned a vault whose kit had never been recorded: shown exactly once,
/// no command to fetch it again, and the remembered path (D-40) sending the next launch to a
/// lock screen it had no recovery route out of.
///
/// **A vault may already be open when this is called** — since 2026-08-07 and D-62, the
/// switcher's *New vault* reaches onboarding from inside the shell. It stays open, and that is
/// the second thing the split buys: abandoning onboarding now leaves the user in the vault they
/// were already in, where it used to close it before the new one was certain.
#[tauri::command(rename_all = "snake_case")]
pub fn create_vault(
    state: State<'_, AppState>,
    name: String,
    path: String,
    password: String,
    kdf: KdfSummary,
) -> IpcResult<RecoveryKit> {
    create_vault_inner(&state, name, path, password, kdf)
}

/// Writes the vault [`create_vault`] left pending, and opens it — R-08, D-69.
///
/// **Not sanctioned**: it returns nothing at all, which is what keeps the budget at four. The
/// secret crossed on the way in; this is the acknowledgement coming back.
#[tauri::command(rename_all = "snake_case")]
pub fn commit_vault(app: AppHandle, state: State<'_, AppState>) -> IpcResult<()> {
    commit_vault_inner(&state)?;
    crate::commands::settings::remember_vault(&app, &state);
    Ok(())
}

/// The body of [`create_vault`], reachable without a Tauri runtime.
///
/// Split for the same reason `items.rs` splits its commands: `tests/ipc_session.rs` drives the
/// **real** command bodies through a whole-shell session, and a harness that drove a
/// reimplementation would prove only that the reimplementation is safe. Since D-62 the split
/// carries something else too — the whole "close the old vault, open the new one" sequence is
/// here rather than in the wrapper, which is `delete_vault`'s lesson applied before it bites:
/// an ordering written into the `#[tauri::command]` layer is one no harness can read.
///
/// **The order is the acceptance criterion, and it has two halves.**
///
/// *Everything that can fail happens first.* `Vault::create` and `save_to` run before the open
/// vault is touched, so a path that cannot be written leaves the user exactly where they were
/// rather than locked out of a vault they had open because they mistyped a directory.
///
/// *Then the outgoing vault is flushed and dropped before the new one is installed.* The flush
/// is the half that would go missing silently: reveals buffer in memory (D-31) and lock is what
/// writes the tail, so replacing a vault without flushing discards its audit log — the one
/// record whose absence looks exactly like nothing having happened.
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
    // Checked here **and** again in `commit_vault_inner`, which is not redundant: this one is so
    // the user is sent back to step 1 before being shown a recovery kit for a vault that cannot
    // be saved, and that one is the check that actually guards the file, at the moment of
    // writing. It is `try_exists` rather than `exists` so that a path we cannot even stat fails
    // closed instead of reading as free.
    if path.try_exists().unwrap_or(true) {
        return Err(IpcError::new(ErrorKind::PathInUse));
    }
    let (vault, recovery) = Vault::create(name, &password, params)?;
    let code = recovery.display().to_string();

    // The vault waits here. Nothing is written, the open vault is untouched, and a lock at any
    // point from now until the acknowledgement drops it — D-69, and `AppState::lock`.
    state.with(|inner| inner.pending = Some(PendingVault { vault, path }));

    Ok(RecoveryKit {
        recovery_code: code,
    })
}

/// The body of [`commit_vault`], reachable without a Tauri runtime.
///
/// **The order is the acceptance criterion, and it has the same two halves it always had** —
/// they have simply moved to the command that writes.
///
/// *Everything that can fail happens first.* The existence check and `save_to` run before the
/// open vault is touched, so a path that cannot be written leaves the user exactly where they
/// were rather than locked out of a vault they had open because a directory went away while
/// they were reading their recovery kit.
///
/// *Then the outgoing vault is flushed and dropped before the new one is installed.* The flush
/// is the half that would go missing silently: reveals buffer in memory (D-31) and lock is what
/// writes the tail, so replacing a vault without flushing discards its audit log — the one
/// record whose absence looks exactly like nothing having happened.
///
/// A missing pending vault is `Internal` rather than a kind of its own: the only way to reach it
/// is a frontend that called this without having called `create_vault`, or one that called it
/// after a lock discarded the pending vault — both of which are bugs here, not conditions a user
/// can be told something useful about.
///
/// **A failed write puts the pending vault back.** Taking it out and dropping it on the way past
/// would leave the user on step 3 holding the one and only rendering of a recovery kit for a
/// vault that no longer exists anywhere, with a button that answers `internal` from then on.
/// This is exactly the window the split opened: the user is now sitting on this screen for as
/// long as it takes to write a code down, and a directory can go away inside it.
pub fn commit_vault_inner(state: &AppState) -> IpcResult<()> {
    let pending = state
        .with(|inner| inner.pending.take())
        .flatten()
        .ok_or_else(|| IpcError::new(ErrorKind::Internal))?;

    let PendingVault { mut vault, path } = pending;
    let written = if path.try_exists().unwrap_or(true) {
        Err(IpcError::new(ErrorKind::PathInUse))
    } else {
        vault.save_to(&path).map_err(IpcError::from)
    };
    if let Err(error) = written {
        state.with(|inner| inner.pending = Some(PendingVault { vault, path }));
        return Err(error);
    }

    // Nothing below this line can fail, which is what makes the ordering safe to run at all.
    flush_audit(state);
    state.lock();
    // Every path that leaves a vault open goes through `Inner::opened`, which is where the
    // remembered path (D-40) and the known-vaults list (R-22) are both kept.
    state.with(|inner| inner.opened(vault, path));

    Ok(())
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
    flush_audit(state);

    if state.lock() {
        let _ = app.emit("vault-locked", LockEvent { reason });
    }
}

/// Writes the open vault's buffered audit tail, if it has one — the first half of [`lock_now`].
///
/// Factored out on 2026-08-07 because `create_vault_inner` needs it too and needs it without an
/// `AppHandle` (D-62). Two copies would drift in the direction that costs: a second path that
/// closes a vault and forgets the flush loses the audit tail silently, and a missing audit entry
/// is indistinguishable from a reveal that never happened.
///
/// Best effort, deliberately. A failed flush must not prevent the vault closing: an unlockable
/// vault is a worse outcome than a lost tail, and refusing to lock because a disk is full would
/// leave the master key in memory.
fn flush_audit(state: &AppState) {
    state.with(|inner| {
        if let (Some(vault), Some(path)) = (inner.vault.as_mut(), inner.path.as_ref())
            && vault.has_unflushed_audit()
        {
            let _ = vault.save_to(path);
        }
    });
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
    ///
    /// **Two calls since D-69**, and the second is the point of the split: `create_vault_inner`
    /// leaves the vault in memory and `commit_vault_inner` is what writes it and opens it. Every
    /// test below is about a vault that exists, so every one of them needs both.
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
        commit_vault_inner(&state).expect("the pending vault is writable");
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

    /* ---- Creating a second vault — D-62 ----------------------------------- */

    #[test]
    fn creating_a_vault_never_writes_over_a_file_that_is_already_there() {
        // The trap D-62 opened and this closes. Before it, a second vault could not be created
        // at all; after it, the default name resolves to a default path, and a user who kept
        // "Personal Vault" for both would have had the first one written over by the second --
        // no confirmation, no undo, and no key in memory to have warned with.
        let (state, path) = vault_at("no-overwrite");
        let key = path.display().to_string();
        let before = std::fs::read(&path).unwrap();

        let refused = create_vault_inner(
            &state,
            "Second Vault".into(),
            key,
            "a different master password".into(),
            testing_kdf(),
        );

        assert_eq!(refused.unwrap_err().kind, ErrorKind::PathInUse);
        assert_eq!(
            std::fs::read(&path).unwrap(),
            before,
            "the file on disk is byte-for-byte what it was"
        );
        assert_eq!(
            state.status().display_name,
            "Personal Vault",
            "and the vault that was open is still the one that is open -- a refusal must not \
             cost the user the session they had"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_commit_that_cannot_write_keeps_the_pending_vault_for_a_second_try() {
        // D-69 opened a window that did not exist before: the user now sits on step 3 for as
        // long as it takes to copy a recovery code down, and the directory can go away inside
        // it. If a failed commit dropped the pending vault, the user would be holding the one
        // and only rendering of a kit for a vault that exists nowhere, with a button that
        // answers `internal` from then on. The retry is the whole point of the window.
        let (state, occupied) = vault_at("commit-retry");

        // A path that is already taken is the reachable way to make `save_to` refuse without
        // depending on permissions, which differ per platform and per CI runner.
        create_vault_inner(
            &state,
            "Doomed Vault".into(),
            occupied.display().to_string(),
            "a different master password".into(),
            testing_kdf(),
        )
        .expect_err("create refuses an occupied path up front");

        // So drive it the other way: create at a free path, then occupy that path behind it.
        let free = occupied.with_file_name("trustvault-commit-retry-b.tvault");
        let _ = std::fs::remove_file(&free);
        create_vault_inner(
            &state,
            "Second Vault".into(),
            free.display().to_string(),
            "another master password entirely".into(),
            testing_kdf(),
        )
        .expect("a free path");
        std::fs::write(&free, b"something else got here first").unwrap();

        let refused = commit_vault_inner(&state);
        assert_eq!(refused.unwrap_err().kind, ErrorKind::PathInUse);
        assert_eq!(
            std::fs::read(&free).unwrap(),
            b"something else got here first",
            "the refusal must not have written over what was there"
        );
        assert_eq!(
            state.status().display_name,
            "Personal Vault",
            "and the open vault is untouched by a failed commit"
        );

        // The retry: clear the obstruction and commit the same pending vault again.
        std::fs::remove_file(&free).unwrap();
        commit_vault_inner(&state).expect("the pending vault survived the refusal");
        assert_eq!(state.status().display_name, "Second Vault");

        let _ = std::fs::remove_file(&occupied);
        let _ = std::fs::remove_file(&free);
    }

    #[test]
    fn a_lock_discards_a_pending_vault_rather_than_leaving_a_key_alive() {
        // The security half of D-69. A pending vault holds a decrypted key exactly like an open
        // one, so a lock has to reach it; otherwise onboarding is a way to keep a key alive
        // behind a lock screen. The user-visible consequence is deliberate: the half-made vault
        // is gone and onboarding starts again, which is the right end for a vault whose kit was
        // never written down.
        let state = AppState::default();
        let path = std::env::temp_dir().join(format!(
            "trustvault-pending-lock-{}-{:?}.tvault",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_file(&path);

        create_vault_inner(
            &state,
            "Abandoned Vault".into(),
            path.display().to_string(),
            "correct horse battery staple".into(),
            testing_kdf(),
        )
        .expect("a free path");

        state.lock();

        assert_eq!(
            commit_vault_inner(&state).unwrap_err().kind,
            ErrorKind::Internal,
            "the pending vault is gone, so there is nothing to commit"
        );
        assert!(!path.exists(), "and nothing was ever written");
    }

    #[test]
    fn creating_a_second_vault_closes_the_first_one_before_it_opens_the_new_one() {
        // The ordering `switch_vault` is tested for, in the other command that replaces the
        // open vault. Reversed, there is a window in which the state names the new vault while
        // the previous one's master key is still in memory.
        //
        // Since D-69 the replacement happens on **commit**, not on create, and the assertion
        // between the two is the half that is new: the first vault is still open while the
        // second one is only pending, so abandoning onboarding costs the user nothing.
        let (state, first) = vault_at("second-vault");
        let second = first.with_file_name("trustvault-second-vault-b.tvault");
        let _ = std::fs::remove_file(&second);

        create_vault_inner(
            &state,
            "Work Vault".into(),
            second.display().to_string(),
            "another master password entirely".into(),
            testing_kdf(),
        )
        .expect("a writable path");

        assert_eq!(
            state.status().display_name,
            "Personal Vault",
            "a pending vault must not displace the open one -- abandoning onboarding here used \
             to leave the user with no vault open at all"
        );
        assert!(!second.exists(), "and nothing is on disk until the commit");

        commit_vault_inner(&state).expect("a writable path");

        let status = state.status();
        assert_eq!(status.state, crate::dto::VaultState::Unlocked);
        assert_eq!(status.display_name, "Work Vault");
        assert_eq!(
            status.path.as_deref(),
            Some(second.display().to_string()).as_deref()
        );
        assert!(
            first.is_file(),
            "the first vault's file is left where it is"
        );
        assert_eq!(
            list_vaults_inner(&state).len(),
            2,
            "and it is still in the switcher, which is the whole point of having two"
        );

        let _ = std::fs::remove_file(&first);
        let _ = std::fs::remove_file(&second);
    }

    #[test]
    fn the_suggested_path_steps_around_a_vault_that_is_already_there() {
        // `default_vault_path` resolves a directory this test cannot control, so what is tested
        // is the part that decides: the same stem twice must not name the same file twice.
        let directory = std::env::temp_dir().join(format!("tv-free-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();

        let first = free_path(&directory, "personal-vault");
        assert_eq!(first.file_name().unwrap(), "personal-vault.tvault");
        std::fs::write(&first, b"occupied").unwrap();

        let second = free_path(&directory, "personal-vault");
        assert_eq!(second.file_name().unwrap(), "personal-vault-2.tvault");

        let _ = std::fs::remove_dir_all(&directory);
    }

    fn testing_kdf() -> KdfSummary {
        KdfSummary {
            m_cost: KdfParams::TESTING.m_cost,
            t_cost: KdfParams::TESTING.t_cost,
            p_cost: KdfParams::TESTING.p_cost,
        }
    }
}
