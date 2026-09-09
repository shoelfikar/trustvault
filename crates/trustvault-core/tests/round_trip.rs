//! R-01 and N-09: what goes into a vault comes back out of it, unchanged.
//!
//! The property test generates vaults rather than listing them because the interesting cases
//! are the ones nobody thinks to write down: an empty title, a value that is one byte, a value
//! that is 4 KiB of astral-plane Unicode, an item with no fields at all.
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

use proptest::prelude::*;
use trustvault_core::{Field, FieldKind, Item, ItemKind, ItemStatus, Vault};

use common::{PARAMS, PASSWORD};

/// Strategy: a string that may be empty, long, or full of characters that break naive code.
fn any_text() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        "[a-zA-Z0-9 ]{0,64}",
        // Emoji, combining marks, RTL, and a NUL — CBOR carries all of them, and a format that
        // silently normalizes text is a format that corrupts passwords.
        Just("🔐 café الرقم\u{0301}\u{0000}".to_owned()),
        proptest::collection::vec(any::<char>(), 0..64).prop_map(String::from_iter),
    ]
}

fn any_field() -> impl Strategy<Value = Field> {
    (any_text(), any_text(), any::<bool>()).prop_map(|(label, value, secret)| {
        Field::new(label, value, secret).with_kind(if secret {
            FieldKind::Password
        } else {
            FieldKind::Text
        })
    })
}

fn any_item() -> impl Strategy<Value = Item> {
    (
        prop::sample::select(ItemKind::ALL.as_slice()),
        any_text(),
        proptest::collection::vec(any_field(), 0..6),
        proptest::collection::vec(any_text(), 0..4),
        prop::sample::select(
            [
                ItemStatus::Unknown,
                ItemStatus::Strong,
                ItemStatus::Weak,
                ItemStatus::Reused,
                ItemStatus::Breached,
                ItemStatus::Expired,
            ]
            .as_slice(),
        ),
        any::<bool>(),
    )
        .prop_map(|(kind, title, fields, tags, status, favourite)| {
            let mut item = Item::new(kind, title);
            item.fields = fields;
            item.tags = tags;
            item.status = status;
            item.favourite = favourite;
            item
        })
}

proptest! {
    // 10 000 cases, as R-01 asks for. Affordable only because the fixture KDF parameters are
    // weak; at the production defaults this would take roughly three hours.
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn any_vault_round_trips_through_seal_and_open(items in proptest::collection::vec(any_item(), 0..4)) {
        let (mut vault, _) = Vault::create("Property", PASSWORD, PARAMS)
            .map_err(|_| TestCaseError::fail("vault creation failed"))?;
        for item in &items {
            let id = vault.add_item(item.kind, item.title.clone());
            let stored = vault.item_mut(id).ok_or_else(|| TestCaseError::fail("item vanished"))?;
            stored.fields = item.fields.clone();
            stored.tags = item.tags.clone();
            stored.status = item.status;
            stored.favourite = item.favourite;
        }

        let expected: Vec<_> = vault.items().cloned().collect();
        let bytes = vault.to_bytes().map_err(|_| TestCaseError::fail("seal failed"))?;
        let reopened = Vault::open(&bytes, PASSWORD)
            .map_err(|_| TestCaseError::fail("open failed"))?;
        let actual: Vec<_> = reopened.items().cloned().collect();

        prop_assert_eq!(actual, expected);
    }
}

#[test]
fn a_vault_survives_repeated_save_and_reopen_cycles() {
    // Each cycle draws a new body nonce, so this exercises a path a single round trip does
    // not: the one where the *previous* ciphertext is irrelevant to the next.
    let (mut vault, _) = common::populated_vault();
    let mut bytes = vault.to_bytes().expect("seal");

    for cycle in 0..10 {
        let mut reopened = Vault::open(&bytes, PASSWORD).expect("open");
        assert_eq!(reopened.items().count(), 7, "cycle {cycle}");
        bytes = reopened.to_bytes().expect("re-seal");
    }
}

#[test]
fn a_profile_survives_a_round_trip_and_an_empty_one_stays_empty() {
    // D-70. The profile is the only thing in the body that is neither an item nor bookkeeping,
    // so it is the one field a seal/open cycle could drop without any test noticing.
    let (mut vault, _) = common::populated_vault();
    assert!(vault.profile().is_empty(), "a new vault has no owner named");

    let bytes = vault.to_bytes().expect("seal");
    let untouched = Vault::open(&bytes, PASSWORD).expect("open");
    assert!(untouched.profile().is_empty());

    assert!(vault.set_profile("Budi Santoso", "budi@warungpintar.id"));
    let bytes = vault.to_bytes().expect("re-seal");
    let reopened = Vault::open(&bytes, PASSWORD).expect("re-open");
    assert_eq!(reopened.profile().name, "Budi Santoso");
    assert_eq!(reopened.profile().email, "budi@warungpintar.id");
}

#[test]
fn every_item_kind_survives_a_round_trip() {
    // R-01 names all seven explicitly, so they are asserted explicitly rather than trusted to
    // the generator's sampling.
    let (mut vault, _) = common::populated_vault();
    let bytes = vault.to_bytes().expect("seal");
    let reopened = Vault::open(&bytes, PASSWORD).expect("open");

    let kinds: Vec<ItemKind> = reopened.items().map(|item| item.kind).collect();
    assert_eq!(kinds, ItemKind::ALL.to_vec());

    for item in reopened.items() {
        assert_eq!(item.fields.len(), 2);
        assert!(item.fields.iter().any(|field| field.secret));
        assert!(item.fields.iter().any(|field| !field.secret));
    }
}
