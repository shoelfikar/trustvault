//! The IPC audit harness — `docs/ipc-contract.md` §9.
//!
//! This is the enforcement behind the contract. Every rule the contract states as MUST is
//! either checked here or explicitly marked in the contract as unchecked; a rule nobody can
//! check is a comment.
//!
//! The harness calls the **real command bodies** through their `_inner` functions. A harness
//! that reimplemented the boundary would prove only that the reimplementation is safe.

// An integration test is its own crate with no `#[cfg(test)]` module, so clippy's
// `allow-unwrap-in-tests` does not reach it and the workspace's Tier-1 lints apply at full
// force. Lifted here, with the reason, exactly as crates/trustvault-core/tests/ does.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use trustvault_core::{CharSets, FieldId, FieldKind, ItemId, ItemKind, KdfParams, Vault};
use trustvault_lib::commands::{generator, import, items, search, totp, vault as vault_cmd};
use trustvault_lib::dto::{Copied, MASK};
use trustvault_lib::error::ErrorKind;
use trustvault_lib::state::AppState;

/// The secret planted in every fixture. If this string appears in any payload that is not a
/// sanctioned response, the boundary leaked.
const SECRET: &str = "correct-horse-battery-staple";
/// A non-secret field value, which is allowed to cross freely (`docs/ipc-contract.md` §6.1).
const USERNAME: &str = "octocat";
/// A TOTP seed, base32 of the RFC 6238 test key. As secret as the password beside it: the code
/// it produces may cross this boundary (D-45), the seed never may.
const SEED: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

/// An unlocked state holding one login with one secret and one public field.
fn unlocked() -> (AppState, ItemId, FieldId, FieldId) {
    let (mut vault, _) =
        Vault::create("Personal Vault", "master pw", KdfParams::TESTING).expect("valid params");
    let item = vault.add_item(ItemKind::Login, "GitHub");
    let entry = vault.item_mut(item).expect("just added");
    let public = entry.set_field("Username", USERNAME, false);
    let secret = entry.set_field("Password", SECRET, true);
    // The seed goes into the shared fixture rather than into the TOTP test alone, so that
    // every check below — the list, the detail, the search, the session — is asserting against
    // a vault that holds one. A second secret shape only the test that knows about it can see
    // is a secret shape the other checks are not checking.
    let seed = entry.set_field("2FA secret", SEED, true);
    if let Some(field) = entry.fields.iter_mut().find(|field| field.id == seed) {
        // Set explicitly, as both real write paths do: `set_field` guesses `password` from
        // `secret`, which is the trap D-53 records one requirement over.
        field.kind = FieldKind::Otp;
    }

    let state = AppState::default();
    state.with(|inner| {
        inner.vault = Some(vault);
        inner.path = Some(PathBuf::from("/tmp/personal.tvault"));
    });
    (state, item, secret, public)
}

/// Reads the command list out of the `generate_handler!` block in `src/lib.rs`.
///
/// Parsed from source rather than maintained as a second list, because a second list is one
/// that drifts. It is only ever compared against the contract, never used to dispatch.
fn registered_commands() -> Vec<String> {
    let source = include_str!("../src/lib.rs");
    let start = source
        .find("generate_handler![")
        .expect("lib.rs registers commands");
    let block = &source[start..];
    let end = block.find(']').expect("the handler list is closed");
    block[..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            // Command paths only: skip the macro line itself and the section comments.
            line.strip_prefix("commands::")
                .and_then(|path| path.rsplit("::").next())
                .map(str::to_owned)
        })
        .collect()
}

/// Reads the command names the contract documents, split into shipped and planned.
///
/// A declaration carrying a trailing `// planned` marker is specified but not yet registered
/// — §1 of the contract. The marker exists so the contract can go on being written before the
/// code, which is the practice that caught D-25 and four Phase 2 findings; it is checked in
/// **both** directions by the caller, so it cannot be used to park a command that shipped.
fn documented_commands() -> (Vec<String>, Vec<String>) {
    let contract = include_str!("../../docs/ipc-contract.md");
    let (mut shipped, mut planned) = (Vec::new(), Vec::new());
    for line in contract.lines() {
        // A command declaration in the contract looks like `name({...}): Shape` or `name():`.
        let Some(open) = line.find('(') else { continue };
        let name = &line[..open];
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
            && line[open..].contains("):")
        {
            if line.contains("// planned") {
                planned.push(name.to_owned());
            } else {
                shipped.push(name.to_owned());
            }
        }
    }
    for set in [&mut shipped, &mut planned] {
        set.sort();
        set.dedup();
    }
    (shipped, planned)
}

/// Check 1 — the registered set and the documented set are the same set.
///
/// The failure this exists for is a command that exists but is undocumented: it is how a
/// fourth sanctioned command arrives without anyone deciding to add one.
#[test]
fn every_command_is_documented_and_every_documented_command_exists() {
    let mut registered = registered_commands();
    registered.sort();
    let (shipped, planned) = documented_commands();

    let undocumented: Vec<_> = registered
        .iter()
        .filter(|name| !shipped.contains(name))
        .collect();
    assert!(
        undocumented.is_empty(),
        "commands registered but absent from docs/ipc-contract.md: {undocumented:?}"
    );

    let unimplemented: Vec<_> = shipped
        .iter()
        .filter(|name| !registered.contains(name))
        .collect();
    assert!(
        unimplemented.is_empty(),
        "commands documented but not registered: {unimplemented:?} — \
         if one is still being built, mark its declaration `// planned`"
    );

    // The other direction, which is the half that keeps the marker honest. A command that
    // shipped while its declaration still says `// planned` is invisible to the check above:
    // it would be registered, documented, and excluded from both comparisons at once.
    let stale: Vec<_> = planned
        .iter()
        .filter(|name| registered.contains(name))
        .collect();
    assert!(
        stale.is_empty(),
        "registered but still marked `// planned` in docs/ipc-contract.md: {stale:?} — \
         delete the marker in the commit that implements the command"
    );
}

/// Every command names its arguments in `snake_case`, which is the wire format the contract
/// documents.
///
/// Found 2026-08-06 by reading the built binary rather than by any test: Tauri v2's
/// `#[tauri::command]` renames argument keys to **camelCase** by default, and
/// `tauri::ipc::CommandItem` looks that key up exactly, with no fallback. So the host was
/// asking for `itemId` while `src/lib/ipc.ts` — which converts to snake_case on purpose, so
/// the wire format is exactly what `docs/ipc-contract.md` §6 prints — sent `item_id`. Every
/// command taking a multi-word argument was unreachable from the webview: `get_item`,
/// `reveal_field`, `copy_field`.
///
/// It survived two phases because nothing had exercised one. The `_inner` split that lets this
/// harness drive real command bodies also skips the argument decoding, and until `add_item`
/// landed the item list was always empty, so no id was ever passed from the frontend.
///
/// The attribute is asserted on **every** command rather than only the ones that need it
/// today: what makes the bug expensive is that adding a two-word argument reintroduces it
/// silently, and a uniform rule has no such edge.
#[test]
fn every_command_names_its_arguments_in_snake_case() {
    const REQUIRED: &str = r#"#[tauri::command(rename_all = "snake_case")]"#;

    let modules = [
        ("items.rs", include_str!("../src/commands/items.rs")),
        ("vault.rs", include_str!("../src/commands/vault.rs")),
        ("settings.rs", include_str!("../src/commands/settings.rs")),
        ("strength.rs", include_str!("../src/commands/strength.rs")),
        ("import.rs", include_str!("../src/commands/import.rs")),
        ("generator.rs", include_str!("../src/commands/generator.rs")),
        ("search.rs", include_str!("../src/commands/search.rs")),
        ("totp.rs", include_str!("../src/commands/totp.rs")),
    ];

    // `include_str!` needs a literal path, so the list above is written by hand — and a
    // hand-written list of the files in a directory is exactly the thing that goes stale on
    // the day someone adds one. This reads the directory to prove it has not: a new command
    // module that nobody added above would otherwise be silently exempt from the whole check.
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands");
    for entry in std::fs::read_dir(&directory).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name == "mod.rs" {
            continue;
        }
        assert!(
            modules.iter().any(|(module, _)| *module == name),
            "src/commands/{name} is not in this test's module list, so its commands are \
             unchecked — add it beside the others"
        );
    }

    for (module, source) in modules {
        for (number, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("#[tauri::command") {
                assert_eq!(
                    line,
                    REQUIRED,
                    "src/commands/{module}:{} declares a command without \
                     `rename_all = \"snake_case\"` — Tauri v2 would then look its arguments up \
                     in camelCase and every call from src/lib/ipc.ts would miss",
                    number + 1
                );
            }
        }
    }
}

/// Check 3 — exactly four commands may return a secret, and they are the named four.
///
/// The count moved from three to four on 2026-08-05, which is the one change this test exists
/// to make expensive: it asserts the sentence **and** that a decision is cited beside it by
/// number, so the budget cannot be raised by editing prose alone.
///
/// It cannot check that the decision log actually holds that row, and the reason is worth
/// recording rather than working around: `trustvault-state.md` is gitignored — the process
/// record stays on the author's disk while the code is public — so a test that read it would
/// compile here and fail to compile in CI. Written that way first, and caught by looking at
/// `.gitignore` rather than by CI, which is the cheaper of the two.
#[test]
fn the_sanctioned_set_is_exactly_four_and_unchanged() {
    let contract = include_str!("../../docs/ipc-contract.md");
    for name in [
        "create_vault",
        "unlock_recovery_kit",
        "reveal_field",
        "generate_password",
    ] {
        assert!(
            contract.contains(name),
            "{name} is a sanctioned command and must stay documented"
        );
    }
    assert!(
        contract.contains("There are **four** sanctioned commands"),
        "the budget is the contract's load-bearing sentence; if it changed, \
         a decision log entry should have changed with it"
    );

    assert!(
        contract.contains("since **D-44** — `generate_password`"),
        "the budget may only move with a decision cited beside it, by number"
    );
}

/// Checks 2 and 4 — a list carries no secret, and a detail carries at most a mask.
#[test]
fn no_list_or_detail_response_carries_a_secret() {
    let (state, item, _, _) = unlocked();

    let list = serde_json::to_string(&items::list_items_inner(&state).unwrap()).unwrap();
    assert!(!list.contains(SECRET), "list_items leaked the password");
    assert!(
        !list.contains(USERNAME),
        "list_items carries no field values at all, secret or not"
    );

    let detail = serde_json::to_string(&items::get_item_inner(&state, item).unwrap()).unwrap();
    assert!(!detail.contains(SECRET), "get_item leaked the password");
    assert!(detail.contains(MASK), "the secret field crosses as a mask");
    assert!(
        detail.contains(USERNAME),
        "a field the user declared is not secret crosses as its value"
    );
    assert!(
        !detail.contains("history"),
        "history is absent from the shape, not merely elided"
    );
}

/// Checks 2 and 4 for the palette — a search response is a list, and lists carry no values.
///
/// The second assertion is the one worth having. It is not about the response shape, which is
/// `ItemSummary` and could not hold a value if it wanted to: it is that a query **equal to a
/// stored password matches nothing**. A palette that ranked on secret values would confirm a
/// guessed password through the order of its rows, with nothing crossing this boundary and no
/// audit entry written — a leak with no payload to find afterwards.
#[test]
fn a_search_response_carries_no_values_and_secrets_are_not_searchable() {
    let (state, _, _, _) = unlocked();

    let hits = search::search_items_inner(&state, "github", 10).unwrap();
    assert_eq!(hits.len(), 1, "the fixture's one item is found by title");

    let encoded = serde_json::to_string(&hits).unwrap();
    assert!(
        !encoded.contains(SECRET),
        "search_items leaked the password"
    );
    assert!(
        !encoded.contains(USERNAME),
        "search results carry no field values at all — not even the one that matched (D-46)"
    );

    // The username *is* searchable, which is R-16, and the value still does not come back.
    assert_eq!(
        search::search_items_inner(&state, USERNAME, 10)
            .unwrap()
            .len(),
        1,
        "a non-secret username is one of the four haystacks R-16 names"
    );

    assert!(
        search::search_items_inner(&state, SECRET, 10)
            .unwrap()
            .is_empty(),
        "a query equal to a stored password must not identify the item holding it"
    );
}

/// Check 7 — `totp_code` returns a code and never the seed — §6.6, §7.1, D-45.
///
/// D-45 draws the line at the seed rather than at the code, and the argument it explicitly
/// refuses is "the code expires soon" — that one would also license returning a password about
/// to be rotated. So what has to be pinned is not that the code crosses, which is the decision,
/// but the two limits the decision came with: the seed does not, and no *list* carries a code.
/// The second is the shape §2 exists to prevent — a list of live codes is a list of secrets on
/// a refresh timer, and it is one convenience commit away at any moment.
#[test]
fn totp_returns_a_code_and_never_the_seed() {
    let (state, item, _, _) = unlocked();

    let response = totp::totp_code_inner(&state, item).unwrap();
    assert_eq!(response.code.len(), 6, "a code did come back");

    let encoded = serde_json::to_string(&response).unwrap();
    assert!(!encoded.contains(SEED), "totp_code leaked the seed");
    assert!(
        !encoded.contains(SECRET),
        "and it is not a second path to the password beside it"
    );

    // The seed is elided everywhere a field is listed, exactly like the password: `kind: otp`
    // changes how it renders, never whether it crosses.
    let detail = serde_json::to_string(&items::get_item_inner(&state, item).unwrap()).unwrap();
    assert!(!detail.contains(SEED), "get_item leaked the seed");

    // No code in any list. Asserted on the *key* rather than on the six digits, because six
    // digits can occur inside a UUID by chance and a flaky boundary check is one that gets
    // deleted.
    let list = serde_json::to_string(&items::list_items_inner(&state).unwrap()).unwrap();
    assert!(!list.contains(SEED), "list_items leaked the seed");
    assert!(
        !list.contains("\"code\""),
        "no code appears in a list, however cheap it would be to add (§6.6)"
    );
}

/// Check 2 — the one sanctioned response carries exactly one secret and nothing else.
#[test]
fn reveal_returns_one_secret_and_copy_returns_none() {
    let (state, item, secret, _) = unlocked();

    let (revealed, _) = items::reveal_field_inner(&state, item, secret).unwrap();
    assert_eq!(revealed.value, SECRET, "the sanctioned path does reveal it");
    let encoded = serde_json::to_string(&revealed).unwrap();
    assert_eq!(
        encoded.matches(SECRET).count(),
        1,
        "exactly one secret per invocation (R-10)"
    );

    // copy_field is not exercised here: it writes to the real system clipboard, which a CI
    // runner may not have. Its response *shape* carries no value by construction -- `Copied`
    // has one field and it is an integer -- and that is asserted in the unit tests instead.
    // Named rather than silently skipped, because a check that quietly does not run is worse
    // than one that is documented as not running.
}

/// Check 2 again, for the fourth sanctioned command — R-10, D-44.
///
/// The generator is the one sanctioned command that returns a secret the vault has never seen,
/// so the two things worth pinning are both about its *class*: exactly one secret in the
/// response, and no vault required to get it. The second is why it is absent from the
/// locked-state check below — a generator that refused while locked could not fill the
/// password field of the first item in a brand-new vault.
#[test]
fn the_generator_returns_one_secret_and_needs_no_vault() {
    let (state, _, _, _) = unlocked();
    state.lock();

    let generated = generator::generate_password_inner(24, CharSets::ALL, true).unwrap();
    assert_eq!(generated.password.chars().count(), 24);

    let encoded = serde_json::to_string(&generated).unwrap();
    assert_eq!(
        encoded.matches(&generated.password).count(),
        1,
        "exactly one secret per invocation (R-10)"
    );

    // `copy_generated` is not exercised for the same reason `copy_field` is not: it writes to
    // the real system clipboard, which a CI runner may not have. What can be checked without
    // one is that its response shape carries no value at all — `Copied` has a single integer
    // field, so there is nowhere for the password it just copied to ride along.
    let copied = serde_json::to_string(&Copied { clears_at: 0 }).unwrap();
    assert_eq!(copied, r#"{"clears_at":0}"#);
}

/// Check 6 — every vault-class and sanctioned command answers `locked` when it is.
///
/// Worth more than it looks: this is the regression test for a webview reload. A reload leaves
/// the frontend's stores empty and its lock state whatever the host says, and the bug it
/// prevents is a command that reads a cached handle instead of re-checking.
#[test]
fn every_vault_command_refuses_while_locked() {
    let (state, item, secret, _) = unlocked();
    state.lock();

    assert_eq!(
        items::list_items_inner(&state).unwrap_err().kind,
        ErrorKind::Locked
    );
    assert_eq!(
        items::get_item_inner(&state, item).unwrap_err().kind,
        ErrorKind::Locked
    );
    assert_eq!(
        items::reveal_field_inner(&state, item, secret)
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
    assert_eq!(
        items::copy_field_inner(&state, item, secret)
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
    assert_eq!(
        search::search_items_inner(&state, "github", 10)
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
    // `totp_code` is vault-class and refuses; `totp_preview` is ambient and does not appear
    // here, for the same reason the generator does not — it reads no vault, and a seed being
    // typed into a dialog protects nothing yet.
    assert_eq!(
        totp::totp_code_inner(&state, item).unwrap_err().kind,
        ErrorKind::Locked
    );
    // The import pair refuses **before** it reads the file, which is why the path here does
    // not exist: a locked vault that still parses a foreign vault into this process would
    // have loaded plaintext nothing is going to lock.
    assert_eq!(
        import::import_preview_inner(&state, "/nonexistent/export.json")
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
    assert_eq!(
        import::import_commit_inner(&state, "/nonexistent/export.json")
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
}

/// A reload does not unlock anything, and nothing the frontend does can change that.
///
/// There is no command that sets the lock state to unlocked without a credential — `unlock`
/// and `unlock_recovery_kit` both go through the KDF. This test states the property by
/// showing that the state survives being read repeatedly and stays locked.
#[test]
fn a_locked_state_stays_locked_however_often_it_is_asked() {
    let (state, _, _, _) = unlocked();
    state.lock();

    for _ in 0..5 {
        let status = state.status();
        assert_eq!(
            status.state,
            trustvault_lib::dto::VaultState::Locked,
            "asking does not unlock"
        );
        assert_eq!(status.item_count, None);
    }
}

/// The reveal is recorded only when the setting is on, and the log never holds the value.
#[test]
fn the_audit_log_follows_the_setting_and_holds_no_secret() {
    let (state, item, secret, _) = unlocked();

    // Off by default (D-31).
    items::reveal_field_inner(&state, item, secret).unwrap();
    let recorded = state
        .with(|inner| inner.vault.as_ref().map(|v| v.audit_entries().len()))
        .flatten();
    assert_eq!(recorded, Some(0), "off by default means nothing is written");

    state.with(|inner| inner.settings.audit_log_enabled = true);
    items::reveal_field_inner(&state, item, secret).unwrap();

    let encoded = state
        .with(|inner| {
            inner
                .vault
                .as_ref()
                .map(|v| serde_json::to_string(v.audit_entries()).unwrap())
        })
        .flatten()
        .unwrap();
    assert!(encoded.contains(&item.to_string()), "the item is named");
    assert!(!encoded.contains(SECRET), "the value never is");
    assert!(!encoded.contains("Password"), "nor the label");
}

/// Revealing a field that is not secret is refused, so "reveal" means one thing everywhere.
#[test]
fn a_public_field_cannot_be_revealed() {
    let (state, item, _, public) = unlocked();
    assert_eq!(
        items::reveal_field_inner(&state, item, public)
            .unwrap_err()
            .kind,
        ErrorKind::NotSecret
    );
}

/// N-07 — the CSP names no wildcard origin, and the one relaxation stays the only one.
///
/// `style-src 'unsafe-inline'` is present because Svelte injects component styles, and that is
/// recorded as the single deliberate relaxation. Widening `script-src`, `connect-src`, or
/// `font-src` needs a decision log entry, so this test is what makes "needs an entry" cost
/// something.
#[test]
fn the_csp_has_no_wildcard_origin() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid config");
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("a CSP is configured");

    assert!(!csp.contains('*'), "no wildcard origin anywhere: {csp}");
    for directive in [
        "default-src 'self'",
        "script-src 'self'",
        "font-src 'self'",
        "object-src 'none'",
        "frame-src 'none'",
        "base-uri 'none'",
        "form-action 'none'",
    ] {
        assert!(csp.contains(directive), "missing `{directive}` in the CSP");
    }
    assert_eq!(
        csp.matches("unsafe-inline").count(),
        1,
        "exactly one relaxation, and it is style-src for Svelte's injected component styles"
    );
    assert!(
        csp.contains("style-src 'self' 'unsafe-inline'"),
        "the one relaxation is on style-src and nowhere else"
    );
}

/// The lock event's reason survives serialization, since the lock screen routes on it.
#[test]
fn the_lock_event_names_its_reason() {
    let encoded = serde_json::to_string(&vault_cmd::LockEvent {
        reason: trustvault_lib::state::LockReason::Timeout,
    })
    .unwrap();
    assert!(encoded.contains("timeout"));
}
