//! S-07a: how long does the **local** Watchtower scan of the reference vault take?
//!
//! Run with `cargo bench --bench watchtower`. The number this prints is the one quoted in the
//! success-criteria table of `trustvault-requirements.md` — re-run it and update the table when
//! the scoring changes or when the measurement moves to another machine.
//!
//! # Why this benchmark is the whole of S-07a and none of S-07b
//!
//! S-07 was one criterion until 2026-08-15 and it could not be measured: a full scan *including*
//! HIBP is ≈ 400 s at the rate the live service gives, against a written budget of 10 s (D-77,
//! and the measurements are in `docs/ipc-contract.md` §6.9). What that number measures is the
//! machine's link and HIBP's cache, not this code. The split put the wall clock on the half this
//! code owns: **no socket is opened here and none can be** — `trustvault-core` has no network
//! stack in its dependency tree (N-02), so "with the network untouched" is a property of the
//! crate rather than a condition of the run.
//!
//! # Two vaults, because one number would be a lie in either direction
//!
//! - The **reference vault** is what the criterion names: 1 000 items, 1 000 distinct passwords,
//!   all of the shape `pw-{index}-xK9`. It is zxcvbn's **cheap** case — nothing matches a
//!   dictionary, so the matchers exit early — and it has no reuse to group. That caveat travels
//!   with the number wherever it is quoted.
//! - The **audit vault** is the shape a real one has: reuse groups, dictionary words, a password
//!   at every score. It is 21 items, so it says nothing about scale; what it says is how much
//!   more a *matching* password costs than an unmatched one, which is the factor by which the
//!   reference number understates a real vault.
//!
//! No criterion framework, for `kdf.rs`'s reason: this is one wall-clock median.

// A benchmark is its own crate, so the library's Tier-1 lints apply at full force. Lifted with
// the reason the integration tests give: in a measurement harness a panic is the failure report.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::process::ExitCode;
use std::time::Instant;

use trustvault_core::auditfixture::audit_vault;
use trustvault_core::benchfixture::{ITEMS, reference_vault};
use trustvault_core::{KdfParams, Vault, scan};

/// S-07a's budget for a 1 000-item vault, in milliseconds — **set from the first measurement**,
/// which is what D-77 left blank and this benchmark is here to fill.
///
/// 62.6 ms was measured on 2026-08-15 (Ubuntu 26.04 x86-64, median of 5). The budget is not
/// that number, and the gap is written down rather than being slack nobody can account for:
///
/// - **×1.9 for zxcvbn's expensive case.** The reference vault is its cheap one — no
///   `pw-{index}-xK9` matches a dictionary, so the matchers exit early. The audit vault, whose
///   passwords are the kind people actually choose, costs 1.9× per password. A budget set at the
///   cheap number would fail on the first real vault and blame the wrong thing.
/// - **×2 for a machine that is not this desktop.** The same allowance every other measured
///   criterion carries when it moves to a CI runner.
/// - **The rest is the line at which a scan needs a progress indicator and has none.** Progress
///   events exist for the breach half only (`docs/ipc-contract.md` §8): the local scan is a
///   command that returns, so it must stay under the point where a returning command reads as a
///   hang. Half a second is that point, and it is the same order as S-03's deliberate 511 ms
///   unlock — the scan is never what makes opening a vault feel slow.
///
/// Exceeding it is a build failure rather than a printed regret, which is what makes this a
/// criterion instead of a number in a document.
const BUDGET_MS: f64 = 500.0;

/// Discarded runs. zxcvbn builds its matchers lazily, so the first scan pays for a dictionary
/// every later one reuses.
const WARMUP: usize = 1;

/// Measured runs. The scan is seconds rather than microseconds, so the sample count is small
/// and the median is the number reported.
const SAMPLES: usize = 5;

fn main() -> ExitCode {
    let (reference, _) = reference_vault(KdfParams::TESTING).expect("testing parameters are valid");
    let (audit, _) = audit_vault(KdfParams::TESTING).expect("testing parameters are valid");

    println!("Watchtower local scan — S-07a. Median of {SAMPLES} after {WARMUP} warm-up.\n");

    let reference_ms = measure("reference vault", &reference);
    let audit_ms = measure("audit vault", &audit);

    // Per-password rather than per-vault, because the two vaults are 1 000 items and 21: the
    // only comparable figure is the cost of one password, and the ratio between them is the
    // caveat this benchmark exists to quantify.
    let reference_each = reference_ms / ITEMS as f64;
    let audit_each = audit_ms / 21.0;
    println!(
        "\n  per password: reference {reference_each:.3} ms, audit {audit_each:.3} ms — \
         a password zxcvbn's dictionaries match costs {:.1}x one they do not",
        audit_each / reference_each
    );
    println!(
        "\nS-07a: {reference_ms:.1} ms against a budget of {BUDGET_MS:.0} ms for 1 000 items, \
         with the caveat that this fixture is zxcvbn's cheap case."
    );

    if reference_ms > BUDGET_MS {
        eprintln!(
            "\nS-07a FAILED: {reference_ms:.1} ms > {BUDGET_MS:.0} ms. Either the scan got \
             slower or this machine is not the one the budget was set on — say which in \
             trustvault-state.md rather than raising the number."
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Scans `vault` [`SAMPLES`] times and prints what it found, returning the median in ms.
fn measure(label: &str, vault: &Vault) -> f64 {
    let mut samples = Vec::with_capacity(SAMPLES);
    let mut report = None;

    for run in 0..(WARMUP + SAMPLES) {
        let start = Instant::now();
        let scanned = scan(vault);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(scanned.findings.len());
        if run >= WARMUP {
            samples.push(elapsed);
        }
        report = Some(scanned);
    }

    samples.sort_by(f64::total_cmp);
    let median = samples[SAMPLES / 2];
    let report = report.expect("at least one run");
    println!(
        "  {label:<20} {median:>9.1} ms   {} items, {} password fields, {} distinct, {} findings",
        vault.items().count(),
        report.passwords,
        report.distinct,
        report.findings.len()
    );
    median
}
