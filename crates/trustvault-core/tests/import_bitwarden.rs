//! R-29: a Bitwarden export imports with **every field either mapped or named in a refusal**.
//!
//! The acceptance criterion is not "it imports". D-42's survey found that the documented
//! failure of every importer looked at is the same one — a field dropped in silence — so the
//! test that matters here is [`nothing_in_the_export_is_dropped_in_silence`], which walks the
//! fixture generically and demands an account of every value in it. The named tests below it
//! pin the individual mappings; that one pins the *rule*, and it is the one that will fail when
//! somebody adds a type and maps four fields out of five.
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

use serde_json::Value;
use trustvault_core::{Field, FieldKind, ImportReport, Item, ItemKind, Vault};

use common::{PARAMS, PASSWORD};

/// The committed export. No real credential is in it and none may ever be — it is public,
/// like the rest of this repository (D-17).
fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/bitwarden-export.json")
}

fn fixture_json() -> Value {
    serde_json::from_str(&std::fs::read_to_string(fixture_path()).unwrap()).unwrap()
}

/// An empty vault to import into.
fn vault() -> Vault {
    Vault::create("Import target", PASSWORD, PARAMS).unwrap().0
}

/// The imported item with that title.
fn item<'a>(vault: &'a Vault, title: &str) -> &'a Item {
    vault
        .items()
        .find(|item| item.title == title)
        .unwrap_or_else(|| panic!("no item titled {title}"))
}

/// The first field with that label, whether it is one of the type's own or a custom one.
fn field<'a>(item: &'a Item, label: &str) -> &'a Field {
    item.fields
        .iter()
        .find(|field| field.label == label)
        .unwrap_or_else(|| panic!("{} has no field labelled {label}", item.title))
}

fn refusal_fields(report: &ImportReport) -> Vec<&str> {
    report
        .refusals
        .iter()
        .map(|refusal| refusal.field.as_str())
        .collect()
}

#[test]
fn the_fixture_imports_as_seven_items_of_five_kinds() {
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();

    // Eight items in the export, one of them in Bitwarden's trash.
    assert_eq!(report.total, 7);
    assert_eq!(vault.items().count(), 7);

    let counts: Vec<(ItemKind, usize)> = report
        .per_kind
        .iter()
        .map(|entry| (entry.kind, entry.count))
        .collect();
    assert_eq!(
        counts,
        vec![
            (ItemKind::Login, 2),
            (ItemKind::Card, 1),
            // The secure note, and the bank account that has no type of its own.
            (ItemKind::Note, 2),
            (ItemKind::SshKey, 1),
            (ItemKind::Identity, 1),
        ],
        "kinds are counted in the New-item dialog's own order"
    );
}

#[test]
fn api_key_and_wifi_are_not_reachable_from_a_bitwarden_export() {
    // Not an omission — Bitwarden has no such types, so producing one would mean guessing
    // from a title, which is the inference D-43 exists to prevent. Written as a test because
    // the tempting "improvement" is a heuristic, and a heuristic is invisible once merged.
    let mut vault = vault();
    vault.import_bitwarden(fixture_path()).unwrap();

    assert!(
        vault
            .items()
            .all(|item| item.kind != ItemKind::ApiKey && item.kind != ItemKind::WiFi)
    );
}

#[test]
fn a_totp_seed_reaches_the_totp_field_and_not_a_note() {
    // The failure every importer surveyed for D-42 makes.
    let mut vault = vault();
    vault.import_bitwarden(fixture_path()).unwrap();

    let login = item(&vault, "Example Login");
    let totp = field(login, "2FA secret");
    assert_eq!(totp.kind, FieldKind::Otp);
    assert!(totp.secret);
    assert_eq!(totp.value.expose(), "JBSWY3DPEHPK3PXP");

    let note = field(login, "Note");
    assert!(
        !note.value.expose().contains("JBSWY3DPEHPK3PXP"),
        "the seed must not have been swept into the note"
    );
}

#[test]
fn a_login_maps_onto_the_fields_our_own_dialog_draws() {
    let mut vault = vault();
    vault.import_bitwarden(fixture_path()).unwrap();
    let login = item(&vault, "Example Login");

    assert_eq!(login.kind, ItemKind::Login);
    assert_eq!(field(login, "Username").value.expose(), "fixture-user");
    assert_eq!(field(login, "Password").value.expose(), "fixture-password");
    assert_eq!(
        field(login, "Website").value.expose(),
        "https://example.com"
    );
    assert!(field(login, "Password").secret);
    assert!(!field(login, "Username").secret);
    assert!(login.favourite);

    // A second URI has nowhere of its own to go, so it arrives as a custom field rather than
    // overwriting the first — which is what `push_field` never merging buys (D-43).
    let second = field(login, "Website 2");
    assert_eq!(second.value.expose(), "https://login.example.com");
    assert!(second.custom);

    // Bitwarden keeps old passwords per item; we keep them per field.
    assert_eq!(login.history.len(), 2);
    let password_id = field(login, "Password").id;
    assert!(
        login
            .history
            .iter()
            .all(|entry| entry.field_id == password_id)
    );
}

#[test]
fn two_custom_fields_with_one_name_both_survive() {
    // Bitwarden permits it and collapsing them is a field dropped in silence, which is exactly
    // what R-29 refuses to call an import.
    let mut vault = vault();
    vault.import_bitwarden(fixture_path()).unwrap();
    let login = item(&vault, "Example Login");

    let answers: Vec<&str> = login
        .fields
        .iter()
        .filter(|field| field.label == "Security question")
        .map(|field| field.value.expose())
        .collect();
    assert_eq!(answers, vec!["fixture-answer", "fixture-second-answer"]);

    // A hidden custom field stays hidden, and `secret` is stored rather than re-inferred.
    let hidden = field(login, "Recovery code");
    assert!(hidden.secret && hidden.custom);

    // A custom field called "Username" would collide with the login's own; this fixture's
    // custom fields do not, but the item's own username must still be the one from `login`.
    assert_eq!(field(login, "Username").value.expose(), "fixture-user");
    assert!(!field(login, "Username").custom);
}

#[test]
fn a_card_joins_the_two_expiry_keys_and_says_so() {
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();
    let card = item(&vault, "Example Card");

    assert_eq!(field(card, "Expiry").value.expose(), "8 / 2029");
    assert_eq!(
        field(card, "Card number").value.expose(),
        "4111111111111111"
    );
    assert!(field(card, "Card number").secret && field(card, "CVV").secret);
    assert_eq!(field(card, "Brand").value.expose(), "Visa");

    assert!(
        report.converted.iter().any(|converted| {
            converted.item_title == "Example Card" && converted.field.contains("expMonth")
        }),
        "a shape change the user cannot see in the item must be in the report"
    );
}

#[test]
fn an_identity_keeps_the_fields_our_type_has_no_room_for() {
    let mut vault = vault();
    vault.import_bitwarden(fixture_path()).unwrap();
    let identity = item(&vault, "Example Identity");

    assert_eq!(
        field(identity, "Full name").value.expose(),
        "Fixture Q Person"
    );
    assert_eq!(
        field(identity, "Email").value.expose(),
        "fixture@example.com"
    );
    // Everything else lands as a custom field rather than being refused: the data is the
    // user's and a field they can read beats a line in a report they cannot act on.
    assert_eq!(
        field(identity, "Passport number").value.expose(),
        "X0000000"
    );
    assert!(field(identity, "Passport number").secret);
    assert!(field(identity, "National ID").secret);
    assert!(!field(identity, "City").secret);
    // Ten of the export's thirteen leftovers; the three it left null are not invented here.
    assert_eq!(identity.custom_fields().count(), 10);
}

#[test]
fn an_ssh_key_keeps_its_key_material_as_custom_fields() {
    // Our SSH key type holds a host, a user and a passphrase — none of which Bitwarden
    // exports — so this is the type where the two models overlap least.
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();
    let key = item(&vault, "Example SSH Key");

    assert_eq!(key.kind, ItemKind::SshKey);
    assert!(field(key, "Private key").secret);
    assert!(!field(key, "Public key").secret);
    assert!(
        report
            .converted
            .iter()
            .any(|converted| converted.item_title == "Example SSH Key")
    );
}

#[test]
fn a_type_this_build_does_not_know_becomes_a_note_rather_than_a_hole() {
    // Bank account, driving licence and passport are types 6, 7 and 8, and there will be a 9.
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();
    let account = item(&vault, "Example Bank Account");

    assert_eq!(account.kind, ItemKind::Note);
    assert_eq!(field(account, "bankName").value.expose(), "Fixture Bank");
    assert_eq!(field(account, "accountNumber").value.expose(), "0000000000");
    assert!(
        field(account, "accountNumber").secret,
        "nothing is known about what an unknown type holds, so it is masked"
    );
    assert_eq!(field(account, "verified").value.expose(), "false");
    assert!(report.converted.iter().any(|converted| {
        converted.item_title == "Example Bank Account" && converted.field == "type"
    }));
    // A list cannot live in a field, so it is named rather than flattened into one.
    assert!(
        refusal_fields(&report)
            .iter()
            .any(|field| field.contains("branches"))
    );
}

#[test]
fn an_item_bitwarden_had_deleted_is_not_resurrected() {
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();

    assert!(
        vault
            .items()
            .all(|item| item.title != "Example Deleted Login")
    );
    assert!(report.refusals.iter().any(|refusal| {
        refusal.item_title == "Example Deleted Login" && refusal.field == "deletedDate"
    }));
}

#[test]
fn everything_without_a_home_is_named() {
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();
    let fields = refusal_fields(&report);

    for expected in [
        "login.fido2Credentials[0]", // passkeys
        "login.uris[1].match",       // browser auto-fill
        "fields/Auto-fill username", // a linked field
        "attachments/statement.pdf",
        "collectionIds",
        "organizationId",
        "reprompt",
        "revisionDate",     // "not-a-timestamp"
        "folderId",         // names a folder the export does not define
        "quantumEntangled", // a key no version of this build knows
        "bankAccount.branches",
    ] {
        assert!(
            fields.contains(&expected),
            "expected a refusal naming {expected}, got {fields:?}"
        );
    }
}

#[test]
fn a_report_carries_no_value_from_the_vault_it_describes() {
    // `docs/ipc-contract.md` §6.8. A report that quoted what it could not import would be a
    // plaintext dump of exactly the parts of the foreign vault we understood least — and it
    // crosses IPC into a heap that cannot be wiped.
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();
    let serialized = serde_json::to_string(&report).unwrap();

    for secret in [
        "fixture-password",
        "fixture-user",
        "JBSWY3DPEHPK3PXP",
        "4111111111111111",
        "fixture-hidden-value",
        "fixture-old-password-1",
        "0000000000",
        "BEGIN OPENSSH PRIVATE KEY",
    ] {
        assert!(
            !serialized.contains(secret),
            "the report quotes {secret}: {serialized}"
        );
    }
}

#[test]
fn folders_become_tags_and_an_existing_tag_is_merged_rather_than_duplicated() {
    let mut vault = vault();
    let seeded = vault.add_item(ItemKind::Login, "Already here");
    vault
        .item_mut(seeded)
        .unwrap()
        .set_tags(["Personal".to_owned()]);

    let report = vault.import_bitwarden(fixture_path()).unwrap();

    assert_eq!(report.tags_created, vec!["Work/Clients".to_owned()]);
    assert_eq!(report.tags_merged, vec!["Personal".to_owned()]);

    // Verbatim, not split: `Work/Clients` is one tag, because splitting it would claim a
    // hierarchy tags do not have (D-43).
    assert_eq!(item(&vault, "Example Login").tags, vec!["Work/Clients"]);
    let all_tags: Vec<&String> = vault.items().flat_map(|item| item.tags.iter()).collect();
    assert_eq!(
        all_tags.iter().filter(|tag| **tag == "Personal").count(),
        3,
        "three items carry the one tag; a merge is not a rename"
    );
}

#[test]
fn a_preview_commits_nothing_and_reports_what_the_import_would_do() {
    let mut vault = vault();
    let preview = vault.preview_bitwarden(fixture_path()).unwrap();
    assert_eq!(vault.items().count(), 0, "a preview writes nothing");

    let committed = vault.import_bitwarden(fixture_path()).unwrap();
    assert_eq!(preview, committed);
    assert_eq!(vault.items().count(), 7);
}

#[test]
fn a_file_that_is_not_an_unencrypted_export_lands_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let cases = [
        ("garbage.json", "this is not JSON at all {{{"),
        ("empty.json", "{}"),
        ("encrypted.json", r#"{"encrypted":true,"items":[]}"#),
        ("wrong-schema.json", r#"{"vault":[{"title":"x"}]}"#),
    ];

    for (name, contents) in cases {
        let path = directory.path().join(name);
        std::fs::write(&path, contents).unwrap();
        let mut vault = vault();
        assert!(
            vault.import_bitwarden(&path).is_err(),
            "{name} must not import"
        );
        assert_eq!(vault.items().count(), 0, "{name} left something behind");
    }
}

#[test]
fn imported_items_survive_a_save_and_reopen() {
    // The import is only real if it is still there after the file is closed — the same line
    // the Phase 3 gate asks a human to run by hand for an item created through the UI.
    let mut vault = vault();
    vault.import_bitwarden(fixture_path()).unwrap();
    let bytes = vault.to_bytes().unwrap();
    drop(vault);

    let reopened = Vault::open(&bytes, PASSWORD).unwrap();
    assert_eq!(reopened.items().count(), 7);
    let login = item(&reopened, "Example Login");
    assert_eq!(field(login, "Password").value.expose(), "fixture-password");
    assert_eq!(login.tags, vec!["Work/Clients"]);
}

/// Keys that carry no user content, so having no home for them is not a loss.
///
/// Each one is justified in `import/bitwarden.rs`'s module documentation. The list is here as
/// well as there so that adding to it is a visible edit to a test, rather than a quiet way to
/// make [`nothing_in_the_export_is_dropped_in_silence`] pass.
const STRUCTURAL: [&str; 12] = [
    "id",
    "organizationId",
    "collectionIds",
    "folderId",
    "type",
    "reprompt",
    "favorite",
    "key",
    "encrypted",
    "linkedId",
    "match",
    "secureNote",
];

/// Keys whose value is mapped somewhere no string comparison can see it.
const MAPPED_ELSEWHERE: [&str; 4] = [
    "creationDate",
    "revisionDate",
    "deletedDate",
    "lastUsedDate",
];

/// Subtrees refused whole, where refusing the parent accounts for the children.
const REFUSED_SUBTREES: [&str; 2] = ["attachments", "fido2Credentials"];

#[test]
fn nothing_in_the_export_is_dropped_in_silence() {
    // The acceptance criterion of R-29, tested as a rule rather than as a list: walk every
    // leaf of the fixture and demand that its value is either in the vault or named in a
    // refusal. It is deliberately annoying to satisfy — a mapping that drops a field fails
    // here even if every named test above still passes.
    let mut vault = vault();
    let report = vault.import_bitwarden(fixture_path()).unwrap();

    let stored: String = vault
        .items()
        .flat_map(|item| {
            std::iter::once(item.title.clone())
                .chain(item.tags.iter().cloned())
                .chain(
                    item.fields
                        .iter()
                        .map(|field| field.value.expose().to_owned()),
                )
                .chain(
                    item.history
                        .iter()
                        .map(|entry| entry.value.expose().to_owned()),
                )
        })
        .collect::<Vec<String>>()
        .join("\u{1}");
    let refused = refusal_fields(&report).join("\u{1}");

    let mut unaccounted: Vec<String> = Vec::new();
    walk(&fixture_json(), &mut Vec::new(), &mut |path, value| {
        // `items5` is the sixth item; the lists above name keys, not positions.
        let parts: Vec<&str> = path
            .iter()
            .map(|part| part.trim_end_matches(|character: char| character.is_ascii_digit()))
            .collect();
        let key = parts.last().copied().unwrap_or_default();
        if STRUCTURAL.contains(&key)
            || MAPPED_ELSEWHERE.contains(&key)
            || parts.iter().any(|part| REFUSED_SUBTREES.contains(part))
        {
            return;
        }

        let text = match value {
            Value::String(text) if !text.trim().is_empty() => text.clone(),
            Value::Bool(flag) => flag.to_string(),
            _ => return,
        };
        // A value is accounted for if it is in the vault, or if any refusal names a key on
        // its path — the deleted item's whole subtree is covered by the one refusal that
        // names `deletedDate`, because the item did not come in at all.
        let named = parts.iter().any(|part| refused.contains(part));
        if !stored.contains(&text) && !named {
            unaccounted.push(format!("{} = {text}", path.join(".")));
        }
    });

    assert!(
        unaccounted.is_empty(),
        "these values were neither imported nor refused: {unaccounted:#?}"
    );
}

/// Calls `visit` for every leaf of a JSON document, with the path that reached it.
fn walk(value: &Value, path: &mut Vec<String>, visit: &mut impl FnMut(&[String], &Value)) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                path.push(key.clone());
                walk(child, path, visit);
                path.pop();
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let last = path.pop().unwrap_or_default();
                path.push(format!("{last}{index}"));
                walk(child, path, visit);
                path.pop();
                path.push(last);
            }
        }
        leaf => visit(path, leaf),
    }
}
