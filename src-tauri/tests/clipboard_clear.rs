//! S-11 and R-14 with a stopwatch on them — Phase 2 gate line 3.
//!
//! The gate asks that the clipboard clear within the configured window + 200 ms. That is a
//! measurement, not an assertion about code, so this file measures the real
//! `commands::items::clear_after` rather than a re-implementation of it.
//!
//! **This test needs a clipboard, and therefore a display.** A headless CI runner has neither,
//! and the honest handling of that is to say so on stdout and pass, rather than to fail a build
//! for a machine's missing hardware or — worse — to be deleted for being flaky. The number in
//! the gate evidence comes from a run on a real desktop session; the line in the log says which
//! kind of run produced it.

// An integration test is its own crate with no `#[cfg(test)]` module, so clippy's
// `allow-unwrap-in-tests` does not reach it and the workspace's Tier-1 lints apply at full
// force. Lifted here, with the reason, exactly as tests/ipc_audit.rs does.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::{Duration, Instant};

use trustvault_lib::clipboard;
use trustvault_lib::commands::items::clear_after;

/// What the clipboard holds during the measurement. Not a real password, and it does not need
/// to be: the property under test is *when it stops being there*.
const PLANTED: &str = "trustvault-clipboard-stopwatch";

/// The window used for the measurement. One second rather than the default twelve, because the
/// thing being measured is the overshoot, and twelve seconds of sleep in a test suite buys
/// exactly the same number.
const WINDOW: u64 = 1;

/// S-11's tolerance: the clear may be late, by up to this much.
const TOLERANCE: Duration = Duration::from_millis(200);

#[test]
fn the_clipboard_clears_within_the_configured_window_plus_200ms() {
    if clipboard::set(PLANTED).is_err() {
        println!(
            "SKIPPED: no system clipboard on this machine (headless runner). \
             This measurement is gate evidence and must be produced on a desktop session."
        );
        return;
    }

    let started = Instant::now();
    clear_after(WINDOW);
    let elapsed = started.elapsed();

    let budget = Duration::from_secs(WINDOW) + TOLERANCE;
    println!(
        "clipboard cleared after {:?} (window {}s, budget {:?})",
        elapsed, WINDOW, budget
    );
    assert!(
        elapsed >= Duration::from_secs(WINDOW),
        "cleared early, at {elapsed:?} — a clipboard that clears before the window is a paste \
         the user was promised and did not get"
    );
    assert!(
        elapsed <= budget,
        "cleared late, at {elapsed:?}, over the {budget:?} budget — S-11"
    );

    // And it is actually gone, not merely unblocked: an assertion on the timer alone would pass
    // for a `clear_after` that slept and did nothing.
    let remaining = arboard::Clipboard::new()
        .and_then(|mut board| board.get_text())
        .unwrap_or_default();
    assert_ne!(
        remaining, PLANTED,
        "the window elapsed but the secret is still on the clipboard"
    );
}
