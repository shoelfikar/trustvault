//! The IPC audit harness — `docs/ipc-contract.md` §9.
//!
//! This is the enforcement behind the contract. Every rule the contract states as MUST is
//! either checked here or explicitly marked in the contract as unchecked; a rule nobody can
//! check is a comment.
//!
//! The harness calls the **real command bodies** through their `_inner` functions. A harness
//! that reimplemented the boundary would prove only that the reimplementation is safe.

// An integration test is its own crate with no `#[cfg(test)]` module, so clippy's
// `allow-unwrap-in-tests` does not reach it and the workspace's Tier-1 lints apply at full
// force. Lifted here, with the reason, exactly as crates/trustvault-core/tests/ does.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use trustvault_core::{FieldId, ItemId, ItemKind, KdfParams, Vault};
use trustvault_lib::commands::{items, vault as vault_cmd};
use trustvault_lib::dto::MASK;
use trustvault_lib::error::ErrorKind;
use trustvault_lib::state::AppState;

/// The secret planted in every fixture. If this string appears in any payload that is not a
/// sanctioned response, the boundary leaked.
const SECRET: &str = "correct-horse-battery-staple";
/// A non-secret field value, which is allowed to cross freely (`docs/ipc-contract.md` §6.1).
const USERNAME: &str = "octocat";

/// An unlocked state holding one login with one secret and one public field.
fn unlocked() -> (AppState, ItemId, FieldId, FieldId) {
    let (mut vault, _) =
        Vault::create("Personal Vault", "master pw", KdfParams::TESTING).expect("valid params");
    let item = vault.add_item(ItemKind::Login, "GitHub");
    let entry = vault.item_mut(item).expect("just added");
    let public = entry.set_field("Username", USERNAME, false);
    let secret = entry.set_field("Password", SECRET, true);

    let state = AppState::default();
    state.with(|inner| {
        inner.vault = Some(vault);
        inner.path = Some(PathBuf::from("/tmp/personal.tvault"));
    });
    (state, item, secret, public)
}

/// Reads the command list out of the `generate_handler!` block in `src/lib.rs`.
///
/// Parsed from source rather than maintained as a second list, because a second list is one
/// that drifts. It is only ever compared against the contract, never used to dispatch.
fn registered_commands() -> Vec<String> {
    let source = include_str!("../src/lib.rs");
    let start = source
        .find("generate_handler![")
        .expect("lib.rs registers commands");
    let block = &source[start..];
    let end = block.find(']').expect("the handler list is closed");
    block[..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            // Command paths only: skip the macro line itself and the section comments.
            line.strip_prefix("commands::")
                .and_then(|path| path.rsplit("::").next())
                .map(str::to_owned)
        })
        .collect()
}

/// Reads the command names the contract documents, from its fenced `ts` blocks.
fn documented_commands() -> Vec<String> {
    let contract = include_str!("../../docs/ipc-contract.md");
    let mut names = Vec::new();
    for line in contract.lines() {
        // A command declaration in the contract looks like `name({...}): Shape` or `name():`.
        let Some(open) = line.find('(') else { continue };
        let name = &line[..open];
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
            && line[open..].contains("):")
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Check 1 — the registered set and the documented set are the same set.
///
/// The failure this exists for is a command that exists but is undocumented: it is how a
/// fourth sanctioned command arrives without anyone deciding to add one.
#[test]
fn every_command_is_documented_and_every_documented_command_exists() {
    let mut registered = registered_commands();
    registered.sort();
    let documented = documented_commands();

    let undocumented: Vec<_> = registered
        .iter()
        .filter(|name| !documented.contains(name))
        .collect();
    assert!(
        undocumented.is_empty(),
        "commands registered but absent from docs/ipc-contract.md: {undocumented:?}"
    );

    let unimplemented: Vec<_> = documented
        .iter()
        .filter(|name| !registered.contains(name))
        .collect();
    assert!(
        unimplemented.is_empty(),
        "commands documented but not registered: {unimplemented:?}"
    );
}

/// Check 3 — exactly three commands may return a secret, and they are the named three.
#[test]
fn the_sanctioned_set_is_exactly_three_and_unchanged() {
    let contract = include_str!("../../docs/ipc-contract.md");
    for name in ["create_vault", "unlock_recovery_kit", "reveal_field"] {
        assert!(
            contract.contains(name),
            "{name} is a sanctioned command and must stay documented"
        );
    }
    assert!(
        contract.contains("There are **three** sanctioned commands"),
        "the budget of three is the contract's load-bearing sentence; if it changed, \
         a decision log entry should have changed with it"
    );
}

/// Checks 2 and 4 — a list carries no secret, and a detail carries at most a mask.
#[test]
fn no_list_or_detail_response_carries_a_secret() {
    let (state, item, _, _) = unlocked();

    let list = serde_json::to_string(&items::list_items_inner(&state).unwrap()).unwrap();
    assert!(!list.contains(SECRET), "list_items leaked the password");
    assert!(
        !list.contains(USERNAME),
        "list_items carries no field values at all, secret or not"
    );

    let detail = serde_json::to_string(&items::get_item_inner(&state, item).unwrap()).unwrap();
    assert!(!detail.contains(SECRET), "get_item leaked the password");
    assert!(detail.contains(MASK), "the secret field crosses as a mask");
    assert!(
        detail.contains(USERNAME),
        "a field the user declared is not secret crosses as its value"
    );
    assert!(
        !detail.contains("history"),
        "history is absent from the shape, not merely elided"
    );
}

/// Check 2 — the one sanctioned response carries exactly one secret and nothing else.
#[test]
fn reveal_returns_one_secret_and_copy_returns_none() {
    let (state, item, secret, _) = unlocked();

    let (revealed, _) = items::reveal_field_inner(&state, item, secret).unwrap();
    assert_eq!(revealed.value, SECRET, "the sanctioned path does reveal it");
    let encoded = serde_json::to_string(&revealed).unwrap();
    assert_eq!(
        encoded.matches(SECRET).count(),
        1,
        "exactly one secret per invocation (R-10)"
    );

    // copy_field is not exercised here: it writes to the real system clipboard, which a CI
    // runner may not have. Its response *shape* carries no value by construction -- `Copied`
    // has one field and it is an integer -- and that is asserted in the unit tests instead.
    // Named rather than silently skipped, because a check that quietly does not run is worse
    // than one that is documented as not running.
}

/// Check 6 — every vault-class and sanctioned command answers `locked` when it is.
///
/// Worth more than it looks: this is the regression test for a webview reload. A reload leaves
/// the frontend's stores empty and its lock state whatever the host says, and the bug it
/// prevents is a command that reads a cached handle instead of re-checking.
#[test]
fn every_vault_command_refuses_while_locked() {
    let (state, item, secret, _) = unlocked();
    state.lock();

    assert_eq!(
        items::list_items_inner(&state).unwrap_err().kind,
        ErrorKind::Locked
    );
    assert_eq!(
        items::get_item_inner(&state, item).unwrap_err().kind,
        ErrorKind::Locked
    );
    assert_eq!(
        items::reveal_field_inner(&state, item, secret)
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
    assert_eq!(
        items::copy_field_inner(&state, item, secret)
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
}

/// A reload does not unlock anything, and nothing the frontend does can change that.
///
/// There is no command that sets the lock state to unlocked without a credential — `unlock`
/// and `unlock_recovery_kit` both go through the KDF. This test states the property by
/// showing that the state survives being read repeatedly and stays locked.
#[test]
fn a_locked_state_stays_locked_however_often_it_is_asked() {
    let (state, _, _, _) = unlocked();
    state.lock();

    for _ in 0..5 {
        let status = state.status();
        assert_eq!(
            status.state,
            trustvault_lib::dto::VaultState::Locked,
            "asking does not unlock"
        );
        assert_eq!(status.item_count, None);
    }
}

/// The reveal is recorded only when the setting is on, and the log never holds the value.
#[test]
fn the_audit_log_follows_the_setting_and_holds_no_secret() {
    let (state, item, secret, _) = unlocked();

    // Off by default (D-31).
    items::reveal_field_inner(&state, item, secret).unwrap();
    let recorded = state
        .with(|inner| inner.vault.as_ref().map(|v| v.audit_entries().len()))
        .flatten();
    assert_eq!(recorded, Some(0), "off by default means nothing is written");

    state.with(|inner| inner.settings.audit_log_enabled = true);
    items::reveal_field_inner(&state, item, secret).unwrap();

    let encoded = state
        .with(|inner| {
            inner
                .vault
                .as_ref()
                .map(|v| serde_json::to_string(v.audit_entries()).unwrap())
        })
        .flatten()
        .unwrap();
    assert!(encoded.contains(&item.to_string()), "the item is named");
    assert!(!encoded.contains(SECRET), "the value never is");
    assert!(!encoded.contains("Password"), "nor the label");
}

/// Revealing a field that is not secret is refused, so "reveal" means one thing everywhere.
#[test]
fn a_public_field_cannot_be_revealed() {
    let (state, item, _, public) = unlocked();
    assert_eq!(
        items::reveal_field_inner(&state, item, public)
            .unwrap_err()
            .kind,
        ErrorKind::NotSecret
    );
}

/// N-07 — the CSP names no wildcard origin, and the one relaxation stays the only one.
///
/// `style-src 'unsafe-inline'` is present because Svelte injects component styles, and that is
/// recorded as the single deliberate relaxation. Widening `script-src`, `connect-src`, or
/// `font-src` needs a decision log entry, so this test is what makes "needs an entry" cost
/// something.
#[test]
fn the_csp_has_no_wildcard_origin() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid config");
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("a CSP is configured");

    assert!(!csp.contains('*'), "no wildcard origin anywhere: {csp}");
    for directive in [
        "default-src 'self'",
        "script-src 'self'",
        "font-src 'self'",
        "object-src 'none'",
        "frame-src 'none'",
        "base-uri 'none'",
        "form-action 'none'",
    ] {
        assert!(csp.contains(directive), "missing `{directive}` in the CSP");
    }
    assert_eq!(
        csp.matches("unsafe-inline").count(),
        1,
        "exactly one relaxation, and it is style-src for Svelte's injected component styles"
    );
    assert!(
        csp.contains("style-src 'self' 'unsafe-inline'"),
        "the one relaxation is on style-src and nowhere else"
    );
}

/// The lock event's reason survives serialization, since the lock screen routes on it.
#[test]
fn the_lock_event_names_its_reason() {
    let encoded = serde_json::to_string(&vault_cmd::LockEvent {
        reason: trustvault_lib::state::LockReason::Timeout,
    })
    .unwrap();
    assert!(encoded.contains("timeout"));
}
