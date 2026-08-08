//! The instrumented session — Phase 2 gate line 4 and part of Phase 3's first gate line,
//! `docs/ipc-contract.md` §9 check 5.
//!
//! §9 listed this check as **not automated**, for an honest reason: it needs a shell to drive,
//! and when the contract was written there was none. There is one now, so the check is here.
//!
//! What it does is what the gate asks for: script a whole-shell session against a real vault
//! file on disk, record **every value that crosses the boundary** into a log, and then read
//! the log rather than the code. The distinction matters — `ipc_audit.rs` asserts the shape of
//! each response in isolation, which cannot catch a secret that leaks on the third call
//! because of what the first two did. This one replays a session and searches the transcript.
//!
//! Two limits, named rather than left for a reader to discover:
//!
//! * It drives the **command bodies**, not a live webview. It therefore proves what the host
//!   sends, not what the frontend asks for; the second half is covered by `src/lib/ipc.ts`
//!   being the only file that calls `invoke`, which CI greps for.
//! * It creates its item through `add_item`, quits, relaunches and reads it back — which is
//!   the Phase 3 exit gate's first line **minus the human at the keyboard**. Phase 2 had to
//!   seed through the core's API because it shipped no mutation command (D-38); that is no
//!   longer true, and what remains un-automated is the UI half: that the dialog submits what
//!   the user typed and the detail pane renders what came back. The gate still asks a person
//!   to do it once on a real app.

// An integration test is its own crate with no `#[cfg(test)]` module, so clippy's
// `allow-unwrap-in-tests` does not reach it and the workspace's Tier-1 lints apply at full
// force. Lifted here, with the reason, exactly as tests/ipc_audit.rs does.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use trustvault_core::{FieldKind, ItemKind, KdfParams};
use trustvault_lib::commands::{items, vault as vault_cmd};
use trustvault_lib::dto::{EditField, FieldSummary, KdfSummary, NewField};
use trustvault_lib::state::AppState;

/// The master password. It crosses **inbound** and must never come back out.
const MASTER: &str = "waltz-jumbled-fox-quiz-97";
/// The stored secret. It may cross outbound exactly once per explicit reveal, and never
/// otherwise.
const SECRET: &str = "correct-horse-battery-staple";
/// The replacement typed into the edit form. It crosses **inbound only** and is never
/// revealed, so it must not appear anywhere in the transcript at all.
const EDITED_SECRET: &str = "staple-battery-horse-correct";
/// A field the user declared is not secret, which is allowed to cross freely.
const USERNAME: &str = "octocat";

/// One line of the transcript: what was called, and everything it sent back.
struct Crossing {
    /// The command name as the webview would invoke it.
    command: &'static str,
    /// The serialized response — `Ok` and `Err` alike, because an error message is a payload
    /// too and composing one out of the failing value is a classic way to leak it.
    payload: String,
}

/// The transcript, plus the assertions that read it.
#[derive(Default)]
struct SessionLog {
    crossings: Vec<Crossing>,
}

impl SessionLog {
    /// Records one crossing, serializing whatever the command returned.
    fn record<T: serde::Serialize, E: serde::Serialize>(
        &mut self,
        command: &'static str,
        result: &Result<T, E>,
    ) {
        let payload = match result {
            Ok(value) => serde_json::to_string(value).unwrap(),
            Err(error) => serde_json::to_string(error).unwrap(),
        };
        self.crossings.push(Crossing { command, payload });
    }

    /// Records a command that cannot fail.
    fn record_infallible<T: serde::Serialize>(&mut self, command: &'static str, value: &T) {
        self.crossings.push(Crossing {
            command,
            payload: serde_json::to_string(value).unwrap(),
        });
    }

    /// The crossings that carry `needle`, by command name.
    fn carrying(&self, needle: &str) -> Vec<&'static str> {
        self.crossings
            .iter()
            .filter(|crossing| crossing.payload.contains(needle))
            .map(|crossing| crossing.command)
            .collect()
    }

    /// Writes the transcript where a human can read it as gate evidence.
    ///
    /// Secrets and all: this file is the artifact the gate line asks for, it is written under
    /// `target/`, and the vault it describes is a throwaway created by this test.
    fn write_to(&self, path: &Path) {
        let mut text = String::from("# TrustVault IPC session transcript\n#\n");
        text.push_str("# Every value crossing the boundary, in order. Gate G-B′ line 4.\n\n");
        for crossing in &self.crossings {
            text.push_str(crossing.command);
            text.push_str(" -> ");
            text.push_str(&crossing.payload);
            text.push('\n');
        }
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, text);
    }
}

/// A throwaway vault path, unique per run so a leftover file cannot make a later run pass.
fn scratch_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "trustvault-session-{}-{unique}.tvault",
        std::process::id()
    ))
}

/// The login's two fields, as the New-item dialog submits them.
///
/// Built fresh per call because `NewField` owns its plaintext and is consumed on the way in.
fn login_fields() -> Vec<NewField> {
    vec![
        NewField {
            label: "Username".into(),
            kind: FieldKind::Username,
            value: USERNAME.into(),
            secret: false,
            custom: false,
        },
        NewField {
            label: "Password".into(),
            kind: FieldKind::Password,
            value: SECRET.into(),
            secret: true,
            custom: false,
        },
    ]
}

/// The edit an untouched field submits: everything as the detail pane received it, and
/// `value: None` — because the pane never received the value.
fn untouched(field: &FieldSummary) -> EditField {
    EditField {
        id: Some(field.id),
        label: field.label.clone(),
        kind: field.kind,
        value: None,
        secret: field.secret,
        custom: field.custom,
    }
}

/// The whole session, start to finish, read back as a transcript.
///
/// The script is the shell's real path: nothing chosen, a vault created, the app quit and
/// relaunched (a fresh `AppState` over the same file), unlocked, listed, opened, one field
/// revealed, one copied, then locked. Every step's response goes into the log.
#[test]
fn a_whole_session_leaks_nothing_outside_the_sanctioned_path() {
    let path = scratch_path();
    let mut log = SessionLog::default();

    // ---- Launch: nothing is open ---------------------------------------------------------
    let state = AppState::default();
    log.record_infallible("vault_status", &state.status());

    // ---- Onboarding: the password is scored, then the vault is created -------------------
    // TESTING parameters, not calibrated ones: this test is about what crosses the boundary,
    // and a 511 ms KDF per unlock would make it the slowest test in the workspace.
    let strength = trustvault_lib::commands::strength::score(MASTER, &["Session Vault".into()]);
    log.record_infallible("score_password", &strength);

    let created = vault_cmd::create_vault_inner(
        &state,
        "Session Vault".into(),
        path.display().to_string(),
        MASTER.into(),
        KdfSummary {
            m_cost: KdfParams::TESTING.m_cost,
            t_cost: KdfParams::TESTING.t_cost,
            p_cost: KdfParams::TESTING.p_cost,
        },
    );
    log.record("create_vault", &created);
    let recovery_code = created.expect("a writable path").recovery_code;

    // D-69: `create_vault` stops at memory, so nothing is on disk yet and no vault is open. The
    // session asserts both, because the whole point of the split is a window closed here leaving
    // no half-made vault behind — and an assertion is the only thing that keeps it true.
    assert!(
        !path.exists(),
        "create_vault must not write the file; the acknowledgement on step 3 does"
    );
    assert!(
        matches!(
            state.status().state,
            trustvault_lib::dto::VaultState::NoVault
        ),
        "a pending vault is neither open nor locked — it must not reach VaultState"
    );

    let committed = vault_cmd::commit_vault_inner(&state);
    log.record_infallible("commit_vault", &committed.is_ok());
    committed.expect("the pending vault is written on acknowledgement");
    assert!(path.exists(), "commit_vault writes the file");

    // ---- The item is created through the command, not seeded through the core --------------
    // Phase 2 had to seed it through `trustvault_core` because it shipped no mutation command
    // (D-38); `add_item` lands in Phase 3, so the first line of this phase's exit gate — an
    // item created, the app quit, relaunched, and the item read back — is scripted here rather
    // than only demonstrated by hand. `add_item` saves before returning, which is what makes
    // the next step (a relaunch) able to find it at all.
    let added = items::add_item_inner(
        &state,
        ItemKind::Login,
        "GitHub".into(),
        vec!["Work/Clients".into(), "dev".into()],
        login_fields(),
    );
    log.record("add_item", &added);
    let created_id = added.expect("a vault is open").item_id;

    // ---- Quit and relaunch ---------------------------------------------------------------
    // A fresh AppState over the same file is what a relaunch is: the host keeps nothing in
    // memory, and the only way back in is a credential. What it *does* keep is the remembered
    // path, out of the settings file, which is the whole of D-40 — `restore_last_vault` is the
    // same call `lib.rs`'s setup makes, driven here without a Tauri runtime.
    let remembered = state
        .with(|inner| inner.settings.last_vault_path.clone())
        .flatten();
    drop(state);

    let state = AppState::default();
    state.with(|inner| inner.settings.last_vault_path = remembered);
    trustvault_lib::commands::settings::restore_last_vault(&state);

    let relaunched = state.status();
    log.record_infallible("vault_status", &relaunched);
    assert_eq!(
        relaunched.state,
        trustvault_lib::dto::VaultState::Locked,
        "a relaunch lands on the lock screen; `no_vault` here means onboarding, and a user \
         with no way back into the vault they just made"
    );
    assert!(
        relaunched.item_count.is_none(),
        "a locked vault cannot know its item count"
    );

    // ---- A wrong password first, because that is the path a user actually takes ----------
    let refused = vault_cmd::unlock_inner(&state, path.display().to_string(), "not it".into());
    log.record("unlock", &refused);
    assert!(refused.is_err(), "a wrong password does not open a vault");

    let opened = vault_cmd::unlock_inner(&state, path.display().to_string(), MASTER.into());
    log.record("unlock", &opened);
    opened.expect("the right password opens it");
    log.record_infallible("vault_status", &state.status());

    // ---- The shell: list, open, reveal, copy ---------------------------------------------
    let listed = items::list_items_inner(&state);
    log.record("list_items", &listed);
    let summaries = listed.expect("unlocked");
    let item_id = summaries.first().expect("the created item").id;
    assert_eq!(
        item_id, created_id,
        "the item read back after a relaunch is the one `add_item` created — the first line \
         of the Phase 3 gate, minus the human at the keyboard"
    );
    assert_eq!(
        summaries[0].tags,
        vec!["Work/Clients".to_owned(), "dev".to_owned()],
        "a folder-shaped tag survives verbatim — D-43"
    );

    let detail = items::get_item_inner(&state, item_id);
    log.record("get_item", &detail);
    let fields = detail.expect("the item exists").fields;
    let secret_field = fields
        .iter()
        .find(|field| field.secret)
        .expect("the password submitted through add_item");
    assert_eq!(
        fields
            .iter()
            .find(|field| !field.secret)
            .and_then(|field| field.value.as_deref()),
        Some(USERNAME),
        "the public field comes back as its value, the secret one as a mask"
    );

    // The one explicit user action that may produce a secret.
    let revealed = items::reveal_field_inner(&state, item_id, secret_field.id).map(|(r, _)| r);
    log.record("reveal_field", &revealed);
    assert_eq!(
        revealed.expect("revealable").value,
        SECRET,
        "the sanctioned path does return it — that is what makes the rest meaningful"
    );

    // copy_field either writes to a real clipboard or fails because the runner has none.
    // Both outcomes are recorded and both are asserted against: an error payload is a payload,
    // and "the machine had no clipboard" must not become a hole in the transcript.
    let copied = items::copy_field_inner(&state, item_id, secret_field.id).map(|(c, _, _)| c);
    log.record("copy_field", &copied);

    // ---- Edit: rename, replace one secret, leave the other field alone --------------------
    // This is the shape of every real edit and the one that is silent when it goes wrong. The
    // username carries `value: None` because the detail pane never received a value it could
    // send back; if that meant "set it to nothing", the rename below would destroy it.
    let public_field = fields
        .iter()
        .find(|field| !field.secret)
        .expect("the username");
    let edited = items::update_item_inner(
        &state,
        item_id,
        "GitHub (work)".into(),
        vec!["dev".into()],
        true,
        vec![
            untouched(public_field),
            EditField {
                id: Some(secret_field.id),
                label: "Password".into(),
                kind: FieldKind::Password,
                value: Some(EDITED_SECRET.into()),
                secret: true,
                custom: false,
            },
            EditField {
                id: None,
                label: "Recovery email".into(),
                kind: FieldKind::Text,
                value: Some("octocat@example.com".into()),
                secret: false,
                custom: true,
            },
        ],
    );
    log.record("update_item", &edited);
    edited.expect("the item exists and the edits name real fields");

    let after_edit = items::get_item_inner(&state, item_id);
    log.record("get_item", &after_edit);
    let after_edit = after_edit.expect("the item still exists");
    assert_eq!(after_edit.summary.title, "GitHub (work)");
    assert!(after_edit.summary.favourite);
    assert_eq!(after_edit.summary.tags, vec!["dev".to_owned()]);
    assert_eq!(
        after_edit.fields[0].value.as_deref(),
        Some(USERNAME),
        "the untouched field kept its value — `value: null` means unchanged, not empty"
    );
    assert_eq!(after_edit.fields.len(), 3, "the new field was appended");
    assert!(after_edit.fields[2].custom);

    // The replacement is stored, and reading it back is still an explicit reveal — logged
    // like every other crossing, because a step left out of the transcript is a hole in the
    // evidence rather than a step that did not happen.
    let re_revealed = items::reveal_field_inner(&state, item_id, secret_field.id).map(|(r, _)| r);
    log.record("reveal_field", &re_revealed);
    assert_eq!(
        re_revealed.expect("revealable").value,
        EDITED_SECRET,
        "the edit landed in the vault, not just in the response"
    );

    // ---- Delete: the item goes, and so does every way of reading it ------------------------
    let deleted = items::delete_item_inner(&state, item_id);
    log.record("delete_item", &deleted);
    deleted.expect("the item exists");

    let after_delete = items::list_items_inner(&state);
    log.record("list_items", &after_delete);
    assert!(
        after_delete.expect("unlocked").is_empty(),
        "the deleted item is gone from the list"
    );
    let gone = items::get_item_inner(&state, item_id);
    log.record("get_item", &gone);
    assert!(gone.is_err(), "a deleted item cannot be opened");
    log.record("delete_item", &items::delete_item_inner(&state, item_id));

    // ---- Lock, then prove the shell is closed --------------------------------------------
    state.lock();
    log.record_infallible("vault_status", &state.status());
    log.record("list_items", &items::list_items_inner(&state));
    log.record("get_item", &items::get_item_inner(&state, item_id));
    log.record(
        "reveal_field",
        &items::reveal_field_inner(&state, item_id, secret_field.id).map(|(r, _)| r),
    );

    // ---- Unlock with the recovery kit, which spends it and issues a fresh one -------------
    let recovered = vault_cmd::unlock_recovery_kit_inner(
        &state,
        path.display().to_string(),
        recovery_code.clone(),
    );
    log.record("unlock_recovery_kit", &recovered);
    let reissued = recovered.expect("the kit opens it").recovery_code;
    assert_ne!(reissued, recovery_code, "using a kit issues a new one");

    let spent =
        vault_cmd::unlock_recovery_kit_inner(&state, path.display().to_string(), recovery_code);
    log.record("unlock_recovery_kit", &spent);
    assert!(spent.is_err(), "the spent kit no longer opens the vault");

    // ---- Read the transcript --------------------------------------------------------------
    log.write_to(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target/ipc-session.log")
            .as_path(),
    );

    // The master password crosses inbound and must never come back.
    assert!(
        log.carrying(MASTER).is_empty(),
        "the master password came back out of the boundary: {:?}",
        log.carrying(MASTER)
    );

    // The stored secret crosses outbound only where the user asked for it, one command per
    // action. Two reveals were scripted: one on an unlocked vault, one on a locked one, and
    // the second must have been refused.
    assert_eq!(
        log.carrying(SECRET),
        vec!["reveal_field"],
        "the stored secret crossed somewhere other than a reveal"
    );

    // The same for the value typed into the **edit** form, and it is the stronger statement of
    // the two: it crossed inbound through `update_item`, so every response after that point —
    // the read-back, the list, the deletion, the locked refusals — is a chance to echo it.
    // Only the one explicit reveal may.
    assert_eq!(
        log.carrying(EDITED_SECRET),
        vec!["reveal_field"],
        "a value submitted through update_item came back outside a reveal"
    );

    // `add_item`, `update_item` and `delete_item` return an identifier or nothing at all. The
    // check is by name, because what a later edit would change is the shape.
    for crossing in log
        .crossings
        .iter()
        .filter(|c| matches!(c.command, "add_item" | "update_item" | "delete_item"))
    {
        for secret in [SECRET, EDITED_SECRET, MASTER] {
            assert!(
                !crossing.payload.contains(secret),
                "{} echoed a value it was given",
                crossing.command
            );
        }
        assert!(
            !crossing.payload.contains(USERNAME),
            "{} returns no field value of any kind",
            crossing.command
        );
    }

    // Never more than one secret per invocation — R-10, over the whole session rather than
    // one response at a time.
    for crossing in &log.crossings {
        assert!(
            crossing.payload.matches(SECRET).count() <= 1,
            "{} returned the secret more than once in a single invocation",
            crossing.command
        );
    }

    // `copy_field` returns no value at all. Asserted by name rather than by shape, because
    // the shape is what a future edit would change.
    for crossing in log.crossings.iter().filter(|c| c.command == "copy_field") {
        assert!(
            !crossing.payload.contains(SECRET),
            "copy_field returned the value it is defined by not returning"
        );
        assert!(
            !crossing.payload.contains(USERNAME),
            "copy_field returns no field value of any kind"
        );
    }

    // The list carries no field value at all, secret or public.
    for crossing in log.crossings.iter().filter(|c| c.command == "list_items") {
        assert!(
            !crossing.payload.contains(USERNAME),
            "list_items carried a field value"
        );
    }

    // Both recovery codes are secrets and both are accounted for: each appears in exactly the
    // one response that minted it.
    assert_eq!(
        log.carrying(&reissued),
        vec!["unlock_recovery_kit"],
        "the reissued kit crossed outside the command that issued it"
    );

    let _ = fs::remove_file(&path);
}

/// The profile survives a lock and a relaunch, and an unchanged one does not rewrite the file.
///
/// D-70. Driven against a real file rather than in `ipc_audit.rs`, because setting a profile
/// **saves**, and the point of the test is what is on disk afterwards. Two properties, and the
/// second is the one that would go wrong silently: the Edit-profile dialog's Save is pressed
/// whether or not anything was typed, so a `set_profile` that wrote unconditionally would
/// re-encrypt and atomically replace the whole vault file on every open-and-cancel.
#[test]
fn a_profile_persists_across_a_relaunch_and_an_unchanged_one_writes_nothing() {
    let path = scratch_path();
    let state = AppState::default();

    vault_cmd::create_vault_inner(
        &state,
        "Session Vault".into(),
        path.display().to_string(),
        MASTER.into(),
        KdfSummary {
            m_cost: KdfParams::TESTING.m_cost,
            t_cost: KdfParams::TESTING.t_cost,
            p_cost: KdfParams::TESTING.p_cost,
        },
    )
    .expect("a writable path");
    vault_cmd::commit_vault_inner(&state).expect("the pending vault is written");

    assert_eq!(
        state.status().profile.expect("open").name,
        "",
        "onboarding's three steps do not ask, so a new vault has no owner named"
    );

    vault_cmd::set_profile_inner(
        &state,
        "  Budi Santoso  ".into(),
        "budi@warungpintar.id".into(),
    )
    .expect("an open vault takes a profile");

    let stored = state.status().profile.expect("still open");
    assert_eq!(stored.name, "Budi Santoso", "trimmed on the way in");
    assert_eq!(stored.email, "budi@warungpintar.id");

    // Submitting the same values again must not touch the file. Compared by modification time
    // rather than by bytes, because every save draws a fresh nonce — identical content
    // re-encrypts to different ciphertext, so equal bytes could never have been the assertion.
    let before = fs::metadata(&path)
        .expect("the vault exists")
        .modified()
        .ok();
    vault_cmd::set_profile_inner(&state, "Budi Santoso".into(), "budi@warungpintar.id".into())
        .expect("still open");
    let after = fs::metadata(&path).expect("still there").modified().ok();
    assert_eq!(before, after, "an unchanged profile is not a save");

    // Quit, relaunch, unlock: the profile came off disk, not out of memory.
    drop(state);
    let state = AppState::default();
    assert!(
        state.status().profile.is_none(),
        "nothing is open, so nothing is knowable"
    );
    vault_cmd::unlock_inner(&state, path.display().to_string(), MASTER.into()).expect("opens");

    let reopened = state.status().profile.expect("open again");
    assert_eq!(reopened.name, "Budi Santoso");
    assert_eq!(reopened.email, "budi@warungpintar.id");

    let _ = fs::remove_file(&path);
}

/// The transcript is written, and it is written where the gate can find it.
///
/// Separate from the assertions above so that a failure here reads as "the evidence is
/// missing" rather than "the boundary leaked".
#[test]
fn the_transcript_is_gate_evidence_and_says_so() {
    let mut log = SessionLog::default();
    log.record_infallible("vault_status", &serde_json::json!({"state": "no_vault"}));
    let path =
        std::env::temp_dir().join(format!("trustvault-transcript-{}.log", std::process::id()));
    log.write_to(&path);

    let written = fs::read_to_string(&path).expect("the transcript is written");
    assert!(
        written.contains("G-B′ line 4"),
        "it names the gate it serves"
    );
    assert!(
        written.contains("vault_status -> "),
        "and it holds the crossings"
    );
    let _ = fs::remove_file(&path);
}
