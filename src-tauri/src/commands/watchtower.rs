//! Watchtower — `docs/ipc-contract.md` §6.9, R-23…R-26.
//!
//! **Two commands, and the split is the S-10 argument made structural** (D-76). The local half
//! scores and groups; the network half is the only thing in this application that opens a socket.
//! With one combined command, "zero packets while breach checking is off" would be a branch inside
//! a function somebody must keep taking; with two, it is a command nobody calls — and with the
//! setting off, [`watchtower_breach_check`] returns before a client is ever constructed.
//!
//! Three things cross back, and none of them is a secret:
//!
//! * The [`Report`] — findings naming item ids, scores and crack-time strings. The grouping key
//!   is a SHA-256 of a password and it never leaves the core; `shared_with` names the other
//!   **items**, which is what the user needs and none of what an offline attacker does.
//! * The [`BreachReport`] — ids, a corpus count, and the words `off` / `offline` / `http`. No
//!   prefix, no suffix, no hash: `requested` is a count rather than a list, because a list of
//!   prefixes is a description of the vault's passwords in the heap that cannot be wiped.
//! * Nothing else. Both write the per-item status cache into the vault as a side effect, and the
//!   item list reads that through `list_items` like it always has.
//!
//! Nothing here computes a hash or opens a socket itself. The SHA-1 is `trustvault-core`'s
//! ([`breach_queries`], D-78's argument: hashing every password means reading every password) and
//! the socket is [`crate::hibp`]'s. This file is the layer that decides **whether** either may
//! happen, which is exactly what §6.9 means by *the setting owns the egress, not the caller*.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tauri::{AppHandle, Emitter as _, Manager as _, State};

use serde::Serialize;
use trustvault_core::{BreachQuery, ItemId, Report, breach_queries, record_breaches};

use crate::commands::{save_open_vault, with_vault};
use crate::dto::{BreachHit, BreachReport, UncheckedField};
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::hibp::{RangeClient, RangeError};
use crate::state::{AppState, now_ms};

/// **Vault-class.** Scores every password and groups the reused ones — R-23, R-24.
///
/// Not sanctioned: a finding is not a secret. §6.9's budget of four stands, and if a Watchtower
/// command ever appears to want a plaintext password in the webview, the design is wrong rather
/// than the budget.
#[tauri::command(rename_all = "snake_case")]
pub fn watchtower_scan(state: State<'_, AppState>) -> IpcResult<Report> {
    watchtower_scan_inner(&state)
}

/// The body of [`watchtower_scan`], reachable without a Tauri runtime.
///
/// **Synchronous, and holding the state lock for the whole scan is deliberate.** §6.9 says the
/// results are discarded if the vault locked while the scan was running; here that cannot
/// happen, because `with_vault` holds the vault for the duration and an auto-lock cannot take it
/// away mid-flight. The breach check will not have that luxury — minutes rather than the 61 ms
/// S-07a measured — and this is the command whose shape makes the difference visible rather than
/// two commands that look alike and differ in what they must handle.
///
/// **It saves, and the save is the point of the status cache.** Writing statuses into memory
/// only would mean the item list drew fresh pips until the next relaunch and blank ones after
/// it, which looks exactly like a scan that never ran. Unlike `set_profile`, there is no
/// unchanged case to skip: `last_scan_at` moves on every scan, and "when did this last run" is
/// on screen.
pub fn watchtower_scan_inner(state: &AppState) -> IpcResult<Report> {
    with_vault(state, |vault, inner| {
        let report = trustvault_core::scan_and_record(vault);
        save_open_vault(vault, inner)?;
        Ok(report)
    })
}

/// How many range requests may be in flight at once — **D-85**.
///
/// A bound rather than a target. §6.9 measured 48 cold prefixes at 1.87 req/s serially and
/// 2.5 req/s at eight concurrent: eight times the sockets for **1.34×** the throughput, because
/// the rate is the service's rather than ours. So the number is not chosen to go fast — it is
/// chosen to be a polite burst against a free, unauthenticated API somebody else runs, while
/// still absorbing one stalled connection (a 30 s timeout on one worker leaves three working,
/// where serial would stop the whole check dead for those 30 s).
const WORKERS: usize = 4;

/// The one reason [`crate::hibp`] cannot produce — §6.9's `unchecked.reason` vocabulary.
///
/// It is here rather than there because a client that can report "the setting was off" is a
/// client that ran. This word is the command declining to call it at all.
const OFF: &str = "off";

/// `watchtower-progress` — §8. Two integers, and nothing else about the vault.
///
/// A progress event naming the item being checked would be a running commentary on the vault,
/// emitted on a timer, with no user action behind any of it.
#[derive(Debug, Clone, Copy, Serialize)]
struct Progress {
    done: usize,
    total: usize,
}

/// **Vault-class.** Asks Have I Been Pwned about every distinct password — R-25, R-26.
///
/// Not sanctioned: the response carries ids and counts, and no value from the vault.
///
/// `async` because this is minutes rather than milliseconds. Tauri runs a synchronous command on
/// the main thread, and a thousand blocking range requests there is a frozen window; the work goes
/// to the blocking pool and the body below is an ordinary synchronous function that
/// `tests/ipc_audit.rs` can drive without a runtime, like every other `_inner` in this module.
#[tauri::command(rename_all = "snake_case")]
pub async fn watchtower_breach_check(app: AppHandle) -> IpcResult<BreachReport> {
    let handle = tauri::async_runtime::spawn_blocking(move || {
        let emitter = app.clone();
        breach_check_inner(
            &app.state::<AppState>(),
            &RangeClient::default(),
            &move |done, total| {
                let _ = emitter.emit("watchtower-progress", Progress { done, total });
            },
        )
    });

    // A panicked worker is reported as an internal error rather than as a failed check: the
    // difference matters on screen, because "not checked" invites a retry and this does not.
    handle
        .await
        .unwrap_or_else(|_| Err(IpcError::new(ErrorKind::Internal)))
}

/// The body of [`watchtower_breach_check`], reachable without a Tauri runtime.
///
/// Both the client and the progress sink are injected, which is what lets the tests below assert
/// the two claims that matter most about this command **offline**: that a refused check contacts
/// nothing (D-84's shape — watch the place the bytes would have gone), and that a vault which
/// locks mid-check has its results dropped.
///
/// # The vault is held three times and never across the network
///
/// 1. **Open it, read the setting, build the queries, let go.** Holding the state mutex for
///    minutes would block every other command, including `lock`.
/// 2. **The network, holding nothing.** The queries own everything the requests need, and a
///    [`BreachQuery`] is a prefix, a suffix with no accessor, and a list of ids — so what the
///    worker threads carry is not readable as a password even by the code carrying it.
/// 3. **Re-open it to write, and only if it is the same vault.** §6.9: results are discarded if
///    the vault locked while the check was running. The generation is what makes "the same vault"
///    checkable — a lock and an unlock in between bumps it twice, and a stale write would put
///    verdicts about one vault's passwords into another's status cache.
///
/// **The second acquisition must not touch the idle clock**, which is why it does not go through
/// [`with_vault`]. A background task that resets the auto-lock timer keeps a vault unlocked for as
/// long as it runs, and this one runs for minutes — R-09 would become "locks when idle, unless
/// something is busy", which is not what it says.
pub fn breach_check_inner(
    state: &AppState,
    client: &RangeClient,
    progress: &(dyn Fn(usize, usize) + Sync),
) -> IpcResult<BreachReport> {
    // The user asked for this, so the first acquisition *does* touch the clock — it is a command
    // like any other. Everything after this point is the machine's work rather than the user's.
    let (enabled, generation, queries) = with_vault(state, |vault, inner| {
        Ok((
            inner.settings.breach_check_enabled,
            inner.generation,
            breach_queries(vault),
        ))
    })?;

    if !enabled {
        // R-26 and §6.9: a refusal, not an error. An error would be read by the UI as a failure
        // to be retried, and this is the product working exactly as the user configured it.
        // Nothing below this line runs — no client is constructed, no thread is spawned.
        return Ok(refused(&queries));
    }

    let outcomes = check_all(client, &queries, progress);

    let mut breached = Vec::new();
    let mut unchecked = Vec::new();
    let mut hits: Vec<ItemId> = Vec::new();
    // Every value answered for. A partial pass is the one thing that must not stamp the vault's
    // `last_breach_check_at` (D-86), because that stamp is all a later launch can see.
    let mut complete = true;

    for (query, outcome) in queries.iter().zip(outcomes) {
        match outcome {
            Ok(Some(count)) => {
                for &(item_id, field_id) in query.members() {
                    breached.push(BreachHit {
                        item_id,
                        field_id,
                        count,
                    });
                    hits.push(item_id);
                }
            }
            // Absent from the range is the only clean answer there is, and it is recorded by
            // *not* writing anything: `record_breaches` promotes nothing to strong.
            Ok(None) => {}
            Err(error) => {
                complete = false;
                for &(item_id, field_id) in query.members() {
                    unchecked.push(UncheckedField {
                        item_id,
                        field_id,
                        reason: error.reason(),
                    });
                }
            }
        }
    }

    let checked_at = record(state, generation, &hits, complete)?;

    Ok(BreachReport {
        checked_at,
        // Attempted, not succeeded. A failed request still put bytes on the wire — or tried to —
        // and S-10's claim is about the difference between this number being zero and not.
        requested: queries.len(),
        breached,
        unchecked,
    })
}

/// The report a check that never ran answers with — §6.9, R-26.
///
/// **Every password field is named**, and they are named out of the same queries the enabled path
/// would have sent. One enumeration rather than two: a second "which fields are checkable" would
/// let the off case report about a different set of fields than the on case checks, and the
/// difference would be invisible on screen.
fn refused(queries: &[BreachQuery]) -> BreachReport {
    BreachReport {
        checked_at: now_ms(),
        requested: 0,
        breached: Vec::new(),
        unchecked: queries
            .iter()
            .flat_map(|query| query.members())
            .map(|&(item_id, field_id)| UncheckedField {
                item_id,
                field_id,
                reason: OFF,
            })
            .collect(),
    }
}

/// Runs every query through the client, [`WORKERS`] at a time, in the queries' own order.
///
/// Returns one outcome per query, aligned by index: `Ok(Some(count))` is a hit, `Ok(None)` is a
/// value the corpus does not hold, and `Err` is a value nobody can say anything about.
///
/// A worker that panics leaves its slots empty, and an empty slot is read as
/// [`RangeError::Offline`] — *not checked*. That is the fail-closed direction, and it is the
/// reason the outcomes are collected into a fixed-size table rather than pushed onto a shared
/// list: a missing answer must be visible as a missing answer, not as one value's result silently
/// standing in for another's.
fn check_all(
    client: &RangeClient,
    queries: &[BreachQuery],
    progress: &(dyn Fn(usize, usize) + Sync),
) -> Vec<Result<Option<u64>, RangeError>> {
    let total = queries.len();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);

    let collected: Vec<(usize, Result<Option<u64>, RangeError>)> = thread::scope(|scope| {
        let workers: Vec<_> = (0..WORKERS.min(total))
            .map(|_| {
                scope.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(query) = queries.get(index) else {
                            return mine;
                        };
                        // The matching happens here, against the query, and the count is all
                        // that comes back — `Range::count_for` asks `BreachQuery::matches`,
                        // because this layer has no way to read the suffix it is matching.
                        let outcome = client
                            .range(query.prefix())
                            .map(|range| range.count_for(query));
                        mine.push((index, outcome));
                        progress(done.fetch_add(1, Ordering::Relaxed) + 1, total);
                    }
                })
            })
            .collect();

        workers
            .into_iter()
            .filter_map(|worker| worker.join().ok())
            .flatten()
            .collect()
    });

    let mut outcomes: Vec<Result<Option<u64>, RangeError>> =
        (0..total).map(|_| Err(RangeError::Offline)).collect();
    for (index, outcome) in collected {
        if let Some(slot) = outcomes.get_mut(index) {
            *slot = outcome;
        }
    }
    outcomes
}

/// Writes the hits into the still-open vault, or refuses because it is no longer the same one.
///
/// Deliberately **not** [`with_vault`]: that function touches the idle clock, and see the note on
/// [`breach_check_inner`] for why a background task must not. The generation check is what
/// `with_vault` cannot do here — a vault that locked and was unlocked again mid-check is open,
/// and writing into it would be recording one vault's verdicts in another vault's cache.
fn record(state: &AppState, generation: u64, hits: &[ItemId], complete: bool) -> IpcResult<i64> {
    state
        .with(|inner| {
            if inner.generation != generation {
                return Err(IpcError::locked());
            }
            let Some(mut vault) = inner.vault.take() else {
                return Err(IpcError::locked());
            };
            let checked_at = record_breaches(&mut vault, hits, complete);
            // The save can fail, and the vault goes back either way: a failed write must not
            // also lose the open vault. Same ordering as `watchtower_scan`.
            let saved = save_open_vault(&mut vault, inner);
            inner.vault = Some(vault);
            saved?;
            Ok(checked_at)
        })
        .unwrap_or_else(|| Err(IpcError::locked()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use trustvault_core::{Field, FieldKind, ItemKind, ItemStatus, KdfParams, Vault, Verdict};

    /// An unlocked state over a vault with one reused password and one strong one.
    ///
    /// No path that can be written: these tests are about what the command computes and refuses,
    /// and the save is exercised against a real file in `tests/ipc_session.rs`.
    fn unlocked() -> AppState {
        let (mut vault, _) =
            Vault::create("Personal", "master pw", KdfParams::TESTING).expect("valid params");
        for title in ["Forum", "Wiki"] {
            let id = vault.add_item(ItemKind::Login, title);
            let item = vault.item_mut(id).expect("just added");
            item.push_field(
                Field::new("Password", "password", true).with_kind(FieldKind::Password),
            );
        }
        let id = vault.add_item(ItemKind::Login, "Stripe");
        let item = vault.item_mut(id).expect("just added");
        item.push_field(
            Field::new("Password", "qX7#vn2Lp!4dRt", true).with_kind(FieldKind::Password),
        );

        let state = AppState::default();
        state.with(|inner| {
            inner.vault = Some(vault);
            inner.path = Some(PathBuf::from("/nonexistent/personal.tvault"));
        });
        state
    }

    /// The scan reports what R-23 and R-24 ask for, through the command rather than the core.
    #[test]
    fn the_command_returns_the_findings_the_core_produced() {
        let state = unlocked();
        // The save fails — the path is unwritable on purpose — so the report is read off the
        // vault instead. What this asserts is the wiring: the command reaches the core's scan
        // with the open vault, which is the half a unit test of `scan` cannot cover.
        let _ = watchtower_scan_inner(&state);
        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            let report = trustvault_core::scan(vault);
            assert_eq!(report.passwords, 3);
            assert_eq!(report.distinct, 2, "one request per distinct value — S-07b");
            assert!(
                report
                    .findings
                    .iter()
                    .any(|finding| finding.verdict == Verdict::Reused)
            );
        });
    }

    /// A failed save does not leave the caller believing the cache was written.
    ///
    /// The order matters and this is what pins it: statuses are written into the in-memory vault
    /// first and the save can still fail, so the command must return the error rather than the
    /// report it computed. A report returned beside a silently unwritten cache is the state where
    /// the Watchtower screen and the item list disagree after a relaunch.
    #[test]
    fn a_scan_that_cannot_be_saved_fails_rather_than_reporting_success() {
        let state = unlocked();
        assert!(
            watchtower_scan_inner(&state).is_err(),
            "an unwritable vault path is a failed scan, not a silent one"
        );
        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            assert!(
                vault.items().any(|item| item.status == ItemStatus::Reused),
                "the in-memory cache is written before the save is attempted"
            );
        });
    }

    /// Vault-class: locked means locked, and the check is the contract's §9 check 6.
    #[test]
    fn a_locked_vault_refuses_to_be_scanned() {
        let state = AppState::default();
        let refused = watchtower_scan_inner(&state);
        assert!(refused.is_err(), "there is no vault to scan");
    }

    /* ---- The breach half — R-25, R-26, S-10 -------------------------------------------- */

    /// The same trimmed real response `hibp.rs` is tested against.
    const FIXTURE: &str = include_str!("../../tests/fixtures/hibp-range-5BAA6.txt");

    /// How many times HIBP says `password` appears in the corpus, per the fixture.
    const PASSWORD_COUNT: u64 = 52_372_427;

    /// A writable vault path of this test's own, since a breach check that finds something saves.
    fn writable(state: &AppState, name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "trustvault-breach-{name}-{}.tvault",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        state.with(|inner| inner.path = Some(path.clone()));
        path
    }

    fn enable(state: &AppState) {
        state.with(|inner| inner.settings.breach_check_enabled = true);
    }

    /// Progress that records what it was told, so the event's arithmetic is checkable offline.
    fn counting() -> (std::sync::Mutex<Vec<(usize, usize)>>,) {
        (std::sync::Mutex::new(Vec::new()),)
    }

    /// **S-10, as far as a test can take it: with the setting off, nothing is contacted.**
    ///
    /// D-84's shape rather than an assertion about a boolean. A test reading
    /// `settings.breach_check_enabled` back would pass with the check deleted; this one starts a
    /// listener at the only address the client could reach and asserts it never hears from us —
    /// so the refusal is measured where the bytes would have gone. The packet capture the gate
    /// asks for is the same claim one layer down, and this is the half that runs on every push.
    #[test]
    fn with_the_setting_off_no_request_is_made_and_nothing_is_contacted() {
        let (endpoint, listening) =
            crate::hibp::testing::serving(&crate::hibp::testing::ok_response(FIXTURE));
        let state = unlocked();
        // The path is deliberately unwritable: a refusal must not save the vault either, and if
        // it tried, this would come back as an error instead of a report.
        let report = breach_check_inner(&state, &RangeClient::new(&endpoint), &|_, _| {})
            .expect("a refusal is not an error — §6.9");

        assert_eq!(report.requested, 0, "S-10: no range request was made");
        assert!(report.breached.is_empty());
        assert_eq!(
            report.unchecked.len(),
            3,
            "every password field is named, so nothing reads as checked"
        );
        assert!(report.unchecked.iter().all(|field| field.reason == "off"));
        assert!(
            listening.try_recv().is_err(),
            "a request reached the service with breach checking off"
        );
        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            assert!(
                vault.body().last_breach_check_at.is_none(),
                "a check that did not run must not stamp the vault"
            );
        });
    }

    /// A hit is reported, cached as `breached`, and a complete pass stamps the vault — D-86.
    #[test]
    fn a_hit_is_reported_for_every_member_and_a_complete_pass_stamps_the_vault() {
        let (endpoint, _listening) =
            crate::hibp::testing::serving(&crate::hibp::testing::ok_response(FIXTURE));
        let state = unlocked();
        let path = writable(&state, "hit");
        enable(&state);
        watchtower_scan_inner(&state).expect("the scan saves");

        let (seen,) = counting();
        let report = breach_check_inner(&state, &RangeClient::new(&endpoint), &|done, total| {
            if let Ok(mut log) = seen.lock() {
                log.push((done, total));
            }
        })
        .expect("the vault is open and the fixture is served");

        assert_eq!(
            report.requested, 2,
            "one request per distinct value — S-07b"
        );
        assert_eq!(
            report.breached.len(),
            2,
            "both items sharing the value are reported from the one answer"
        );
        assert!(
            report
                .breached
                .iter()
                .all(|hit| hit.count == PASSWORD_COUNT)
        );
        assert!(report.unchecked.is_empty());
        assert_eq!(
            seen.lock().map(|log| log.last().copied()).ok().flatten(),
            Some((2, 2)),
            "progress ends where the work does — §8"
        );

        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            let breached = vault
                .items()
                .filter(|item| item.status == ItemStatus::Breached)
                .count();
            assert_eq!(
                breached, 2,
                "the hit is cached where the item list reads it"
            );
            assert_eq!(
                vault.body().last_breach_check_at,
                Some(report.checked_at),
                "a complete pass says when it ran"
            );
        });
        let _ = std::fs::remove_file(&path);
    }

    /// A failed request reports **not checked**, and nothing about it reads as safe — R-25.
    ///
    /// A 400 rather than a 429 or a 500 on purpose: those are retried, and the backoff sleeps for
    /// real. The retry policy is tested in `hibp.rs`, where the sleep is injected; what this test
    /// is about is the shape of the report and what it leaves in the vault.
    #[test]
    fn a_failed_request_is_reported_unchecked_and_leaves_the_statuses_alone() {
        let (endpoint, _listening) =
            crate::hibp::testing::serving("HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
        let state = unlocked();
        let path = writable(&state, "failed");
        enable(&state);
        watchtower_scan_inner(&state).expect("the scan saves");

        let report = breach_check_inner(&state, &RangeClient::new(&endpoint), &|_, _| {})
            .expect("a refused service is a report, not an error");

        assert!(report.breached.is_empty());
        assert_eq!(
            report.unchecked.len(),
            3,
            "every field behind a failed request is named"
        );
        assert!(report.unchecked.iter().all(|field| field.reason == "http"));

        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("still open");
            assert!(
                vault
                    .items()
                    .all(|item| item.status != ItemStatus::Breached),
                "a check that failed proves nothing in either direction"
            );
            assert!(
                vault.body().last_breach_check_at.is_none(),
                "an incomplete pass must not read afterwards as a check that ran — D-86"
            );
        });
        let _ = std::fs::remove_file(&path);
    }

    /// A vault that locks while the check runs has the results dropped — §6.9.
    ///
    /// The lock is pulled from inside the progress callback, which is the one hook in this
    /// command that runs *during* the network phase. That makes the race deterministic instead of
    /// hoped for: by the time the outcomes are being recorded, the vault is gone.
    #[test]
    fn a_vault_that_locks_mid_check_keeps_nothing_the_check_computed() {
        let (endpoint, _listening) =
            crate::hibp::testing::serving(&crate::hibp::testing::ok_response(FIXTURE));
        let state = unlocked();
        let path = writable(&state, "locked");
        enable(&state);

        let refused = breach_check_inner(&state, &RangeClient::new(&endpoint), &|_, _| {
            state.lock();
        });

        assert!(
            refused.is_err(),
            "results computed against a vault that is no longer open must be dropped"
        );
        state.with(|inner| {
            assert!(inner.vault.is_none(), "the lock stands");
        });
        assert!(
            !path.exists(),
            "a dropped result must not have been written to the file either"
        );
    }

    /// A vault that locked **and was opened again** mid-check is not the vault that was checked.
    ///
    /// The sharper half of the rule above, and the one the "is anything open" test cannot reach:
    /// by the time the results are recorded there *is* an open vault, and writing into it would
    /// put one vault's verdicts into another's status cache. Only the generation says they are
    /// different vaults — verified by deleting the check, which leaves this test failing and the
    /// one above passing.
    #[test]
    fn a_vault_reopened_mid_check_does_not_receive_the_previous_vaults_verdicts() {
        let (endpoint, _listening) =
            crate::hibp::testing::serving(&crate::hibp::testing::ok_response(FIXTURE));
        let state = unlocked();
        let path = writable(&state, "reopened");
        enable(&state);

        let refused = breach_check_inner(&state, &RangeClient::new(&endpoint), &|_, _| {
            // Locked, then a *different* vault opened in its place — two generation bumps, and
            // an open vault waiting at the end of the check.
            if state.with(|inner| inner.vault.is_some()) == Some(true) {
                state.lock();
                let (other, _) =
                    Vault::create("Work", "master pw", KdfParams::TESTING).expect("valid params");
                state.with(|inner| inner.opened(other, path.clone()));
            }
        });

        assert!(refused.is_err(), "the vault that was checked is gone");
        state.with(|inner| {
            let vault = inner.vault.as_ref().expect("the second vault is open");
            assert_eq!(vault.name(), "Work");
            assert!(
                vault.body().last_breach_check_at.is_none(),
                "a vault nobody checked must not be stamped as checked"
            );
        });
        let _ = std::fs::remove_file(&path);
    }

    /// Vault-class: locked means locked, before any client is built — §9 check 6.
    #[test]
    fn a_locked_vault_refuses_to_be_breach_checked() {
        let state = AppState::default();
        // Port 1 is not listening and never will be; if this command reached the network at all,
        // the test would be slow rather than merely failing.
        let refused =
            breach_check_inner(&state, &RangeClient::new("http://127.0.0.1:1"), &|_, _| {});
        assert!(refused.is_err(), "there is no vault to check");
    }
}
