//! Vault lifecycle: status, creation, unlock, lock — `docs/ipc-contract.md` §5 and §7.

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Manager as _, State};
use trustvault_core::{EXTENSION, FORMAT_VERSION, KdfParams, RecoveryCode, Vault};

use crate::dto::{BuildInfo, KdfSummary, VaultStatus};
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, LockReason, now_ms};

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
    state.with(|inner| {
        inner.generation = inner.generation.wrapping_add(1);
        inner.vault = Some(vault);
        // Remembered here rather than in the command wrapper, so that every path which
        // leaves a vault open records it -- D-40. The wrapper only writes it to disk.
        inner.settings.last_vault_path = Some(path.display().to_string());
        inner.path = Some(path);
        inner.last_activity = now_ms();
    });

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
    state.with(|inner| {
        inner.generation = inner.generation.wrapping_add(1);
        inner.vault = Some(vault);
        // Remembered here rather than in the command wrapper, so that every path which
        // leaves a vault open records it -- D-40. The wrapper only writes it to disk.
        inner.settings.last_vault_path = Some(path.display().to_string());
        inner.path = Some(path);
        inner.last_activity = now_ms();
    });
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

    state.with(|inner| {
        inner.generation = inner.generation.wrapping_add(1);
        inner.vault = Some(vault);
        // Remembered here rather than in the command wrapper, so that every path which
        // leaves a vault open records it -- D-40. The wrapper only writes it to disk.
        inner.settings.last_vault_path = Some(path.display().to_string());
        inner.path = Some(path);
        inner.last_activity = now_ms();
    });

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
