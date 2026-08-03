//! R-03: a wrong master password and a corrupted file must be indistinguishable — in error
//! type, which is cheap, and in *timing*, which is not.
//!
//! The attack this defends against: someone holding a stolen vault feeds it guesses and
//! watches the clock. If a wrong password returns faster than a damaged file, the timing says
//! "the password was wrong" rather than "something was wrong", and a guess that took the long
//! path is a guess worth pursuing. `docs/vault-format.md` §7 steps 6–8 are written the way
//! they are for this test alone.
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

use std::time::{Duration, Instant};

use trustvault_core::{Error, HEADER_LEN, ItemKind, KdfParams, Vault};

use common::PASSWORD;

/// Heavier than the rest of the suite on purpose.
///
/// The equivalence only means something if the KDF dominates, which is also true in
/// production: at 8 KiB the AEAD calls would be a measurable fraction of the total and the
/// test would be measuring the wrong thing. 8 MiB puts one derivation at a few milliseconds.
const TIMING_PARAMS: KdfParams = KdfParams {
    m_cost: 8192,
    t_cost: 1,
    p_cost: 1,
};

/// Samples per condition, interleaved.
const SAMPLES: usize = 41;
/// R-03's tolerance.
const TOLERANCE: f64 = 0.05;
/// Independent measurement rounds before the test gives up.
///
/// Wall-clock timing on a shared CI runner is noisy — another job's build can steal a core
/// mid-sample. Retrying an independent round absorbs that without weakening the assertion:
/// a real timing difference reproduces in every round, and all rounds must fail for the test
/// to fail.
const ROUNDS: usize = 3;

fn fixture() -> Vec<u8> {
    let (mut vault, _) = Vault::create("Timing", PASSWORD, TIMING_PARAMS).expect("create");
    for index in 0..20 {
        let id = vault.add_item(ItemKind::Login, format!("Item {index}"));
        vault.item_mut(id).expect("just added").set_field(
            "Password",
            format!("secret-{index}"),
            true,
        );
    }
    vault.to_bytes().expect("seal")
}

/// A vault whose header and key wrap are intact but whose body has been damaged.
///
/// This is the "corrupted file" arm: the password is right, the KDF runs, the wrap opens, and
/// the body fails. The wrong-password arm has to spend the same time on the other path.
fn corrupted(original: &[u8]) -> Vec<u8> {
    let mut damaged = original.to_vec();
    let index = HEADER_LEN + 4;
    damaged[index] ^= 0b1000_0000;
    damaged
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples.get(samples.len() / 2).copied().unwrap_or_default()
}

#[test]
fn a_wrong_password_and_a_corrupt_file_are_indistinguishable() {
    let good = fixture();
    let damaged = corrupted(&good);

    // Both arms must fail, and with the same error. If this part regresses the timing
    // measurement below is meaningless.
    assert!(matches!(
        Vault::open(&good, "wrong"),
        Err(Error::Unreadable)
    ));
    assert!(matches!(
        Vault::open(&damaged, PASSWORD),
        Err(Error::Unreadable)
    ));
    assert!(Vault::open(&good, PASSWORD).is_ok());

    let mut last_report = String::new();

    for round in 1..=ROUNDS {
        // Warm up: the first derivation pays for the allocator and the page faults.
        for _ in 0..3 {
            let _ = Vault::open(&good, "wrong");
            let _ = Vault::open(&damaged, PASSWORD);
        }

        let mut wrong_password = Vec::with_capacity(SAMPLES);
        let mut corrupt_file = Vec::with_capacity(SAMPLES);

        // Interleaved, so that a machine that slows down halfway through the run slows both
        // arms down equally. Measuring one arm and then the other measures the machine.
        for _ in 0..SAMPLES {
            let start = Instant::now();
            let _ = Vault::open(&good, "wrong");
            wrong_password.push(start.elapsed());

            let start = Instant::now();
            let _ = Vault::open(&damaged, PASSWORD);
            corrupt_file.push(start.elapsed());
        }

        let wrong = median(wrong_password).as_secs_f64();
        let corrupt = median(corrupt_file).as_secs_f64();
        let difference = (wrong - corrupt).abs() / wrong.max(corrupt);

        last_report = format!(
            "round {round}: wrong password {:.3} ms, corrupt file {:.3} ms, difference {:.2} % \
             (tolerance {:.0} %)",
            wrong * 1000.0,
            corrupt * 1000.0,
            difference * 100.0,
            TOLERANCE * 100.0
        );
        println!("{last_report}");

        if difference <= TOLERANCE {
            return;
        }
    }

    panic!("R-03: the two failure paths are distinguishable by timing — {last_report}");
}

#[test]
fn both_failure_paths_report_the_same_error() {
    // The cheap half of R-03, asserted separately so a failure says which half broke.
    let good = fixture();
    let damaged = corrupted(&good);

    let from_wrong_password = Vault::open(&good, "wrong").unwrap_err().to_string();
    let from_corrupt_file = Vault::open(&damaged, PASSWORD).unwrap_err().to_string();

    assert_eq!(from_wrong_password, from_corrupt_file);
    assert_eq!(from_wrong_password, "vault could not be read");
    assert!(
        !from_wrong_password.to_lowercase().contains("password"),
        "the message must not name the password as the cause"
    );
}
