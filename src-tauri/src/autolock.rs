//! The idle timer and the sleep detector — R-09.
//!
//! R-09 wants three triggers: on demand, on timeout, and on OS sleep. The first two are
//! straightforward. The third is not, and the honest description of what this module does is
//! worth more than a comment claiming it hooks a suspend signal.
//!
//! **There is no portable OS sleep notification.** Linux has logind's `PrepareForSleep` over
//! D-Bus, macOS has `NSWorkspaceWillSleepNotification`, Windows has `WM_POWERBROADCAST` — three
//! platform integrations, two of which cannot be tested on this machine, for one requirement.
//! What this module uses instead is a **wall-clock jump detector**: the ticker wakes on a fixed
//! interval and compares how much wall-clock time actually passed. A machine that suspends for
//! twenty minutes wakes up having skipped twenty minutes of ticks, and that jump is the signal.
//!
//! What that costs, stated rather than hidden:
//!
//! * It fires on a **clock change** too — an NTP correction or a timezone-driven jump large
//!   enough to clear the threshold locks the vault. Locking spuriously is the safe direction of
//!   that error, and the user loses nothing but an unlock.
//! * It detects sleep on **wake**, not before it. A machine that suspends with the vault open
//!   has the master key in RAM while it sleeps, and this cannot change that — only a real
//!   pre-suspend hook could. That is a genuine gap and it belongs in the Phase 2 gate evidence,
//!   not in a comment claiming the requirement is fully met.

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::commands::vault::lock_now;
use crate::state::{AppState, LockReason, now_ms};

/// How often the idle timer wakes. Short enough that the lock is prompt, long enough to be
/// invisible in a power profile.
const TICK: Duration = Duration::from_secs(5);

/// Wall-clock overshoot past a tick that counts as "the machine slept".
///
/// Generous on purpose: a loaded machine can delay a 5-second timer by a second or two, and a
/// vault that locks itself because a build was running would be turned off by its owner within
/// a week.
const SLEEP_JUMP_MS: i64 = 60_000;

/// Starts the background ticker. Runs for the life of the process.
pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut last_tick = now_ms();
        loop {
            thread::sleep(TICK);
            let now = now_ms();
            let elapsed = now - last_tick;
            last_tick = now;

            let state = app.state::<AppState>();
            let Some((idle_ms, timeout_seconds, unlocked)) = state.with(|inner| {
                (
                    now - inner.last_activity,
                    inner.settings.auto_lock_seconds,
                    inner.vault.is_some(),
                )
            }) else {
                continue;
            };

            if let Some(reason) = decide(unlocked, elapsed, idle_ms, timeout_seconds) {
                lock_now(&app, &state, reason);
            }
        }
    });
}

/// Whether this tick locks the vault, and why — the whole of R-09's automatic half.
///
/// Split out of the ticker so it can be *measured* rather than asserted: the loop above needs
/// a Tauri runtime and a wall clock to reach, and a rule that can only be exercised by waiting
/// five minutes is a rule nobody exercises. Every branch here is a line of Phase 2 gate
/// evidence.
pub fn decide(
    unlocked: bool,
    elapsed_ms: i64,
    idle_ms: i64,
    timeout_seconds: u64,
) -> Option<LockReason> {
    if !unlocked {
        return None;
    }

    // Sleep first: a machine that just woke is also, by definition, idle, and the user is
    // better served by "your machine slept" than by "you were away".
    if elapsed_ms > SLEEP_JUMP_MS {
        return Some(LockReason::OsSleep);
    }

    // A timeout of zero means never, which is a setting a user is entitled to have and a
    // footgun the UI should say so about.
    if timeout_seconds > 0 && idle_ms > i64::try_from(timeout_seconds * 1000).unwrap_or(i64::MAX) {
        return Some(LockReason::Timeout);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One tick's worth of elapsed wall clock on a machine that did not sleep.
    const NORMAL: i64 = 5_000;

    #[test]
    fn a_locked_vault_is_never_locked_again() {
        assert_eq!(decide(false, 10_000_000, 10_000_000, 300), None);
    }

    #[test]
    fn the_idle_timeout_fires_just_past_the_configured_window() {
        // R-09, trigger two. 300 s is the default; the boundary is what is tested, because an
        // off-by-one here means either a vault that locks early or one that never does.
        assert_eq!(decide(true, NORMAL, 299_999, 300), None);
        assert_eq!(
            decide(true, NORMAL, 300_001, 300),
            Some(LockReason::Timeout)
        );
    }

    #[test]
    fn a_wall_clock_jump_reads_as_sleep_and_outranks_the_timeout() {
        // R-09, trigger three, and the ordering matters: a machine that just woke is also idle,
        // and "your machine slept" is the more useful of the two true statements.
        assert_eq!(
            decide(true, 20 * 60_000, 20 * 60_000, 300),
            Some(LockReason::OsSleep)
        );
        assert_eq!(
            decide(true, SLEEP_JUMP_MS + 1, 0, 0),
            Some(LockReason::OsSleep)
        );
    }

    #[test]
    fn a_loaded_machine_is_not_mistaken_for_a_sleeping_one() {
        // The reason SLEEP_JUMP_MS is generous: a 5-second timer delayed by a few seconds
        // under load must not lock the vault, or its owner turns the feature off within a week.
        assert_eq!(decide(true, 12_000, 1_000, 300), None);
    }

    #[test]
    fn a_timeout_of_zero_means_never() {
        assert_eq!(decide(true, NORMAL, 10_000_000, 0), None);
    }
}
