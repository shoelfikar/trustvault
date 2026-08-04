//! The item model — the plaintext that lives inside the sealed body.
//!
//! Specified in `docs/vault-format.md` §6. Two properties of this module are load-bearing and
//! neither is obvious from the struct definitions:
//!
//! * **Unknown fields survive a round trip** (N-09). Every structure carries an `unknown` map
//!   holding keys this build does not recognize, and writes them back untouched. A vault
//!   edited by a newer version and then saved by an older one must not lose data silently.
//! * **`secret` is stored, never inferred** (§6.3). Guessing from the label masks
//!   "Recovery e-mail" because it contains "recovery", and leaves "PIN" in the clear because
//!   nobody thought of it.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use ciborium::Value;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::secret::SecretString;

/// Identifier of an item, stable for its lifetime.
pub type ItemId = Uuid;
/// Identifier of a field, stable across edits so the UI can address one field of one item.
pub type FieldId = Uuid;

/// Unknown keys preserved verbatim across a load/save cycle (N-09).
///
/// `BTreeMap` rather than `HashMap` so the order is deterministic: a vault re-saved without
/// edits must produce the same bytes, or the known-answer vectors mean nothing.
pub type Unknown = BTreeMap<String, Value>;

/// Milliseconds since the Unix epoch, UTC.
///
/// A plain integer rather than a date type: the vault format should not depend on a calendar
/// library's serialization, and formatting for display is the UI's problem.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// The kinds of item a vault can hold.
///
/// Fixed at seven by the design; adding an eighth is a format change and therefore a decision
/// log entry, not a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemKind {
    /// Username and password for a site or service.
    Login,
    /// An API key or token.
    ApiKey,
    /// A payment card.
    Card,
    /// Free-form encrypted text.
    Note,
    /// A Wi-Fi network and its passphrase.
    ///
    /// Renamed explicitly: `snake_case` would derive `wi_fi` from the variant name, and the
    /// on-disk string is `wifi` as specified in `docs/vault-format.md` §6.1.
    #[serde(rename = "wifi")]
    WiFi,
    /// An SSH key and its passphrase.
    SshKey,
    /// Identity documents.
    Identity,
}

impl ItemKind {
    /// Every variant, in the order the design's "New item" dialog presents them.
    pub const ALL: [ItemKind; 7] = [
        ItemKind::Login,
        ItemKind::ApiKey,
        ItemKind::Card,
        ItemKind::Note,
        ItemKind::WiFi,
        ItemKind::SshKey,
        ItemKind::Identity,
    ];
}

/// What the last Watchtower scan concluded about an item.
///
/// **A cache, not a truth.** It is stored so the item list can draw its status pips without
/// re-scanning at every open, and it is stale by definition until the next scan. Nothing may
/// make a security decision from it. The vocabulary is the design's
/// (`design-system/password-manager/MASTER.md`), where every status is rendered as an icon
/// *and* a word — never colour alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemStatus {
    /// Never scanned. The state every item starts in.
    #[default]
    Unknown,
    /// Scanned and nothing to report.
    Strong,
    /// zxcvbn scored it low.
    Weak,
    /// The same password appears on another item in this vault.
    Reused,
    /// Found in a breach corpus.
    Breached,
    /// Past its expiry date.
    Expired,
}

/// What a field holds, which is how the UI decides to render it.
///
/// Distinct from [`Field::secret`]: `kind` says what the value *is*, `secret` says whether it
/// must be masked. A URL is not secret; a note may well be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FieldKind {
    /// A single line of text.
    #[default]
    Text,
    /// A username or account identifier.
    Username,
    /// A password or passphrase.
    Password,
    /// A URL.
    Url,
    /// An e-mail address.
    Email,
    /// A TOTP secret, in the form the RFC 6238 generator consumes.
    Otp,
    /// Multi-line free text.
    Note,
    /// An ISO-8601 date.
    Date,
}

/// One field of one item.
///
/// `Eq` is absent on purpose: [`Unknown`] holds `ciborium::Value`, which can contain a float.
/// Preserving a newer version's data (N-09) is worth more than a total equality relation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// Stable identifier, so the UI can address this field across edits.
    pub id: FieldId,
    /// The label shown next to the value.
    pub label: String,
    /// The value. Always wrapped, secret or not — see [`SecretString`].
    pub value: SecretString,
    /// What the value is.
    #[serde(default)]
    pub kind: FieldKind,
    /// Whether the value is masked by default. Stored, never inferred (§6.3).
    pub secret: bool,
    /// Keys written by a newer version, preserved untouched (N-09).
    #[serde(flatten)]
    pub unknown: Unknown,
}

impl Field {
    /// A new field with a fresh identifier.
    pub fn new(label: impl Into<String>, value: impl Into<String>, secret: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            value: SecretString::new(value),
            kind: if secret {
                FieldKind::Password
            } else {
                FieldKind::Text
            },
            secret,
            unknown: Unknown::new(),
        }
    }

    /// Sets the field kind, replacing the one `new` guessed from `secret`.
    #[must_use]
    pub fn with_kind(mut self, kind: FieldKind) -> Self {
        self.kind = kind;
        self
    }
}

/// A previous value of a field, kept so an accidental overwrite is recoverable.
///
/// As sensitive as the field itself, and therefore inside the sealed body like everything
/// else. It is not an audit log: R-13's reveal log is a separate, later thing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// When the change happened, Unix milliseconds UTC.
    pub changed_at: i64,
    /// Which field changed.
    pub field_id: FieldId,
    /// The value it held before.
    pub value: SecretString,
}

/// One entry in the vault.
///
/// Not `Eq`, for the reason given on [`Field`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Stable identifier.
    pub id: ItemId,
    /// Which of the seven types this is.
    pub kind: ItemKind,
    /// The title shown in the list.
    pub title: String,
    /// The fields, in display order.
    pub fields: Vec<Field>,
    /// Free-form tags.
    pub tags: Vec<String>,
    /// The last Watchtower verdict. A cache — see [`ItemStatus`].
    #[serde(default)]
    pub status: ItemStatus,
    /// Whether the user pinned this item.
    #[serde(default)]
    pub favourite: bool,
    /// Creation time, Unix milliseconds UTC.
    pub created_at: i64,
    /// Last modification time, Unix milliseconds UTC.
    pub updated_at: i64,
    /// Previous values of fields that were overwritten.
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    /// Keys written by a newer version, preserved untouched (N-09).
    #[serde(flatten)]
    pub unknown: Unknown,
}

impl Item {
    /// A new, empty item of the given kind.
    pub fn new(kind: ItemKind, title: impl Into<String>) -> Self {
        let now = now_ms();
        Self {
            id: Uuid::new_v4(),
            kind,
            title: title.into(),
            fields: Vec::new(),
            tags: Vec::new(),
            status: ItemStatus::Unknown,
            favourite: false,
            created_at: now,
            updated_at: now,
            history: Vec::new(),
            unknown: Unknown::new(),
        }
    }

    /// Finds a field by identifier.
    pub fn field(&self, id: FieldId) -> Option<&Field> {
        self.fields.iter().find(|field| field.id == id)
    }

    /// Sets a field by label, creating it if it does not exist.
    ///
    /// Overwriting an existing value pushes the old one onto [`Item::history`] — losing a
    /// password to a mistyped edit is a support case that cannot be undone otherwise.
    pub fn set_field(&mut self, label: &str, value: impl Into<String>, secret: bool) -> FieldId {
        let value = SecretString::new(value);
        let now = now_ms();

        if let Some(field) = self.fields.iter_mut().find(|field| field.label == label) {
            if field.value != value {
                self.history.push(HistoryEntry {
                    changed_at: now,
                    field_id: field.id,
                    value: core::mem::take(&mut field.value),
                });
                field.value = value;
            }
            field.secret = secret;
            self.updated_at = now;
            return field.id;
        }

        let field = Field::new(label, value.expose(), secret);
        let id = field.id;
        self.fields.push(field);
        self.updated_at = now;
        id
    }

    /// Removes a field, returning whether one was there.
    pub fn remove_field(&mut self, id: FieldId) -> bool {
        let before = self.fields.len();
        self.fields.retain(|field| field.id != id);
        let removed = self.fields.len() != before;
        if removed {
            self.updated_at = now_ms();
        }
        removed
    }
}

/// One reveal, recorded when the audit setting is on (R-13, D-31).
///
/// **Holds no secret and must never be given one.** Not the value, not the field label, not
/// the item title — an audit log that quotes what it audited is a second copy of the vault
/// with none of the ceremony. The two identifiers are enough to name the reveal, and both are
/// meaningless without the vault they came from.
///
/// It lives inside the sealed body for the same reason [`HistoryEntry`] does: a plaintext
/// record of *which* secret was read *when* is sensitive on its own, whatever it omits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the reveal happened, Unix milliseconds UTC.
    pub at: i64,
    /// Which item was revealed.
    pub item_id: ItemId,
    /// Which field of it.
    pub field_id: FieldId,
}

/// The decrypted contents of a vault.
///
/// This is the plaintext that the body seal protects. It never leaves the core as a whole:
/// the IPC boundary hands out elided lists and, on explicit user action, one secret at a time
/// (R-10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultBody {
    /// Identifier of the vault itself, stable across renames and moves.
    pub vault_id: Uuid,
    /// Display name. The filename is not authoritative.
    pub name: String,
    /// Creation time, Unix milliseconds UTC.
    pub created_at: i64,
    /// Last save time, Unix milliseconds UTC.
    pub updated_at: i64,
    /// The items.
    pub items: Vec<Item>,
    /// Reveals recorded while the audit setting was on, oldest first (R-13, D-31).
    ///
    /// `skip_serializing_if` is not an optimization. An empty log writes **no key at all**,
    /// so a vault that has never recorded a reveal serializes to exactly the bytes it did
    /// before this field existed — which is what keeps the known-answer vectors in
    /// `tests/vectors/` valid without regenerating them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit: Vec<AuditEntry>,
    /// Keys written by a newer version, preserved untouched (N-09).
    #[serde(flatten)]
    pub unknown: Unknown,
}

impl VaultBody {
    /// The most reveals kept. Oldest are dropped first once the log is longer — D-31.
    ///
    /// A count cap and no age cap, which bounds the file but not the history: for a light
    /// user 1000 entries may reach back years. That trade was made deliberately and is
    /// written down in the decision log rather than left to be discovered here.
    pub const AUDIT_MAX_ENTRIES: usize = 1000;

    /// An empty vault body.
    pub fn new(name: impl Into<String>) -> Self {
        let now = now_ms();
        Self {
            vault_id: Uuid::new_v4(),
            name: name.into(),
            created_at: now,
            updated_at: now,
            items: Vec::new(),
            audit: Vec::new(),
            unknown: Unknown::new(),
        }
    }

    /// Appends a reveal to the audit log and enforces the cap.
    ///
    /// Deliberately does **not** call [`Self::touch`]: a reveal reads the vault, it does not
    /// modify it, and moving `updated_at` would make every "last changed" display in the UI
    /// mean "last looked at" instead.
    pub(crate) fn record_reveal(&mut self, item_id: ItemId, field_id: FieldId) {
        self.audit.push(AuditEntry {
            at: now_ms(),
            item_id,
            field_id,
        });
        if self.audit.len() > Self::AUDIT_MAX_ENTRIES {
            let excess = self.audit.len() - Self::AUDIT_MAX_ENTRIES;
            self.audit.drain(..excess);
        }
    }

    /// Marks the body as modified now.
    pub(crate) fn touch(&mut self) {
        self.updated_at = now_ms();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_item_kinds_are_listed() {
        // Guards against adding a variant to ItemKind and forgetting ALL, which would make
        // the new type invisible to the New-item dialog while still being storable.
        assert_eq!(ItemKind::ALL.len(), 7);
        for kind in ItemKind::ALL {
            assert!(ItemKind::ALL.contains(&kind));
        }
    }

    #[test]
    fn discriminants_serialise_to_stable_snake_case() {
        // These strings go into the vault file, so they are part of the format and are quoted
        // verbatim in docs/vault-format.md §6.1.
        let cases = [
            (ItemKind::Login, r#""login""#),
            (ItemKind::ApiKey, r#""api_key""#),
            (ItemKind::Card, r#""card""#),
            (ItemKind::Note, r#""note""#),
            (ItemKind::WiFi, r#""wifi""#),
            (ItemKind::SshKey, r#""ssh_key""#),
            (ItemKind::Identity, r#""identity""#),
        ];
        for (kind, expected) in cases {
            assert_eq!(serde_json::to_string(&kind).unwrap(), expected);
        }

        assert_eq!(
            serde_json::to_string(&ItemStatus::Breached).unwrap(),
            r#""breached""#
        );
        assert_eq!(
            serde_json::to_string(&FieldKind::Password).unwrap(),
            r#""password""#
        );
    }

    #[test]
    fn a_new_item_starts_unscanned() {
        let item = Item::new(ItemKind::Login, "GitHub");
        assert_eq!(item.status, ItemStatus::Unknown);
        assert!(!item.favourite);
        assert_eq!(item.created_at, item.updated_at);
    }

    #[test]
    fn overwriting_a_field_keeps_the_previous_value() {
        let mut item = Item::new(ItemKind::Login, "GitHub");
        let id = item.set_field("Password", "old", true);
        assert!(item.history.is_empty());

        let same_id = item.set_field("Password", "new", true);
        assert_eq!(id, same_id, "a field keeps its identity across edits");
        assert_eq!(item.fields.len(), 1);
        assert_eq!(item.history.len(), 1);
        assert_eq!(item.history[0].field_id, id);
        assert_eq!(item.history[0].value, SecretString::new("old"));
        assert_eq!(item.field(id).unwrap().value, SecretString::new("new"));
    }

    #[test]
    fn setting_a_field_to_its_current_value_is_not_history() {
        let mut item = Item::new(ItemKind::Login, "GitHub");
        item.set_field("Password", "same", true);
        item.set_field("Password", "same", true);
        assert!(item.history.is_empty());
    }

    #[test]
    fn secret_is_stored_not_inferred() {
        // §6.3. The label says nothing; the flag says everything.
        let mut item = Item::new(ItemKind::Note, "Notes");
        let public = item.set_field("Password hint", "my first pet", false);
        let private = item.set_field("Colour", "amber", true);
        assert!(!item.field(public).unwrap().secret);
        assert!(item.field(private).unwrap().secret);
    }

    #[test]
    fn fields_can_be_removed() {
        let mut item = Item::new(ItemKind::Login, "GitHub");
        let id = item.set_field("Username", "octocat", false);
        assert!(item.remove_field(id));
        assert!(!item.remove_field(id));
        assert!(item.field(id).is_none());
    }

    #[test]
    fn field_kind_can_be_set_explicitly() {
        let field = Field::new("Site", "https://example.com", false).with_kind(FieldKind::Url);
        assert_eq!(field.kind, FieldKind::Url);
        // `new` only guesses when nothing better is said.
        assert_eq!(Field::new("Password", "x", true).kind, FieldKind::Password);
    }
}
