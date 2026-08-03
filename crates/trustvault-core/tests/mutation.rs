//! R-04: every single flipped byte must cause a decrypt failure — never a partial read, never
//! garbage, never a silent parameter change.
//!
//! This is the test the exit gate asks for by name, and it is the one that turns "every byte
//! is covered by at least one tag" from an argument in `docs/vault-format.md` §4 into a fact
//! about this build.
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

use trustvault_core::{Error, HEADER_LEN, KdfParams, Vault, WRAP_AAD_LEN};

use common::{PARAMS, PASSWORD};

/// A deliberately small vault: the test flips every bit of every byte, so the file size is a
/// multiplier on the runtime.
fn small_vault() -> Vec<u8> {
    vault_with(PARAMS)
}

fn vault_with(params: KdfParams) -> Vec<u8> {
    let (mut vault, _) = Vault::create("Mutation", PASSWORD, params).expect("create");
    let id = vault.add_item(trustvault_core::ItemKind::Login, "One");
    vault
        .item_mut(id)
        .expect("just added")
        .set_field("Password", "hunter2", true);
    vault.to_bytes().expect("seal")
}

#[test]
fn every_flipped_byte_fails_to_decrypt() {
    let original = small_vault();
    assert!(
        Vault::open(&original, PASSWORD).is_ok(),
        "the unmutated fixture must open, or this test proves nothing"
    );

    let mut checked = 0usize;
    for index in 0..original.len() {
        for bit in 0..8u8 {
            let mut mutated = original.clone();
            mutated[index] ^= 1 << bit;

            match Vault::open(&mutated, PASSWORD) {
                Err(_) => checked += 1,
                Ok(_) => panic!(
                    "byte {index} bit {bit} was flipped and the vault still opened — \
                     that byte is not covered by any tag"
                ),
            }
        }
    }

    assert_eq!(checked, original.len() * 8);
    assert!(
        original.len() > HEADER_LEN,
        "the fixture must have a body as well as a header"
    );
}

#[test]
fn a_truncated_file_never_opens() {
    let original = small_vault();
    for length in 0..original.len() {
        assert!(
            Vault::open(&original[..length], PASSWORD).is_err(),
            "a file truncated to {length} bytes must not open"
        );
    }
}

#[test]
fn appended_bytes_never_open() {
    // §2: trailing bytes are corruption, not an extension point. Without the length check in
    // §7 step 4 they would be silently ignored, which is a place to hide data.
    let original = small_vault();
    for extra in 1..=8usize {
        let mut extended = original.clone();
        extended.extend(std::iter::repeat_n(0u8, extra));
        assert!(matches!(
            Vault::open(&extended, PASSWORD),
            Err(Error::NotAVault)
        ));
    }
}

#[test]
fn the_kdf_parameters_cannot_be_downgraded() {
    // R-02, stated as an attack rather than as a property. An attacker with a stolen vault
    // would love to rewrite m_cost to 8 KiB and brute-force cheaply. The parameter block is
    // associated data for both wraps, so the tag check fails before the cheaper KDF is of any
    // use — and the derived key would be wrong regardless.
    // Created with real work behind it, so that rewriting the parameters is a downgrade
    // rather than a no-op.
    let original = vault_with(KdfParams {
        m_cost: 1024,
        t_cost: 2,
        p_cost: 1,
    });
    let mut downgraded = original.clone();

    let weak = KdfParams {
        m_cost: 8,
        t_cost: 1,
        p_cost: 1,
    };
    downgraded[8..12].copy_from_slice(&weak.m_cost.to_le_bytes());
    downgraded[12..16].copy_from_slice(&weak.t_cost.to_le_bytes());
    downgraded[16] = weak.p_cost;

    assert!(matches!(
        Vault::open(&downgraded, PASSWORD),
        Err(Error::Unreadable)
    ));
    const { assert!(WRAP_AAD_LEN == 20, "the parameter block is bytes 0..20") };
}

#[test]
fn the_two_wraps_are_not_interchangeable() {
    // Copying the password wrap over the recovery wrap must not turn the password into a
    // recovery code: the nonces and salts differ, so the AEAD rejects it. If this ever passed,
    // a stolen vault plus a known password would yield a working recovery kit.
    let original = small_vault();
    let mut swapped = original.clone();
    let (nonce_pw, wrap_pw) = (52..76, 76..124);
    let pw_nonce: Vec<u8> = swapped[nonce_pw.clone()].to_vec();
    let pw_wrap: Vec<u8> = swapped[wrap_pw.clone()].to_vec();
    swapped[124..148].copy_from_slice(&pw_nonce);
    swapped[148..196].copy_from_slice(&pw_wrap);

    // The body tag covers the whole header, so the file is rejected outright.
    assert!(matches!(
        Vault::open(&swapped, PASSWORD),
        Err(Error::Unreadable)
    ));
}
