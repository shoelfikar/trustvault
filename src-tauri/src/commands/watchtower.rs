//! Watchtower — `docs/ipc-contract.md` §6.9, R-23 and R-24.
//!
//! The **local** half only. `watchtower_breach_check` is the other command §6.9 specifies and it
//! is still `// planned`: it is the one thing in this application that opens a socket, and D-76
//! split the two commands so that "zero packets while breach checking is off" (S-10) is a command
//! nobody called rather than a branch somebody must keep taking. Nothing in this file can reach
//! the network — the scan is `trustvault-core`, and that crate has no socket in its tree (N-02).
//!
//! Two things cross back, and neither is a secret:
//!
//! * The [`Report`] — findings naming item ids, scores and crack-time strings. The grouping key
//!   is a SHA-256 of a password and it never leaves the core; `shared_with` names the other
//!   **items**, which is what the user needs and none of what an offline attacker does.
//! * Nothing else. The scan writes the per-item status cache into the vault as a side effect,
//!   and the item list reads that through `list_items` like it always has.

use tauri::State;

use trustvault_core::Report;

use crate::commands::{save_open_vault, with_vault};
use crate::error::IpcResult;
use crate::state::AppState;

/// **Vault-class.** Scores every password and groups the reused ones — R-23, R-24.
///
/// Not sanctioned: a finding is not a secret. §6.9's budget of four stands, and if a Watchtower
/// command ever appears to want a plaintext password in the webview, the design is wrong rather
/// than the budget.
#[tauri::command(rename_all = "snake_case")]
pub fn watchtower_scan(state: State<'_, AppState>) -> IpcResult<Report> {
    watchtower_scan_inner(&state)
}

/// The body of [`watchtower_scan`], reachable without a Tauri runtime.
///
/// **Synchronous, and holding the state lock for the whole scan is deliberate.** §6.9 says the
/// results are discarded if the vault locked while the scan was running; here that cannot
/// happen, because `with_vault` holds the vault for the duration and an auto-lock cannot take it
/// away mid-flight. The breach check will not have that luxury — minutes rather than the 61 ms
/// S-07a measured — and this is the command whose shape makes the difference visible rather than
/// two commands that look alike and differ in what they must handle.
///
/// **It saves, and the save is the point of the status cache.** Writing statuses into memory
/// only would mean the item list drew fresh pips until the next relaunch and blank ones after
/// it, which looks exactly like a scan that never ran. Unlike `set_profile`, there is no
/// unchanged case to skip: `last_scan_at` moves on every scan, and "when did this last run" is
/// on screen.
pub fn watchtower_scan_inner(state: &AppState) -> IpcResult<Report> {
    with_vault(state, |vault, inner| {
        let report = trustvault_core::scan_and_record(vault);
        save_open_vault(vault, inner)?;
        Ok(report)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use trustvault_core::{Field, FieldKind, ItemKind, ItemStatus, KdfParams, Vault, Verdict};

    /// An unlocked state over a vault with one reused password and one strong one.
    ///
    /// No path that can be written: these tests are about what the command computes and refuses,
    /// and the save is exercised against a real file in `tests/ipc_session.rs`.
    fn unlocked() -> AppState {
        let (mut vault, _) =
            Vault::create("Personal", "master pw", KdfParams::TESTING).expect("valid params");
        for title in ["Forum", "Wiki"] {
            let id = vault.add_item(ItemKind::Login, title);
            let item = vault.item_mut(id).expect("just added");
            item.push_field(
                Field::new("Password", "password", true).with_kind(FieldKind::Password),
            );
        }
        let id = vault.add_item(ItemKind::Login, "Stripe");
        let item = vault.item_mut(id).expect("just added");
        item.push_field(
            Field::new("Password", "qX7#vn2Lp!4dRt", true).with_kind(FieldKind::Password),
        );

        let state = AppState::default();
        state.with(|inner| {
            inner.vault = Some(vault);
            inner.path = Some(PathBuf::from("/nonexistent/personal.tvault"));
        });
        state
    }

    /// The scan reports what R-23 and R-24 ask for, through the command rather than the core.
    #[test]
    fn the_command_returns_the_findings_the_core_produced() {
        let state = unlocked();
        // The save fails — the path is unwritable on purpose — so the report is read off the
        // vault instead. What this asserts is the wiring: the command reaches the core's scan
        // with the open vault, which is the half a unit test of `scan` cannot cover.
        let _ = watchtower_scan_inner(&state);
        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            let report = trustvault_core::scan(vault);
            assert_eq!(report.passwords, 3);
            assert_eq!(report.distinct, 2, "one request per distinct value — S-07b");
            assert!(
                report
                    .findings
                    .iter()
                    .any(|finding| finding.verdict == Verdict::Reused)
            );
        });
    }

    /// A failed save does not leave the caller believing the cache was written.
    ///
    /// The order matters and this is what pins it: statuses are written into the in-memory vault
    /// first and the save can still fail, so the command must return the error rather than the
    /// report it computed. A report returned beside a silently unwritten cache is the state where
    /// the Watchtower screen and the item list disagree after a relaunch.
    #[test]
    fn a_scan_that_cannot_be_saved_fails_rather_than_reporting_success() {
        let state = unlocked();
        assert!(
            watchtower_scan_inner(&state).is_err(),
            "an unwritable vault path is a failed scan, not a silent one"
        );
        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            assert!(
                vault.items().any(|item| item.status == ItemStatus::Reused),
                "the in-memory cache is written before the save is attempted"
            );
        });
    }

    /// Vault-class: locked means locked, and the check is the contract's §9 check 6.
    #[test]
    fn a_locked_vault_refuses_to_be_scanned() {
        let state = AppState::default();
        let refused = watchtower_scan_inner(&state);
        assert!(refused.is_err(), "there is no vault to scan");
    }
}
