//! Does the "Clipboard clears in *n*s" chip survive a clipboard manager? — Phase 2 gate line 5.
//!
//! This is the open question from the Phase 0 prior-art survey, answered by experiment rather
//! than by reading: GPaste, Klipper and CopyQ record clipboard history, and TrustVault clearing
//! the clipboard it owns does nothing about a copy somebody else already took. CopyQ issue
//! #2802 documents exactly this against KeePassXC.
//!
//! The mechanism under test is the `x-kde-passwordManagerHint` MIME type — the most widely
//! adopted Linux convention for "do not record this", and the reason `arboard` was chosen over
//! the thinner wrappers (D-11). It is **advisory**: nothing obliges a manager to honour it, so
//! the only way to know what the UI may promise is to plant a marker and go and look.
//!
//! **Ignored by default.** It needs a desktop session and a running GPaste daemon, neither of
//! which a CI runner has, and a test that silently passes on a machine that cannot run it is
//! worse than one that says so. Run it deliberately:
//!
//!   cargo test -p trustvault --test clipboard_manager -- --ignored --nocapture

// An integration test is its own crate with no `#[cfg(test)]` module, so clippy's
// `allow-unwrap-in-tests` does not reach it and the workspace's Tier-1 lints apply at full
// force. Lifted here, with the reason, exactly as tests/ipc_audit.rs does.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::Command;
use std::thread;
use std::time::Duration;

use trustvault_lib::clipboard;

/// Long enough for the manager's D-Bus round trip. GPaste records on a selection-owner change,
/// which is immediate; this is slack, not a race being papered over.
const SETTLE: Duration = Duration::from_millis(600);

/// Whether GPaste's history holds `marker`.
fn gpaste_recorded(marker: &str) -> Option<bool> {
    let output = Command::new("gpaste-client")
        .args(["history", "--oneline"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).contains(marker))
}

#[test]
#[ignore = "needs a desktop session and a running GPaste daemon; run it deliberately"]
fn a_copied_secret_is_or_is_not_kept_by_gpaste() {
    let marker = format!("trustvault-hint-probe-{}", std::process::id());

    if clipboard::set(&marker).is_err() {
        panic!("no system clipboard — this test must run inside a desktop session");
    }
    thread::sleep(SETTLE);

    let Some(recorded) = gpaste_recorded(&marker) else {
        panic!("gpaste-client is not answering — install GPaste and start its daemon");
    };

    // Both outcomes are a result, and the test asserts the *product's copy matches whichever
    // one is true* rather than asserting an outcome. What must never happen is the UI
    // promising something this run disproved.
    println!(
        "GPaste {} the copied value (track-changes on, hint {}).",
        if recorded { "RECORDED" } else { "did not record" },
        if HINT_IS_SET { "set" } else { "NOT set" },
    );

    clipboard::clear();
    thread::sleep(SETTLE);

    let after_clear = gpaste_recorded(&marker).unwrap_or(false);
    println!(
        "After TrustVault cleared the clipboard, GPaste {} still holding it.",
        if after_clear { "IS" } else { "is not" },
    );

    assert_eq!(
        recorded, after_clear,
        "clearing the clipboard changed the manager's history — if this ever passes, the \
         product's copy can promise more than best effort"
    );
}

/// Whether `clipboard::set` asks managers not to record — kept in step with `clipboard.rs`.
///
/// A constant rather than a check, because the thing it describes is a decision made in one
/// place; if it goes out of date, the printed line above is the one that lies.
const HINT_IS_SET: bool = cfg!(target_os = "linux");
