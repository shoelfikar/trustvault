//! S-03: how long does one Argon2id derivation take at the default parameters?
//!
//! Run with `cargo bench --bench kdf`. The number this prints is the one quoted in
//! `docs/vault-format.md` §3.2 and in the success-criteria table of
//! `trustvault-requirements.md` — re-run it and update both when the defaults change or when
//! the measurement moves to another machine.
//!
//! No criterion. The measurement is one wall-clock median and a statistics framework would be
//! more code, more dependencies, and more build time than the thing being measured.

use std::time::Instant;

use trustvault_core::{KdfParams, Vault};

/// Warm-up runs, discarded — the first allocation of 256 MiB is not representative.
const WARMUP: usize = 2;
/// Measured runs. The median is reported.
const SAMPLES: usize = 5;

fn main() {
    println!("Argon2id, one derivation, median of {SAMPLES} after {WARMUP} warm-ups\n");

    let cases = [
        ("default (256 MiB, t=3, p=1)", KdfParams::DEFAULT),
        ("testing (8 KiB, t=1, p=1)", KdfParams::TESTING),
    ];

    for (label, params) in cases {
        let mut samples = Vec::with_capacity(SAMPLES);
        for run in 0..(WARMUP + SAMPLES) {
            let start = Instant::now();
            // Creating a vault runs the KDF twice — once for the password wrap, once for the
            // recovery wrap — so the per-derivation cost is half the elapsed time.
            let created = Vault::create("bench", "correct horse battery staple", params);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0 / 2.0;
            assert!(created.is_ok(), "{label}: vault creation failed");
            if run >= WARMUP {
                samples.push(elapsed);
            }
        }
        samples.sort_by(f64::total_cmp);
        let median = samples.get(SAMPLES / 2).copied().unwrap_or(f64::NAN);
        println!("{label:<28} {median:>9.1} ms");
    }

    println!("\nS-03 requires >= 500 ms at the default parameters on the development machine.");

    // Calibration is the thing that actually ships: the default is only a starting point.
    let start = Instant::now();
    let calibrated = KdfParams::calibrate();
    println!(
        "\ncalibrate() -> m={} KiB, t={}, p={}  (took {:.1} s)",
        calibrated.m_cost,
        calibrated.t_cost,
        calibrated.p_cost,
        start.elapsed().as_secs_f64()
    );
}
