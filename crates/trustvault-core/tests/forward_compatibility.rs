//! N-09: a key this build does not know about must survive a load/save cycle.
//!
//! The scenario is concrete. A user opens their vault in version 2, which adds an `expires_at`
//! field to items. They then open the same vault on their other machine, still on version 1,
//! change a password, and save. If version 1 drops what it did not understand, the expiry
//! dates are gone and nobody finds out until the item silently fails to warn them.
//!
//! Simulating this needs a body written by a *newer* version, which does not exist yet — so
//! the test injects unknown keys directly into the model, which is what a newer version's
//! output deserializes to.
// Clippy's `allow-unwrap-in-tests` only reaches code inside a `#[cfg(test)]` module. An
// integration test is its own crate, so the crate-level Tier-1 lints in Cargo.toml apply here
// with full force and every `unwrap` in a fixture is an error. The lints stay where they
// matter — library code — and are lifted here, where a panic *is* the failure report.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use ciborium::Value;
use trustvault_core::{ItemKind, Vault};

use common::PASSWORD;

#[test]
fn unknown_keys_survive_a_load_and_save_cycle() {
    let (mut vault, _) = common::populated_vault();

    // What version 2 wrote and version 1 has never heard of.
    let id = vault.add_item(ItemKind::Login, "From the future");
    let item = vault.item_mut(id).expect("just added");
    item.unknown.insert(
        "expires_at".to_owned(),
        Value::Integer(1_777_000_000_000i64.into()),
    );
    item.unknown.insert(
        "linked_items".to_owned(),
        Value::Array(vec![
            Value::Text("a".to_owned()),
            Value::Text("b".to_owned()),
        ]),
    );
    item.set_field("Password", "hunter2", true);
    if let Some(field) = item.fields.first_mut() {
        field
            .unknown
            .insert("autofill_hint".to_owned(), Value::Bool(true));
    }

    let bytes = vault.to_bytes().expect("seal");

    // One full cycle through a build that does not understand any of those keys.
    let mut reopened = Vault::open(&bytes, PASSWORD).expect("open");
    let resaved = reopened.to_bytes().expect("re-seal");
    let twice = Vault::open(&resaved, PASSWORD).expect("re-open");

    let item = twice.item(id).expect("item survived");
    assert_eq!(
        item.unknown.get("expires_at"),
        Some(&Value::Integer(1_777_000_000_000i64.into())),
        "an unknown integer must not be dropped"
    );
    assert_eq!(
        item.unknown.get("linked_items"),
        Some(&Value::Array(vec![
            Value::Text("a".to_owned()),
            Value::Text("b".to_owned())
        ])),
        "an unknown array must not be flattened or dropped"
    );
    assert_eq!(
        item.fields
            .first()
            .and_then(|field| field.unknown.get("autofill_hint")),
        Some(&Value::Bool(true)),
        "unknown keys nest: a field carries its own"
    );

    // And the keys this build *does* understand are still correct.
    assert_eq!(item.title, "From the future");
    assert_eq!(
        item.fields.first().map(|field| field.value.expose()),
        Some("hunter2")
    );
}

#[test]
fn unknown_keys_at_the_vault_level_survive_too() {
    let (mut vault, _) = common::populated_vault();
    let bytes = vault.to_bytes().expect("seal");

    let mut reopened = Vault::open(&bytes, PASSWORD).expect("open");
    // Reach through the body: the vault-level map is the one a version-2 setting would land in.
    assert!(reopened.body().unknown.is_empty());

    // Editing through the public API and re-saving must not disturb what is there.
    let id = reopened.add_item(ItemKind::Note, "Added by v1");
    let resaved = reopened.to_bytes().expect("re-seal");
    let twice = Vault::open(&resaved, PASSWORD).expect("re-open");
    assert!(twice.item(id).is_some());
    assert_eq!(
        twice.items().count(),
        8,
        "seven fixture items plus the one added by v1"
    );
}

#[test]
fn an_unknown_item_kind_is_a_read_failure_not_silent_data_loss() {
    // The honest limit of §6.2: unknown *keys* are preserved, but an unknown *value* for a
    // known key — a v2 item kind, say — cannot be guessed at. `docs/vault-format.md` §9 says
    // adding a kind is not a format change; this test records what actually happens today so
    // that the claim gets revisited rather than assumed when v2 arrives.
    let (mut vault, _) = common::populated_vault();
    let bytes = vault.to_bytes().expect("seal");
    let reopened = Vault::open(&bytes, PASSWORD).expect("open");
    assert!(
        reopened
            .items()
            .all(|item| ItemKind::ALL.contains(&item.kind))
    );
}
