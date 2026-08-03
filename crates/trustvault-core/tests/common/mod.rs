//! Shared fixtures for the verification suite.
//!
//! Every integration-test binary compiles this module separately, so anything one binary does
//! not use looks dead to that binary. The allow is about Rust's test layout, not about unused
//! code.

#![allow(dead_code)]
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

use trustvault_core::{FieldKind, ItemKind, KdfParams, RecoveryCode, Vault};

/// The password every fixture vault uses.
pub const PASSWORD: &str = "correct horse battery staple";

/// Parameters for the whole suite.
///
/// Deliberately weak, and that is not a compromise: the parameters live in the header, so a
/// vault written with them exercises exactly the same code path as one written with the
/// 256 MiB production defaults. What they buy is a suite that runs in seconds, which is what
/// makes a 10 000-case property test and a 100-iteration kill test affordable.
pub const PARAMS: KdfParams = KdfParams::TESTING;

/// A vault with one item of every kind, each carrying a secret and a non-secret field.
pub fn populated_vault() -> (Vault, RecoveryCode) {
    let (mut vault, recovery) =
        Vault::create("Fixture", PASSWORD, PARAMS).expect("fixture parameters are valid");

    for (index, kind) in ItemKind::ALL.into_iter().enumerate() {
        let id = vault.add_item(kind, format!("Item {index}"));
        let item = vault.item_mut(id).expect("just added");
        item.set_field("Username", format!("user{index}"), false);
        item.set_field("Password", format!("secret-{index}"), true);
        item.tags.push(format!("tag{index}"));
        if let Some(field) = item.fields.first_mut() {
            field.kind = FieldKind::Username;
        }
    }

    (vault, recovery)
}
