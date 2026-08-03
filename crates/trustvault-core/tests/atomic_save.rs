//! R-05: killing the process mid-save leaves the previous vault intact and openable.
//!
//! A hundred injected kills, as the exit gate asks for. It cannot be tested in-process —
//! `panic!` unwinds and runs destructors, which is the opposite of what a power cut does — so
//! the harness re-executes this test binary as a child, lets it save in a loop, and sends it
//! a kill signal at a random moment. The parent then checks that the destination file is
//! whatever it was before or whatever it became, and never something in between.
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

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use trustvault_core::{ItemKind, Vault};

use common::{PARAMS, PASSWORD};

/// Environment variable naming the file the worker child should hammer.
const WORKER_PATH: &str = "TRUSTVAULT_KILL_TEST_PATH";
/// Suffix of the marker the worker writes once it is past start-up and into the save loop.
///
/// Without it the parent is racing the child's start-up — building the fixture vault and
/// running the KDF take longer than the kill delay, so every kill lands before the first
/// write and the test passes while proving nothing.
const READY_SUFFIX: &str = ".worker-ready";
/// Number of injected kills. The figure is R-05's, not a round number chosen for comfort.
const KILLS: usize = 100;

/// A vault large enough that a save takes tens of milliseconds, so a randomly timed kill lands
/// inside `save_to` rather than between two calls to it.
fn seed_vault() -> Vault {
    let (mut vault, _) = Vault::create("Atomic", PASSWORD, PARAMS).expect("create");
    for index in 0..8 {
        let id = vault.add_item(ItemKind::Note, format!("Item {index}"));
        let item = vault.item_mut(id).expect("just added");
        item.set_field("Password", format!("secret-value-number-{index}"), true);
        item.set_field("Note", "x".repeat(32 * 1024), false);
    }
    vault
}

/// The child process: save in a loop until something kills it.
///
/// Marked `#[ignore]` so it never runs as part of the suite; the parent invokes it by exact
/// name with `--ignored`.
#[test]
#[ignore = "spawned as a child process by kill_during_save_always_leaves_a_readable_vault"]
fn save_worker() {
    let Ok(path) = std::env::var(WORKER_PATH) else {
        return;
    };
    let mut vault = seed_vault();

    // One complete save before announcing readiness, so the parent knows start-up is over.
    if vault.save_to(&path).is_err() {
        return;
    }
    if std::fs::write(format!("{path}{READY_SUFFIX}"), b"ready").is_err() {
        return;
    }

    loop {
        // Change something each time, so a surviving file can be told apart from a stale one.
        vault.add_item(ItemKind::Note, "another");
        if vault.save_to(&path).is_err() {
            return;
        }
    }
}

/// Deletes any leftover temp files, returning how many there were.
///
/// A temp file surviving a kill is inert — §8 says so, and it is a complete sealed vault or a
/// fragment of one, neither readable without the password. Here it doubles as the signal that
/// the kill interrupted a write.
fn sweep_temp_files(directory: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return 0;
    };
    let mut swept = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(".tmp-") {
            let _ = std::fs::remove_file(entry.path());
            swept += 1;
        }
    }
    swept
}

/// Waits for the worker to reach its save loop. Returns false if it never does.
fn wait_for_worker(marker: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if marker.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    false
}

fn spawn_worker(path: &Path) -> std::io::Result<std::process::Child> {
    Command::new(std::env::current_exe()?)
        .args(["--exact", "save_worker", "--ignored", "--test-threads=1"])
        .env(WORKER_PATH, path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// A crude uniform draw over `range`, seeded from the clock.
///
/// Deliberately not the CSPRNG: the vault core exposes no random API to tests, and a test
/// fixture reaching for entropy through the crypto layer would be the wrong dependency.
fn jitter(range: std::ops::Range<u64>) -> Duration {
    let nanos = Instant::now().elapsed().subsec_nanos() as u64;
    let clock = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let span = range.end.saturating_sub(range.start).max(1);
    Duration::from_micros(range.start + (nanos ^ clock) % span)
}

#[test]
fn kill_during_save_always_leaves_a_readable_vault() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path: PathBuf = dir.path().join("hammered.tvault");

    // Establish a known-good vault first: the property under test is that the *previous*
    // contents survive, so there must be previous contents.
    let mut initial = seed_vault();
    initial.save_to(&path).expect("initial save");
    let baseline = Vault::open_file(&path, PASSWORD).expect("baseline opens");
    let baseline_items = baseline.items().count();
    drop(baseline);

    let marker = PathBuf::from(format!("{}{READY_SUFFIX}", path.display()));
    let mut survived = 0usize;
    let mut interrupted_mid_write = 0usize;

    for iteration in 0..KILLS {
        let _ = std::fs::remove_file(&marker);
        let mut child = match spawn_worker(&path) {
            Ok(child) => child,
            Err(error) => panic!("iteration {iteration}: could not spawn worker: {error}"),
        };

        let ready = wait_for_worker(&marker);
        // Then somewhere inside the save loop: long enough to be mid-write, short enough that
        // the kill is not always landing at the same point.
        std::thread::sleep(jitter(200..40_000));
        let _ = child.kill();
        let _ = child.wait();
        assert!(
            ready,
            "iteration {iteration}: the worker never reached its save loop"
        );

        // The invariant: whatever is at `path` is a complete, openable vault. Never a
        // truncated one, never a mixture of the old body and the new header.
        let vault = Vault::open_file(&path, PASSWORD).unwrap_or_else(|error| {
            panic!(
                "iteration {iteration}: the vault at {} is not readable after a kill: {error}",
                path.display()
            )
        });

        let items = vault.items().count();
        assert!(
            items >= baseline_items,
            "iteration {iteration}: the file went backwards — {items} items, baseline {baseline_items}"
        );
        survived += 1;
        interrupted_mid_write += sweep_temp_files(dir.path());
    }

    assert_eq!(survived, KILLS);
    println!(
        "{KILLS} kills survived; {interrupted_mid_write} left temp-file debris; the destination \
         was a complete readable vault every time"
    );

    // Deliberately *not* asserted: that some kill landed between `File::create` and `rename`.
    // That window is a memcpy into the page cache plus an fsync, which on the tmpfs a temp
    // directory usually lives on is microseconds out of a save measured in tens of
    // milliseconds — a randomly timed kill hits it perhaps once in a thousand, so asserting it
    // would be a flaky test rather than a stronger one. The rename property is proven
    // deterministically instead, by `the_destination_is_replaced_by_rename_never_written_in_place`
    // below.
}

#[test]
#[cfg(unix)]
fn the_destination_is_replaced_by_rename_never_written_in_place() {
    // §8 step 4, proven structurally rather than by racing a clock. If `save_to` truncated and
    // rewrote the destination, the inode would stay the same and a crash mid-write would leave
    // a half-vault at the real path. A changed inode means a different file was renamed over
    // it, which is the atomic operation the requirement is asking for.
    use std::os::unix::fs::MetadataExt;

    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("vault.tvault");

    let mut vault = seed_vault();
    vault.save_to(&path).expect("first save");
    let first = std::fs::metadata(&path).expect("metadata");

    vault.add_item(ItemKind::Card, "Another");
    vault.save_to(&path).expect("second save");
    let second = std::fs::metadata(&path).expect("metadata");

    assert_ne!(
        first.ino(),
        second.ino(),
        "the destination kept its inode, so it was written in place rather than renamed onto"
    );
    assert!(
        second.len() > first.len(),
        "the second save must have written more"
    );
    assert!(Vault::open_file(&path, PASSWORD).is_ok());
    assert_eq!(
        sweep_temp_files(dir.path()),
        0,
        "a clean save leaves no debris"
    );
}

#[test]
fn debris_from_an_earlier_crash_does_not_affect_the_destination() {
    // A temp file surviving a crash is inert (§8). It must not be picked up, overwritten in
    // place, or confused with the vault on the next save.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("vault.tvault");

    let mut vault = seed_vault();
    vault.save_to(&path).expect("save");

    // A fragment of a previous, interrupted write.
    let debris = dir.path().join(".vault.tvault.tmp-deadbeef");
    std::fs::write(&debris, b"TVLT truncated fragment").expect("write debris");

    vault.add_item(ItemKind::WiFi, "Home");
    vault.save_to(&path).expect("save over debris");

    assert!(debris.exists(), "an unrelated temp file must be left alone");
    let reopened = Vault::open_file(&path, PASSWORD).expect("open");
    assert_eq!(reopened.items().count(), 9);
}
