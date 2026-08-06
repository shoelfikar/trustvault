//! The command palette's query — `docs/ipc-contract.md` §6.5, R-16, D-46.
//!
//! One command, and the whole of it is a call into the core. That is the point of D-46: the
//! query crosses **inbound**, the matching happens against plaintext that never leaves
//! `trustvault-core`, and what comes back is the same elided summary the item list already
//! gets — no field values, including the one that matched.
//!
//! The alternative was to widen `ItemSummary` with `username` and `url` and filter in
//! JavaScript. It is a *permitted* crossing under §6.1, which is what made it worth a decision
//! rather than a rule: a non-secret value may cross freely, so nothing forbade it. It was still
//! the wrong trade, because it puts the vault's entire identifying surface into a heap that
//! cannot be wiped, on every render, to save one round trip on a 50 ms budget (S-04) that the
//! matching itself spends 0.85 ms of.

use tauri::State;

use crate::commands::with_vault;
use crate::dto::ItemSummary;
use crate::error::IpcResult;
use crate::state::AppState;

/// The most results one query may return.
///
/// The palette draws six rows and the contract lets the caller name a limit, so this is a
/// ceiling rather than the number: what it stops is a caller asking for the whole vault through
/// a command whose response shape nobody reviews as a list. `list_items` is the way to get the
/// list, and it is the one place that elision path is exercised in bulk.
const MAX_RESULTS: usize = 50;

/// **Vault-class.** Ranks the vault against a query, secrets elided — R-16, D-46.
#[tauri::command(rename_all = "snake_case")]
pub fn search_items(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> IpcResult<Vec<ItemSummary>> {
    search_items_inner(&state, &query, limit)
}

/// The body of [`search_items`], reachable without a Tauri runtime.
pub fn search_items_inner(
    state: &AppState,
    query: &str,
    limit: usize,
) -> IpcResult<Vec<ItemSummary>> {
    with_vault(state, |vault, _| {
        Ok(vault
            .search(query, limit.min(MAX_RESULTS))
            .into_iter()
            .map(ItemSummary::elide)
            .collect())
    })
}
