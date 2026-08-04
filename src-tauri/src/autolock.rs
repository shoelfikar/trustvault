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

            if !unlocked {
                continue;
            }

            // Sleep first: a machine that just woke is also, by definition, idle, and the user
            // is better served by "your machine slept" than by "you were away".
            if elapsed > SLEEP_JUMP_MS {
                lock_now(&app, &state, LockReason::OsSleep);
                continue;
            }

            // A timeout of zero means never, which is a setting a user is entitled to have and
            // a footgun the UI should say so about.
            if timeout_seconds > 0
                && idle_ms > i64::try_from(timeout_seconds * 1000).unwrap_or(i64::MAX)
            {
                lock_now(&app, &state, LockReason::Timeout);
            }
        }
    });
}
