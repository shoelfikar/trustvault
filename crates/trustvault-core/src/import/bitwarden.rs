//! Bitwarden's unencrypted JSON export — R-29, D-42, and the only import path v1 ships.
//!
//! # Why this parser is shaped the way it is
//!
//! It reads **externally supplied, potentially hostile input** inside a Tier-1 crate: no
//! `unsafe`, no `unwrap`, no panic, and a 90 % coverage floor. That is affordable only because
//! the format costs exactly one pure-Rust dependency (`serde_json`) — no XML, no zip, no second
//! crypto stack. It was the argument for choosing this format over KDBX and it is the reason
//! nothing below reaches for a shortcut that would need a wider one.
//!
//! # Nothing falls off the end
//!
//! Every key Bitwarden writes is **declared** here, whether or not TrustVault has a use for it,
//! and anything left over lands in a `#[serde(flatten)]` map that becomes one refusal per key.
//! That is what keeps R-29 true against a schema that keeps growing: when Bitwarden adds a key
//! next year, the import does not quietly ignore it, it names it in the report.
//!
//! Five keys are declared and deliberately **not** imported, because they carry no user content:
//!
//! | Key | Why it is not a refusal |
//! |-----|-------------------------|
//! | `id` | Bitwarden's own identifier. Our items get fresh UUIDs; keeping theirs would claim a link to a vault we no longer read. |
//! | `folderId` | Not dropped — it is *resolved*, into the tag D-43 says a folder becomes. |
//! | `key` | Bitwarden's per-cipher wrapping key. In an unencrypted export the content beside it is already plaintext, so it protects nothing here. |
//! | `secureNote.type` | One enum with one value in every export seen; the note's content is in the item's `notes`. |
//! | `login.uris[].uriChecksum` | A checksum of a value we imported in full. |
//!
//! # Which of our seven types an import can produce
//!
//! Five: login, note, card, identity and SSH key. **`api_key` and `wifi` are not reachable from
//! a Bitwarden export and must not be**, because Bitwarden has no such types — reaching them
//! would mean guessing from a title, and guessing is the failure mode D-43 exists to prevent.
//! An API key kept in Bitwarden is a login or a secure note there, and it arrives here as what
//! it was stored as, for the user to re-type if they want it to be something else.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::import::{ImportReport, iso8601_utc_ms};
use crate::model::{Field, FieldKind, HistoryEntry, Item, ItemKind};
use crate::secret::SecretString;
use crate::{Error, Result};

/// Bitwarden's item type numbers, read off `CipherType` in `bitwarden/clients`.
///
/// Six, seven and eight — bank account, driving licence and passport — are newer than the
/// four everyone remembers, and they are the reason [`map_item`] has a generic fallback rather
/// than a `match` with four arms and an `unreachable`.
const TYPE_LOGIN: u8 = 1;
const TYPE_SECURE_NOTE: u8 = 2;
const TYPE_CARD: u8 = 3;
const TYPE_IDENTITY: u8 = 4;
const TYPE_SSH_KEY: u8 = 5;

/// The title given to an item whose `name` is missing or blank.
const UNTITLED: &str = "Untitled";

/// A parsed export, ready to be applied to a vault.
///
/// The items are built in full **before** anything touches the vault, which is what makes
/// `docs/ipc-contract.md` §6.8's one-transaction promise structural rather than careful: there
/// is no partial state to leave behind, because the vault is not written to until every item
/// exists.
#[derive(Debug)]
pub struct Parsed {
    /// The items, in export order.
    pub items: Vec<Item>,
    /// What was mapped, converted and refused. Tag counts are filled in by the vault, which
    /// is the only thing that knows which tags already exist.
    pub report: ImportReport,
}

/// Parses an export into items and a report.
///
/// # Errors
///
/// [`Error::NotImportable`] if the text is not an unencrypted Bitwarden JSON export — malformed
/// JSON, the wrong schema, and an *encrypted* export are deliberately one error, because the
/// user's next action is the same for all three: go back to Bitwarden and export again.
pub fn parse(json: &str) -> Result<Parsed> {
    let export: Export = serde_json::from_str(json).map_err(|_| Error::NotImportable)?;
    if export.encrypted {
        return Err(Error::NotImportable);
    }
    let Some(raw_items) = export.items else {
        return Err(Error::NotImportable);
    };

    let mut report = ImportReport::default();
    let mut folders: BTreeMap<String, String> = BTreeMap::new();
    for folder in export.folders {
        let name = folder.name.unwrap_or_default();
        unknown_keys(&mut report, &name, "folders[].", &folder.extra);
        if let Some(id) = folder.id {
            folders.insert(id, name);
        }
    }
    unknown_keys(&mut report, "", "", &export.extra);

    let mut items = Vec::with_capacity(raw_items.len());
    for raw in raw_items {
        if let Some(item) = map_item(raw, &folders, &mut report) {
            report.count(item.kind);
            items.push(item);
        }
    }

    Ok(Parsed { items, report })
}

/// Turns one exported item into one of ours, or into nothing but refusals.
///
/// Returns `None` only for an item that was in Bitwarden's trash at export time — the one case
/// where importing would resurrect something the user deleted.
fn map_item(
    raw: RawItem,
    folders: &BTreeMap<String, String>,
    report: &mut ImportReport,
) -> Option<Item> {
    let title = match raw.name {
        Some(name) if !name.trim().is_empty() => name,
        _ => UNTITLED.to_owned(),
    };

    if raw.deleted_date.is_some() {
        report.refuse(
            &title,
            "deletedDate",
            "the item was in Bitwarden's trash at export time, so it was not imported",
        );
        return None;
    }

    let mut extra = raw.extra;
    let mut item = match raw.r#type {
        Some(TYPE_LOGIN) => {
            let mut item = Item::new(ItemKind::Login, &title);
            map_login(&mut item, raw.login, &title, report);
            item
        }
        Some(TYPE_SECURE_NOTE) => Item::new(ItemKind::Note, &title),
        Some(TYPE_CARD) => {
            let mut item = Item::new(ItemKind::Card, &title);
            map_card(&mut item, raw.card, &title, report);
            item
        }
        Some(TYPE_IDENTITY) => {
            let mut item = Item::new(ItemKind::Identity, &title);
            map_identity(&mut item, raw.identity, &title, report);
            item
        }
        Some(TYPE_SSH_KEY) => {
            let mut item = Item::new(ItemKind::SshKey, &title);
            map_ssh_key(&mut item, raw.ssh_key, &title, report);
            item
        }
        // Bank account, driving licence, passport, and whatever Bitwarden adds next. There is
        // no type here to map them onto and inventing one is a format change, so the content
        // is kept as a note with its own keys as custom fields — named in `converted`, so the
        // user knows to look at it, rather than refused, which would lose the data outright.
        other => {
            let mut item = Item::new(ItemKind::Note, &title);
            map_unknown_type(&mut item, other, &mut extra, &title, report);
            item
        }
    };

    if let Some(notes) = raw.notes {
        // A secure note's text is the type's own field; on any other type it is something the
        // user attached, which is what `custom` means (D-43).
        let custom = item.kind != ItemKind::Note;
        push(
            &mut item,
            "Note",
            notes,
            FieldKind::Note,
            // Secret for the same reason the New-item dialog marks it secret: a secure note is
            // the surface people keep recovery codes on.
            true,
            custom,
        );
    }

    map_custom_fields(&mut item, raw.fields, &title, report);
    map_password_history(&mut item, raw.password_history, &title, report);

    item.set_favourite(raw.favorite);

    if let Some(folder_id) = raw.folder_id {
        match folders.get(&folder_id) {
            // Verbatim, so `Work/Clients` is one tag and not a hierarchy tags cannot express.
            Some(name) if !name.trim().is_empty() => item.set_tags([name.clone()]),
            _ => report.refuse(
                &title,
                "folderId",
                "the export names a folder it does not define, so the item was imported untagged",
            ),
        }
    }

    let timestamp = |label: &str, text: Option<String>, report: &mut ImportReport| {
        let text = text?;
        let parsed = iso8601_utc_ms(&text);
        if parsed.is_none() {
            report.refuse(
                &title,
                label.to_owned(),
                "the timestamp is not UTC ISO-8601, so the import date was used instead",
            );
        }
        parsed
    };
    let created = timestamp("creationDate", raw.creation_date, report);
    let revised = timestamp("revisionDate", raw.revision_date, report);
    if let Some(created) = created {
        item.created_at = created;
    }
    // An export whose revision predates its creation would sort the list wrongly rather than
    // fail, which is the kind of thing that gets blamed on the list.
    item.updated_at = revised.unwrap_or(item.created_at).max(item.created_at);

    if raw.reprompt.is_some_and(|reprompt| reprompt != 0) {
        report.refuse(
            &title,
            "reprompt",
            "asking for the master password again before showing an item has no equivalent in TrustVault",
        );
    }
    if raw.organization_id.is_some() {
        report.refuse(
            &title,
            "organizationId",
            "the item belongs to a Bitwarden organization; it was imported as a personal item",
        );
    }
    if raw.collection_ids.is_some_and(|ids| !ids.is_empty()) {
        report.refuse(
            &title,
            "collectionIds",
            "collections are a sharing concept and TrustVault is offline-only (D-03)",
        );
    }
    if raw.archived_date.is_some() {
        report.refuse(
            &title,
            "archivedDate",
            "TrustVault has no archive, so the item was imported as an ordinary one",
        );
    }
    for attachment in raw.attachments.unwrap_or_default() {
        report.refuse(
            &title,
            format!("attachments/{}", attachment.file_name.unwrap_or_default()),
            "attachments are not part of a JSON export and TrustVault v1 stores no files",
        );
        unknown_keys(report, &title, "attachments[].", &attachment.extra);
    }

    unknown_keys(report, &title, "", &extra);
    Some(item)
}

/// Login: the four fields our own dialog draws, then every extra URI as a custom field.
fn map_login(item: &mut Item, login: Option<RawLogin>, title: &str, report: &mut ImportReport) {
    let Some(login) = login else { return };

    set(item, "Username", login.username, FieldKind::Username, false);
    set(item, "Password", login.password, FieldKind::Password, true);
    // The failure every importer surveyed for D-42 makes is putting this in a note. It goes
    // in the field the TOTP generator reads, whether it is a bare base32 seed or an
    // `otpauth://` URI — both are what Bitwarden exports and both are what we accept.
    set(item, "2FA secret", login.totp, FieldKind::Otp, true);

    for (index, uri) in login.uris.unwrap_or_default().into_iter().enumerate() {
        match index {
            0 => set(item, "Website", uri.uri, FieldKind::Url, false),
            _ => push_optional(
                item,
                format!("Website {}", index + 1),
                uri.uri,
                FieldKind::Url,
                false,
                true,
            ),
        }
        if uri.r#match.is_some() {
            report.refuse(
                title,
                format!("login.uris[{index}].match"),
                "URI match detection belongs to browser auto-fill, which TrustVault does not do",
            );
        }
        unknown_keys(report, title, "login.uris[].", &uri.extra);
    }

    for (index, _) in login
        .fido2_credentials
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        report.refuse(
            title,
            format!("login.fido2Credentials[{index}]"),
            "passkeys are out of scope for v1 and cannot be stored",
        );
    }
    unknown_keys(report, title, "login.", &login.extra);
}

/// Card: expiry month and year are two keys there and one field here.
fn map_card(item: &mut Item, card: Option<RawCard>, title: &str, report: &mut ImportReport) {
    let Some(card) = card else { return };

    set(
        item,
        "Name on card",
        card.cardholder_name,
        FieldKind::Text,
        false,
    );
    set(item, "Card number", card.number, FieldKind::Text, true);
    set(item, "CVV", card.code, FieldKind::Text, true);

    let month = card.exp_month.map(|month| month.expose().to_owned());
    let year = card.exp_year.map(|year| year.expose().to_owned());
    if month.is_some() || year.is_some() {
        let month = month.unwrap_or_default();
        let year = year.unwrap_or_default();
        item.set_field("Expiry", format!("{month} / {year}"), false);
        report.convert(
            title,
            "card.expMonth, card.expYear",
            "joined into the single Expiry field TrustVault's card type has",
        );
    }

    // Not one of our card's four fields, and not something to throw away either.
    push_optional(item, "Brand", card.brand, FieldKind::Text, false, true);
    unknown_keys(report, title, "card.", &card.extra);
}

/// Identity: three of ours, fifteen custom fields, and one join.
fn map_identity(
    item: &mut Item,
    identity: Option<RawIdentity>,
    title: &str,
    report: &mut ImportReport,
) {
    let Some(identity) = identity else { return };

    let names: Vec<String> = [
        identity.first_name,
        identity.middle_name,
        identity.last_name,
    ]
    .into_iter()
    .flatten()
    .map(|part| part.expose().trim().to_owned())
    .filter(|part| !part.is_empty())
    .collect();
    if !names.is_empty() {
        item.set_field("Full name", names.join(" "), false);
        if names.len() > 1 {
            report.convert(
                title,
                "identity.firstName, identity.middleName, identity.lastName",
                "joined into the single Full name field TrustVault's identity type has",
            );
        }
    }

    set(item, "Email", identity.email, FieldKind::Email, false);
    set(item, "Phone", identity.phone, FieldKind::Text, false);

    // The rest have no field in our identity type, so they arrive as custom fields — which is
    // what D-43 built `custom` for. `secret` is decided here, in the mapping table, and is
    // still stored rather than inferred at read time (§6.3): a national insurance number, a
    // passport number and a licence number are masked; an address is not.
    for (label, value, secret) in [
        ("Title", identity.title, false),
        ("Company", identity.company, false),
        ("Username", identity.username, false),
        ("Address 1", identity.address1, false),
        ("Address 2", identity.address2, false),
        ("Address 3", identity.address3, false),
        ("City", identity.city, false),
        ("State", identity.state, false),
        ("Postal code", identity.postal_code, false),
        ("Country", identity.country, false),
        ("National ID", identity.ssn, true),
        ("Passport number", identity.passport_number, true),
        ("Licence number", identity.license_number, true),
    ] {
        push_optional(item, label, value, FieldKind::Text, secret, true);
    }
    unknown_keys(report, title, "identity.", &identity.extra);
}

/// SSH key: our type holds a host, a user and a passphrase, none of which Bitwarden exports.
fn map_ssh_key(item: &mut Item, key: Option<RawSshKey>, title: &str, report: &mut ImportReport) {
    let Some(key) = key else { return };

    let carried = key.private_key.is_some() || key.public_key.is_some();
    push_optional(
        item,
        "Private key",
        key.private_key,
        FieldKind::Text,
        true,
        true,
    );
    push_optional(
        item,
        "Public key",
        key.public_key,
        FieldKind::Text,
        false,
        true,
    );
    push_optional(
        item,
        "Fingerprint",
        key.key_fingerprint,
        FieldKind::Text,
        false,
        true,
    );
    if carried {
        report.convert(
            title,
            "sshKey.privateKey, sshKey.publicKey",
            "TrustVault's SSH key type holds a host, a user and a passphrase, so the key material was imported as custom fields",
        );
    }
    unknown_keys(report, title, "sshKey.", &key.extra);
}

/// An item of a type this build does not know, kept as a note rather than lost.
fn map_unknown_type(
    item: &mut Item,
    kind: Option<u8>,
    extra: &mut Map<String, Value>,
    title: &str,
    report: &mut ImportReport,
) {
    let named = match kind {
        Some(6) => "a bank account",
        Some(7) => "a driving licence",
        Some(8) => "a passport",
        _ => "a type this version of TrustVault does not know",
    };
    report.convert(
        title,
        "type",
        format!(
            "{named} in Bitwarden; imported as a secure note with its fields kept as custom fields"
        ),
    );

    // The type-specific object is the only object-valued key left once every key we know is
    // declared, so it can be found without knowing its name — which is the point, since the
    // next one will have a name nobody here has read yet.
    let Some(key) = extra
        .iter()
        .find(|(_, value)| value.is_object())
        .map(|(key, _)| key.clone())
    else {
        return;
    };
    let Some(Value::Object(payload)) = extra.remove(&key) else {
        return;
    };

    for (label, value) in payload {
        match value {
            Value::Null => {}
            Value::String(text) => push(
                item,
                &label,
                SecretString::from(text),
                FieldKind::Text,
                // Nothing is known about what this holds, and the two mistakes are not
                // symmetrical: masking a house number is a small annoyance, and leaving an
                // account number in the clear is the failure this product exists to prevent.
                true,
                true,
            ),
            Value::Bool(flag) => push(
                item,
                &label,
                SecretString::new(if flag { "true" } else { "false" }),
                FieldKind::Text,
                false,
                true,
            ),
            Value::Number(number) => push(
                item,
                &label,
                SecretString::new(number.to_string()),
                FieldKind::Text,
                false,
                true,
            ),
            _ => report.refuse(
                title,
                format!("{key}.{label}"),
                "the value is a list or a nested object, which a field cannot hold",
            ),
        }
    }
}

/// Bitwarden's own custom fields, which are the closest thing it has to D-43's concept.
fn map_custom_fields(
    item: &mut Item,
    fields: Option<Vec<RawField>>,
    title: &str,
    report: &mut ImportReport,
) {
    for (index, field) in fields.unwrap_or_default().into_iter().enumerate() {
        let label = match field.name {
            Some(name) if !name.trim().is_empty() => name,
            _ => format!("Field {}", index + 1),
        };
        // 0 text, 1 hidden, 2 boolean, 3 linked — `FieldType` in `bitwarden/clients`.
        match field.r#type {
            Some(3) | None if field.linked_id.is_some() => report.refuse(
                title,
                format!("fields/{label}"),
                "a linked field points at another item's field, which TrustVault has no way to express",
            ),
            Some(2) => {
                if let Some(value) = field.value {
                    let flag = value.expose() == "true";
                    push(
                        item,
                        &label,
                        SecretString::new(if flag { "true" } else { "false" }),
                        FieldKind::Text,
                        false,
                        true,
                    );
                    report.convert(
                        title,
                        format!("fields/{label}"),
                        "a yes/no field, stored as the text \"true\" or \"false\"",
                    );
                }
            }
            Some(3) => report.refuse(
                title,
                format!("fields/{label}"),
                "a linked field points at another item's field, which TrustVault has no way to express",
            ),
            // 1 is hidden, and everything else is treated as text — a type we do not
            // recognise is still a label and a value, and dropping it to be safe would be the
            // exact failure R-29 refuses to call an import.
            kind => push_optional(
                item,
                &label,
                field.value,
                FieldKind::Text,
                kind == Some(1),
                true,
            ),
        }
        unknown_keys(report, title, "fields[].", &field.extra);
    }
}

/// Previous passwords, which Bitwarden keeps per item and we keep per field.
fn map_password_history(
    item: &mut Item,
    history: Option<Vec<RawHistory>>,
    title: &str,
    report: &mut ImportReport,
) {
    let history = history.unwrap_or_default();
    if history.is_empty() {
        return;
    }

    let Some(field_id) = item
        .fields
        .iter()
        .find(|field| field.kind == FieldKind::Password)
        .map(|field| field.id)
    else {
        report.refuse(
            title,
            "passwordHistory",
            "the item has no password field for the old values to belong to",
        );
        return;
    };

    let mut carried = 0usize;
    for entry in history {
        let Some(value) = entry.password else {
            continue;
        };
        let changed_at = entry
            .last_used_date
            .as_deref()
            .and_then(iso8601_utc_ms)
            .unwrap_or(item.created_at);
        item.history.push(HistoryEntry {
            changed_at,
            field_id,
            value,
        });
        carried += 1;
        unknown_keys(report, title, "passwordHistory[].", &entry.extra);
    }
    if carried > 0 {
        report.convert(
            title,
            "passwordHistory",
            "Bitwarden keeps old passwords per item and TrustVault keeps them per field, so they were attached to the Password field",
        );
    }
}

/// Sets one of the item type's **own** fields, if the export carried a value for it.
fn set(item: &mut Item, label: &str, value: Option<SecretString>, kind: FieldKind, secret: bool) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    let id = item.set_field(label, value.expose(), secret);
    if let Some(field) = item.fields.iter_mut().find(|field| field.id == id) {
        field.kind = kind;
    }
}

/// Appends a custom field, if the export carried a value for it.
fn push_optional(
    item: &mut Item,
    label: impl AsRef<str>,
    value: Option<SecretString>,
    kind: FieldKind,
    secret: bool,
    custom: bool,
) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    push(item, label.as_ref(), value, kind, secret, custom);
}

/// Appends a field verbatim — never merging, because merging is how a duplicate label becomes
/// a field dropped in silence (D-43).
fn push(
    item: &mut Item,
    label: &str,
    value: SecretString,
    kind: FieldKind,
    secret: bool,
    custom: bool,
) {
    item.push_field(Field {
        id: uuid::Uuid::new_v4(),
        label: label.to_owned(),
        value,
        kind,
        secret,
        custom,
        unknown: BTreeMap::new(),
    });
}

/// Refuses every key this build does not know, so a schema that grows cannot grow past us.
///
/// A `null`, an empty list and an empty object are skipped: they carry nothing, so naming them
/// would fill the report with noise on every item and teach the reader to skim it.
fn unknown_keys(report: &mut ImportReport, title: &str, scope: &str, extra: &Map<String, Value>) {
    for (key, value) in extra {
        let empty = match value {
            Value::Null => true,
            Value::Array(items) => items.is_empty(),
            Value::Object(map) => map.is_empty(),
            Value::String(text) => text.is_empty(),
            _ => false,
        };
        if empty {
            continue;
        }
        report.refuse(
            title,
            format!("{scope}{key}"),
            "this version of TrustVault does not know that key, so its value has no home",
        );
    }
}

// The export's own shape. Every key Bitwarden writes is named here — see the module
// documentation for why, and for the five that are named and deliberately not imported.
//
// Value-bearing strings are `SecretString`, which is not decoration: they are zeroized when the
// parse structures are dropped, so the foreign vault's plaintext does not outlive the import in
// a heap nothing wipes. What `serde_json` leaves behind is the `Value` in an `extra` map, whose
// content is never read — only its key and its JSON type.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Export {
    #[serde(default)]
    encrypted: bool,
    #[serde(default)]
    folders: Vec<Folder>,
    items: Option<Vec<RawItem>>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
struct Folder {
    id: Option<String>,
    name: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    r#type: Option<u8>,
    name: Option<String>,
    notes: Option<SecretString>,
    #[serde(default)]
    favorite: bool,
    fields: Option<Vec<RawField>>,
    folder_id: Option<String>,
    organization_id: Option<String>,
    collection_ids: Option<Vec<String>>,
    reprompt: Option<u8>,
    creation_date: Option<String>,
    revision_date: Option<String>,
    deleted_date: Option<String>,
    archived_date: Option<String>,
    password_history: Option<Vec<RawHistory>>,
    attachments: Option<Vec<RawAttachment>>,
    login: Option<RawLogin>,
    card: Option<RawCard>,
    identity: Option<RawIdentity>,
    ssh_key: Option<RawSshKey>,
    // Declared so they do not reach `extra` and become refusals, and never read — the module
    // documentation says why for each. Removing any of them turns a key that carries no user
    // content into a refusal on every item, which is how a report stops being read.
    /// Always `{ "type": 0 }`; the text lives in the item's `notes`.
    #[allow(dead_code)]
    secure_note: Option<Value>,
    /// Bitwarden's identifier, not ours.
    #[allow(dead_code)]
    id: Option<String>,
    /// Per-cipher wrapping key; protects nothing in an unencrypted export.
    #[allow(dead_code)]
    key: Option<Value>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawField {
    name: Option<String>,
    value: Option<SecretString>,
    r#type: Option<u8>,
    linked_id: Option<Value>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLogin {
    username: Option<SecretString>,
    password: Option<SecretString>,
    totp: Option<SecretString>,
    uris: Option<Vec<RawUri>>,
    fido2_credentials: Option<Vec<Value>>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawUri {
    uri: Option<SecretString>,
    r#match: Option<Value>,
    /// A checksum of a value we imported in full.
    #[allow(dead_code)]
    uri_checksum: Option<Value>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCard {
    cardholder_name: Option<SecretString>,
    brand: Option<SecretString>,
    number: Option<SecretString>,
    exp_month: Option<SecretString>,
    exp_year: Option<SecretString>,
    code: Option<SecretString>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIdentity {
    title: Option<SecretString>,
    first_name: Option<SecretString>,
    middle_name: Option<SecretString>,
    last_name: Option<SecretString>,
    address1: Option<SecretString>,
    address2: Option<SecretString>,
    address3: Option<SecretString>,
    city: Option<SecretString>,
    state: Option<SecretString>,
    postal_code: Option<SecretString>,
    country: Option<SecretString>,
    company: Option<SecretString>,
    email: Option<SecretString>,
    phone: Option<SecretString>,
    ssn: Option<SecretString>,
    username: Option<SecretString>,
    passport_number: Option<SecretString>,
    license_number: Option<SecretString>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSshKey {
    private_key: Option<SecretString>,
    public_key: Option<SecretString>,
    key_fingerprint: Option<SecretString>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawHistory {
    last_used_date: Option<String>,
    password: Option<SecretString>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAttachment {
    file_name: Option<String>,
    // The file itself is not in a JSON export, so its identifier, size and URL describe
    // something this import could not have taken anyway. The refusal names the file name,
    // which is the only part the user can act on.
    #[allow(dead_code)]
    id: Option<String>,
    #[allow(dead_code)]
    size: Option<Value>,
    #[allow(dead_code)]
    size_name: Option<String>,
    #[allow(dead_code)]
    url: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wraps one item in the smallest export that parses.
    fn one_item(item: &str) -> Parsed {
        parse(&format!(r#"{{"encrypted":false,"items":[{item}]}}"#)).expect("valid export")
    }

    fn refused(parsed: &Parsed, field: &str) -> bool {
        parsed
            .report
            .refusals
            .iter()
            .any(|refusal| refusal.field == field)
    }

    #[test]
    fn an_archived_item_is_imported_and_the_archive_is_named() {
        // The item is not lost, because TrustVault having no archive is our gap and not the
        // user's instruction to delete anything.
        let parsed =
            one_item(r#"{"type":1,"name":"Archived","archivedDate":"2026-01-01T00:00:00.000Z"}"#);
        assert_eq!(parsed.items.len(), 1);
        assert!(refused(&parsed, "archivedDate"));
    }

    #[test]
    fn a_type_number_nobody_has_seen_still_imports() {
        // D-50's generic path, exercised on a type that does not exist yet — which is the only
        // honest way to test it, since the ones that do exist will look familiar to whoever
        // reads this next.
        let parsed = one_item(
            r#"{"type":97,"name":"Future","somethingNew":{"label":"text","count":42,
                "flag":true,"nothing":null,"nested":{"a":1}}}"#,
        );
        let item = parsed.items.first().expect("imported");
        assert_eq!(item.kind, ItemKind::Note);
        assert_eq!(
            item.fields.len(),
            3,
            "null is not a field, nested is refused"
        );
        assert!(item.fields.iter().any(|field| field.value.expose() == "42"));
        assert!(
            item.fields
                .iter()
                .any(|field| field.value.expose() == "true")
        );
        assert!(refused(&parsed, "somethingNew.nested"));
        assert!(
            parsed
                .report
                .converted
                .iter()
                .any(|converted| converted.field == "type")
        );
    }

    #[test]
    fn the_three_types_bitwarden_added_are_named_in_the_report() {
        for (number, expected) in [
            (6, "a bank account"),
            (7, "a driving licence"),
            (8, "a passport"),
        ] {
            let parsed = one_item(&format!(r#"{{"type":{number},"name":"X"}}"#));
            let note = &parsed.report.converted.first().expect("converted").note;
            assert!(note.contains(expected), "{number} reported as {note}");
        }
    }

    #[test]
    fn a_custom_field_with_no_name_still_arrives_somewhere_findable() {
        let parsed = one_item(
            r#"{"type":1,"name":"Nameless","fields":[{"name":null,"value":"kept","type":0}]}"#,
        );
        let item = parsed.items.first().expect("imported");
        assert_eq!(item.fields.len(), 1);
        assert_eq!(item.fields[0].label, "Field 1");
        assert_eq!(item.fields[0].value.expose(), "kept");
    }

    #[test]
    fn a_linked_field_is_refused_whether_or_not_it_names_its_target() {
        // Type 3 with no `linkedId` is what a partially-filled linked field exports as; both
        // shapes have to be refused, or one of them lands as an empty text field that looks
        // like the user cleared it.
        for field in [
            r#"{"name":"Linked","value":null,"type":3,"linkedId":100}"#,
            r#"{"name":"Linked","value":null,"type":3}"#,
        ] {
            let parsed = one_item(&format!(r#"{{"type":1,"name":"L","fields":[{field}]}}"#));
            assert!(refused(&parsed, "fields/Linked"), "{field}");
            assert!(parsed.items.first().expect("imported").fields.is_empty());
        }
    }

    #[test]
    fn password_history_with_nowhere_to_attach_is_refused_rather_than_invented() {
        let parsed = one_item(
            r#"{"type":2,"name":"Note with history","passwordHistory":[
                {"lastUsedDate":"2026-01-01T00:00:00.000Z","password":"old"}]}"#,
        );
        assert!(refused(&parsed, "passwordHistory"));
        assert!(parsed.items.first().expect("imported").history.is_empty());
    }

    #[test]
    fn a_history_entry_with_no_password_is_not_an_empty_one() {
        let parsed = one_item(
            r#"{"type":1,"name":"L","login":{"password":"current"},
                "passwordHistory":[{"lastUsedDate":null,"password":null}]}"#,
        );
        let item = parsed.items.first().expect("imported");
        assert!(item.history.is_empty());
        assert!(
            parsed.report.converted.is_empty(),
            "nothing was carried, so nothing is reported as carried"
        );
    }

    #[test]
    fn an_empty_unknown_key_is_not_reported_as_a_loss() {
        // A report full of noise is one nobody reads, which fails R-29 by a slower route
        // than dropping a field does.
        let parsed = one_item(
            r#"{"type":1,"name":"L","futureNull":null,"futureList":[],"futureMap":{},
                "futureText":"","futureReal":"kept nowhere"}"#,
        );
        assert_eq!(refusal_count(&parsed), 1);
        assert!(refused(&parsed, "futureReal"));
    }

    #[test]
    fn an_item_whose_type_object_is_missing_is_still_an_item() {
        // Exports in the wild carry `"type":1` with no `login` key when every login field was
        // empty. It is a titled item and the title is worth keeping.
        for item in [
            r#"{"type":1,"name":"Bare login"}"#,
            r#"{"type":3,"name":"Bare card"}"#,
            r#"{"type":4,"name":"Bare identity"}"#,
            r#"{"type":5,"name":"Bare key"}"#,
        ] {
            let parsed = one_item(item);
            let imported = parsed.items.first().expect("imported");
            assert!(imported.fields.is_empty());
            assert!(imported.title.starts_with("Bare"));
        }
    }

    fn refusal_count(parsed: &Parsed) -> usize {
        parsed.report.refusals.len()
    }
}
