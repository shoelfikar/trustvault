//! S-04, the half that is measurable here: how long does matching a query against the
//! reference vault take?
//!
//! Run with `cargo bench --bench search`. **This is not the whole criterion.** S-04 is
//! *keystroke to filtered results rendered* — this measures the middle of that path, the
//! `search_items` command's body against 1 000 items, and leaves out the IPC hop and the
//! webview's render. What it is for is the question a benchmark can answer and a manual
//! measurement cannot: how much of the 50 ms budget the matching itself spends, and whether a
//! later change to the scoring quietly ate it.
//!
//! The end-to-end number comes from the running app, through the `performance.measure` marks in
//! `src/lib/shell/CommandPalette.svelte`. Nothing in CI runs the app, so that half is a manual
//! measurement like the clipboard ones — recorded as such rather than implied to be automated.
//!
//! No criterion, for the reason `kdf.rs` gives: the answer is a percentile over a few thousand
//! samples, which is a sort and an index.

// A benchmark is its own crate, so the library's Tier-1 lints apply here at full force. Lifted
// with the reason, exactly as the integration tests do it: in a measurement harness a panic is
// the failure report.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::time::Instant;

use trustvault_core::KdfParams;
use trustvault_core::benchfixture::{ITEMS, reference_vault};

/// How many results the palette asks for. It draws six item rows; the extra is headroom so the
/// benchmark is not measuring a limit smaller than the surface uses.
const LIMIT: usize = 10;

/// Queries typed one character at a time, which is what S-04 measures — the keystroke, not the
/// finished word. Chosen to cover the shapes with different costs: a prefix that matches many
/// items, a scattered subsequence, a tag, a username, and a query that matches nothing.
const QUERIES: [&str; 6] = ["github", "dgo", "infrastructure", "user42", "zzzq", "s"];

/// Discarded runs, so the first allocation of the result vector is not in the numbers.
const WARMUP: usize = 200;

fn main() {
    let (vault, _) = reference_vault(KdfParams::TESTING).expect("testing parameters are valid");
    assert_eq!(vault.items().count(), ITEMS);

    println!("Palette matching, {ITEMS} items, limit {LIMIT} — S-04's host half\n");

    let mut all = Vec::new();

    for query in QUERIES {
        let mut samples = Vec::new();
        // Every prefix of the query, because a user types `g`, `gi`, `git`… and the short
        // prefixes are the expensive ones: they match nearly everything.
        for length in 1..=query.chars().count() {
            let prefix: String = query.chars().take(length).collect();
            for run in 0..(WARMUP + 1_000) {
                let start = Instant::now();
                let hits = vault.search(&prefix, LIMIT);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                std::hint::black_box(hits.len());
                if run >= WARMUP {
                    samples.push(elapsed);
                }
            }
        }

        let (median, p95, worst) = percentiles(&mut samples);
        println!(
            "  {query:<16} n={:<6} median {median:>7.3} ms   p95 {p95:>7.3} ms   max {worst:>7.3} ms",
            samples.len()
        );
        all.extend(samples);
    }

    let (median, p95, worst) = percentiles(&mut all);
    println!(
        "\n  {:<16} n={:<6} median {median:>7.3} ms   p95 {p95:>7.3} ms   max {worst:>7.3} ms",
        "all queries",
        all.len()
    );
    println!(
        "\nS-04's budget is 50 ms p95 for the whole path — keystroke, IPC, match, render. \
         The share above is what matching spends; the rest is measured in the app."
    );
}

/// Median, p95 and maximum of a sample set, in the order this benchmark prints them.
fn percentiles(samples: &mut [f64]) -> (f64, f64, f64) {
    samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a duration"));
    let at = |fraction: f64| {
        let index = ((samples.len() as f64 * fraction) as usize).min(samples.len() - 1);
        samples[index]
    };
    (at(0.5), at(0.95), at(1.0))
}
