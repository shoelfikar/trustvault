//! Reading items, revealing one field, and copying one field —
//! `docs/ipc-contract.md` §6.3 and §7.

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager as _, State};
use trustvault_core::{FieldId, ItemId, ItemKind};

use crate::clipboard;
use crate::commands::{save_open_vault, with_vault};
use crate::dto::{AddedItem, Copied, EditField, ItemDetail, ItemSummary, NewField, Revealed};
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, now_ms};

/// How long a revealed field stays revealed — R-12.
///
/// Ten seconds, not sixty, and the reason is in the contract: the frontend dropping its copy
/// on `field-remasked` is the one place the security depends on the untrusted side behaving.
/// A short window is what keeps that dependence small.
const REVEAL_SECONDS: u64 = 10;

/// **Vault-class.** Every item, secrets elided — R-11.
#[tauri::command(rename_all = "snake_case")]
pub fn list_items(state: State<'_, AppState>) -> IpcResult<Vec<ItemSummary>> {
    list_items_inner(&state)
}

/// The body of [`list_items`], reachable without a Tauri runtime.
///
/// Every command is split like this so `tests/ipc_audit.rs` can call it. The harness has to
/// drive the *real* command body — a harness that tests a reimplementation of the boundary
/// proves only that the reimplementation is safe.
pub fn list_items_inner(state: &AppState) -> IpcResult<Vec<ItemSummary>> {
    with_vault(state, |vault, _| {
        Ok(vault.items().map(ItemSummary::elide).collect())
    })
}

/// **Vault-class.** One item with its fields, secrets elided.
#[tauri::command(rename_all = "snake_case")]
pub fn get_item(state: State<'_, AppState>, item_id: ItemId) -> IpcResult<ItemDetail> {
    get_item_inner(&state, item_id)
}

/// The body of [`get_item`], reachable without a Tauri runtime.
pub fn get_item_inner(state: &AppState, item_id: ItemId) -> IpcResult<ItemDetail> {
    with_vault(state, |vault, _| {
        vault
            .item(item_id)
            .map(ItemDetail::elide)
            .ok_or_else(|| IpcError::new(ErrorKind::NoSuchItem))
    })
}

/// **Sanctioned.** Returns one field's plaintext and schedules the remask — R-10, R-12, R-13.
///
/// The only command in the application that returns a stored secret. Three things happen here
/// and they are one operation on purpose:
///
/// 1. The core hands over exactly one field, refusing any field not marked secret.
/// 2. The core records the reveal if the audit setting is on — it takes the flag itself, so a
///    caller cannot read a value without recording it.
/// 3. **The host** starts the remask timer. Not the frontend: a frontend timer is cleared by a
///    reload, and a reload must not extend a reveal.
#[tauri::command(rename_all = "snake_case")]
pub fn reveal_field(
    app: AppHandle,
    state: State<'_, AppState>,
    item_id: ItemId,
    field_id: FieldId,
) -> IpcResult<Revealed> {
    let (revealed, generation) = reveal_field_inner(&state, item_id, field_id)?;
    schedule_remask(app, item_id, field_id, generation);
    Ok(revealed)
}

/// The body of [`reveal_field`], minus the timer, reachable without a Tauri runtime.
///
/// Returns the generation alongside so the caller can schedule a remask that knows whether it
/// has gone stale.
pub fn reveal_field_inner(
    state: &AppState,
    item_id: ItemId,
    field_id: FieldId,
) -> IpcResult<(Revealed, u64)> {
    let (value, generation) = with_vault(state, |vault, inner| {
        let secret = vault.reveal_field(item_id, field_id, inner.settings.audit_log_enabled)?;
        Ok((secret.expose().to_owned(), inner.generation))
    })?;

    let remask_at = now_ms() + i64::try_from(REVEAL_SECONDS * 1000).unwrap_or(0);
    Ok((Revealed { value, remask_at }, generation))
}

/// Emits `field-remasked` after the reveal window, unless the vault changed underneath.
///
/// The generation check is why a lock during a reveal does not produce an event about an item
/// in a vault that is no longer open.
fn schedule_remask(app: AppHandle, item_id: ItemId, field_id: FieldId, generation: u64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(REVEAL_SECONDS));
        let state = app.state::<AppState>();
        let current = state
            .with(|inner| inner.generation)
            .unwrap_or(generation + 1);
        if current == generation {
            let _ = app.emit("field-remasked", FieldEvent { item_id, field_id });
        }
    });
}

/// **Vault-class.** Copies one field to the clipboard and returns **no value** — R-10, R-14.
///
/// This is the command the whole contract is shaped around. Rust reads the secret, writes it
/// to the system clipboard, and drops it; the webview receives only the instant of the clear.
/// The secret does not enter the JavaScript heap in any form, which is the only version of
/// "copy a password" that is safe on a platform whose heap cannot be wiped.
///
/// Not sanctioned, because nothing secret is in the response — which is the point.
#[tauri::command(rename_all = "snake_case")]
pub fn copy_field(
    app: AppHandle,
    state: State<'_, AppState>,
    item_id: ItemId,
    field_id: FieldId,
) -> IpcResult<Copied> {
    let (copied, seconds, generation) = copy_field_inner(&state, item_id, field_id)?;
    schedule_clear(app, item_id, field_id, seconds, generation);
    Ok(copied)
}

/// The body of [`copy_field`], minus the timer, reachable without a Tauri runtime.
pub fn copy_field_inner(
    state: &AppState,
    item_id: ItemId,
    field_id: FieldId,
) -> IpcResult<(Copied, u64, u64)> {
    let (seconds, generation) = with_vault(state, |vault, inner| {
        // Copying reveals to the clipboard, so it is audited exactly like a reveal is. A copy
        // that went unlogged while a reveal was logged would make the log worse than useless:
        // it would look complete.
        let secret = vault.reveal_field(item_id, field_id, inner.settings.audit_log_enabled)?;
        clipboard::set(secret.expose())?;
        Ok((inner.settings.clipboard_clear_seconds, inner.generation))
    })?;

    let clears_at = now_ms() + i64::try_from(seconds * 1000).unwrap_or(0);
    Ok((Copied { clears_at }, seconds, generation))
}

/// Clears the clipboard after the configured window and announces it.
///
/// The clear is **not** gated on the generation: a secret on the clipboard must be cleared
/// whether or not the vault has since locked — locking is a reason to clear sooner, never a
/// reason to skip it. Only the cosmetic event is gated.
fn schedule_clear(
    app: AppHandle,
    item_id: ItemId,
    field_id: FieldId,
    seconds: u64,
    generation: u64,
) {
    thread::spawn(move || {
        clear_after(seconds);
        let state = app.state::<AppState>();
        let current = state
            .with(|inner| inner.generation)
            .unwrap_or(generation + 1);
        if current == generation {
            let _ = app.emit("clipboard-cleared", FieldEvent { item_id, field_id });
        }
    });
}

/// Waits out the window and clears the clipboard — the whole of S-11 and R-14.
///
/// Split from [`schedule_clear`] so the gate can put a stopwatch on the **real** code path
/// rather than on a re-implementation of it; what stays behind is the cosmetic event, which
/// needs a Tauri runtime. `tests/clipboard_clear.rs` measures this function.
pub fn clear_after(seconds: u64) {
    thread::sleep(Duration::from_secs(seconds));
    clipboard::clear();
}

/// **Vault-class.** Creates an item and saves the vault — R-17, `docs/ipc-contract.md` §6.4.
///
/// Returns the identifier and nothing else: the caller re-reads through [`get_item`], so there
/// is one elision path in the application rather than two that can drift apart.
#[tauri::command(rename_all = "snake_case")]
pub fn add_item(
    state: State<'_, AppState>,
    kind: ItemKind,
    title: String,
    tags: Vec<String>,
    fields: Vec<NewField>,
) -> IpcResult<AddedItem> {
    add_item_inner(&state, kind, title, tags, fields)
}

/// The body of [`add_item`], reachable without a Tauri runtime.
pub fn add_item_inner(
    state: &AppState,
    kind: ItemKind,
    title: String,
    tags: Vec<String>,
    fields: Vec<NewField>,
) -> IpcResult<AddedItem> {
    with_vault(state, |vault, inner| {
        let item_id = vault.add_item(kind, title);
        let Some(item) = vault.item_mut(item_id) else {
            // Unreachable: the item was added one line ago. Reported rather than unwrapped
            // because this crate's failure mode is a user's vault, not a stack trace.
            return Err(IpcError::new(ErrorKind::Internal));
        };
        item.set_tags(tags);
        for field in fields {
            // `push_field`, not `set_field`: the item was created empty a moment ago, so
            // there is nothing to merge with, and merging by label is the behaviour D-43
            // took off this path entirely.
            item.push_field(field.into_field());
        }
        save_open_vault(vault, inner)?;
        Ok(AddedItem { item_id })
    })
}

/// **Vault-class.** Rewrites an item from an edit form and saves — R-17, §6.4.
///
/// The three rules that are silent when they go wrong live in the core
/// ([`trustvault_core::Item::apply_edits`]): a `null` value keeps the stored one, omission
/// deletes, and the list is the display order.
#[tauri::command(rename_all = "snake_case")]
pub fn update_item(
    state: State<'_, AppState>,
    item_id: ItemId,
    title: String,
    tags: Vec<String>,
    favourite: bool,
    fields: Vec<EditField>,
) -> IpcResult<()> {
    update_item_inner(&state, item_id, title, tags, favourite, fields)
}

/// The body of [`update_item`], reachable without a Tauri runtime.
pub fn update_item_inner(
    state: &AppState,
    item_id: ItemId,
    title: String,
    tags: Vec<String>,
    favourite: bool,
    fields: Vec<EditField>,
) -> IpcResult<()> {
    // Converted before the vault is touched, so a request the wire shape allows and the model
    // has no meaning for — create a field whose value is unchanged — is refused with nothing
    // written. It is `internal` because the only caller is our own webview: a request in that
    // shape is a bug here, not something a user did.
    let edits = fields
        .into_iter()
        .map(EditField::into_edit)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| IpcError::new(ErrorKind::Internal))?;

    with_vault(state, |vault, inner| {
        let item = vault
            .item_mut(item_id)
            .ok_or_else(|| IpcError::new(ErrorKind::NoSuchItem))?;
        // Fields first. `apply_edits` validates the whole list before writing anything, so a
        // refused edit leaves the item exactly as it was — and that guarantee is only worth
        // having if the title and tags have not already been written by then.
        item.apply_edits(edits)?;
        item.set_title(title);
        item.set_tags(tags);
        item.set_favourite(favourite);
        save_open_vault(vault, inner)
    })
}

/// **Vault-class.** Deletes an item and saves — R-17, R-18, §6.4.
///
/// The confirmation R-18 requires is in the **UI**, not here, and the contract says so
/// explicitly: what it protects against is a mis-click, and the caller is the only user. The
/// opposite case is `delete_vault`, which does verify, because there the typed name is the
/// whole ceremony.
#[tauri::command(rename_all = "snake_case")]
pub fn delete_item(state: State<'_, AppState>, item_id: ItemId) -> IpcResult<()> {
    delete_item_inner(&state, item_id)
}

/// The body of [`delete_item`], reachable without a Tauri runtime.
pub fn delete_item_inner(state: &AppState, item_id: ItemId) -> IpcResult<()> {
    with_vault(state, |vault, inner| {
        // The removed item is dropped here, which zeroizes every value it held — including
        // the history entries, which are as sensitive as the fields they came from.
        if vault.remove_item(item_id).is_none() {
            return Err(IpcError::new(ErrorKind::NoSuchItem));
        }
        save_open_vault(vault, inner)
    })
}

/// Payload of `field-remasked` and `clipboard-cleared`.
///
/// Two identifiers and nothing else. An event carries no `Secret` — there is no user action
/// behind one, so the contract's "only on explicit user action" rule could not be satisfied.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct FieldEvent {
    /// Which item.
    pub item_id: ItemId,
    /// Which field of it.
    pub field_id: FieldId,
}
