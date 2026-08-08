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
//! * **`custom` is stored, never inferred either** (§6.5, D-43), for exactly that reason one
//!   step along: deriving it from "is this label in the type's standard set?" means a login
//!   whose password field was renamed becomes a custom field, and an imported custom field
//!   that happens to be called "Username" becomes the standard one.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use ciborium::Value;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::secret::SecretString;
use crate::{Error, Result};

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
    /// Whether the user (or an import) added this field, rather than it being one of the
    /// type's own fields. Stored, never inferred — §6.5, D-43.
    ///
    /// `skip_serializing_if` is not an optimization, for the same reason it is not one on
    /// [`VaultBody::audit`]: a vault with no custom field anywhere serializes to exactly the
    /// bytes it did before this key existed, which is what keeps the known-answer vectors in
    /// `tests/vectors/` valid without regenerating them.
    #[serde(default, skip_serializing_if = "is_false")]
    pub custom: bool,
    /// Keys written by a newer version, preserved untouched (N-09).
    #[serde(flatten)]
    pub unknown: Unknown,
}

/// Predicate for `skip_serializing_if` on a `bool` that defaults to false.
///
/// Takes a reference because that is the signature serde requires.
fn is_false(value: &bool) -> bool {
    !*value
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
            custom: false,
            unknown: Unknown::new(),
        }
    }

    /// Sets the field kind, replacing the one `new` guessed from `secret`.
    #[must_use]
    pub fn with_kind(mut self, kind: FieldKind) -> Self {
        self.kind = kind;
        self
    }

    /// Marks the field as user-added rather than one of the item type's own — D-43.
    #[must_use]
    pub fn with_custom(mut self, custom: bool) -> Self {
        self.custom = custom;
        self
    }
}

/// One field as an edit form submits it — `docs/ipc-contract.md` §6.4.
///
/// Two variants rather than one struct carrying two `Option`s, and the reason is the shape a
/// struct would also allow: *create a field whose value is unchanged*, which means nothing. A
/// shape that can express a meaningless request is a shape something eventually sends, and the
/// receiving code then has to invent an answer. Here the type refuses it instead.
///
/// **`value: None` on [`FieldEdit::Existing`] means unchanged, and it is the single most
/// load-bearing detail in the edit path.** The form never received the secret values — they are
/// elided on the way out — so a form that sent back what it is holding would be sending back
/// masks. Without this variant, renaming an item would overwrite every password in it with
/// `"••••••••••••"` and push the real values into [`Item::history`], where no v1 surface can
/// reach them.
#[derive(Debug)]
#[non_exhaustive]
pub enum FieldEdit {
    /// Edit the field with this identifier, keeping its place in the item's history.
    Existing {
        /// Which field. Unknown identifiers are refused rather than created — the caller is
        /// working from a list it was just given, so an unknown one is a stale form.
        id: FieldId,
        /// The label, which may be renamed.
        label: String,
        /// What the value is.
        kind: FieldKind,
        /// The new value, or `None` to keep the stored one.
        value: Option<SecretString>,
        /// Whether the value is masked by default.
        secret: bool,
        /// Whether this is a user- or import-added field — D-43.
        custom: bool,
    },
    /// Create a field. It is appended where the edit list puts it, not at the end.
    New {
        /// The label.
        label: String,
        /// What the value is.
        kind: FieldKind,
        /// The value. Not optional: a field being created has nothing to leave unchanged.
        value: SecretString,
        /// Whether the value is masked by default.
        secret: bool,
        /// Whether this is a user- or import-added field — D-43.
        custom: bool,
    },
}

impl FieldEdit {
    /// The field this edit addresses, if it addresses one.
    fn target(&self) -> Option<FieldId> {
        match self {
            Self::Existing { id, .. } => Some(*id),
            Self::New { .. } => None,
        }
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

    /// Sets one of the item type's **own** fields by label, creating it if it does not exist.
    ///
    /// Overwriting an existing value pushes the old one onto [`Item::history`] — losing a
    /// password to a mistyped edit is a support case that cannot be undone otherwise.
    ///
    /// The search skips custom fields (D-43), and that is load-bearing rather than tidy: a
    /// Bitwarden export may carry a custom field called "Username" beside the login's own
    /// username, and without the filter, importing one would overwrite the other and push a
    /// real credential into `history` where no UI in v1 can reach it. Custom fields arrive
    /// through [`Item::push_field`] and are never addressed by label.
    pub fn set_field(&mut self, label: &str, value: impl Into<String>, secret: bool) -> FieldId {
        let value = SecretString::new(value);
        let now = now_ms();

        if let Some(field) = self
            .fields
            .iter_mut()
            .find(|field| !field.custom && field.label == label)
        {
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

    /// Appends a field verbatim, with no lookup and no deduplication — D-43.
    ///
    /// This is the importer's path. It never merges, because merging is how an importer
    /// drops a field in silence: Bitwarden permits two custom fields with the same name, and
    /// a set-by-label that found the first would discard the second while reporting success.
    /// R-29 is met only when every field is either mapped or **named in a refusal**, and a
    /// collapsed duplicate is neither.
    pub fn push_field(&mut self, field: Field) -> FieldId {
        let id = field.id;
        self.fields.push(field);
        self.updated_at = now_ms();
        id
    }

    /// The fields the user or an import added, in display order — D-43.
    ///
    /// Separated because the design draws no custom-field editor: the detail pane renders
    /// these below the type's own fields, and an item with none looks exactly as it did
    /// before this concept existed.
    pub fn custom_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(|field| field.custom)
    }

    /// Renames the item, bumping `updated_at` only if the title actually changed.
    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if self.title != title {
            self.title = title;
            self.updated_at = now_ms();
        }
    }

    /// Replaces the tags, trimmed and deduplicated, in the order given.
    ///
    /// Deduplication is exact rather than case-folded, and the whole string is kept: D-43 makes
    /// a Bitwarden folder path a tag verbatim, so `Work/Clients` is one tag and not a hierarchy
    /// this model pretends to understand.
    pub fn set_tags(&mut self, tags: impl IntoIterator<Item = String>) {
        let mut cleaned: Vec<String> = Vec::new();
        for tag in tags {
            let tag = tag.trim();
            if !tag.is_empty() && !cleaned.iter().any(|kept| kept == tag) {
                cleaned.push(tag.to_owned());
            }
        }
        if self.tags != cleaned {
            self.tags = cleaned;
            self.updated_at = now_ms();
        }
    }

    /// Pins or unpins the item.
    pub fn set_favourite(&mut self, favourite: bool) {
        if self.favourite != favourite {
            self.favourite = favourite;
            self.updated_at = now_ms();
        }
    }

    /// Rewrites the item's fields from an edit form — `docs/ipc-contract.md` §6.4.
    ///
    /// Three rules, each of which is silent when it goes wrong:
    ///
    /// * **`value: None` keeps the stored value.** See [`FieldEdit`]; this is the rule that
    ///   stops a rename from overwriting every password with its own mask.
    /// * **Omission deletes.** A field the list does not mention is removed. The alternative —
    ///   a separate remove command — makes an edit two round trips that can half-succeed.
    /// * **The list is the order.** Fields are written in the order given, so reordering needs
    ///   no command of its own.
    ///
    /// A changed value pushes the previous one onto [`Item::history`]. A *deleted* field does
    /// not, and the asymmetry is deliberate rather than an oversight: an edit that overwrites
    /// is usually a mistake worth recovering from, and a deletion is an explicit instruction to
    /// stop holding that value.
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchField`] if an edit names a field this item does not have, or names one
    /// twice. Validated **before** anything is written, so a rejected edit leaves the item
    /// exactly as it was — a half-applied edit is the worst outcome available here.
    pub fn apply_edits(&mut self, edits: Vec<FieldEdit>) -> Result<()> {
        let mut addressed: Vec<FieldId> = Vec::with_capacity(edits.len());
        for id in edits.iter().filter_map(FieldEdit::target) {
            if !self.fields.iter().any(|field| field.id == id) || addressed.contains(&id) {
                return Err(Error::NoSuchField);
            }
            addressed.push(id);
        }

        let now = now_ms();
        let order_before: Vec<FieldId> = self.fields.iter().map(|field| field.id).collect();
        let mut pool = core::mem::take(&mut self.fields);
        let mut rebuilt: Vec<Field> = Vec::with_capacity(edits.len());
        let mut changed = false;

        for edit in edits {
            match edit {
                FieldEdit::Existing {
                    id,
                    label,
                    kind,
                    value,
                    secret,
                    custom,
                } => {
                    // The validation pass above proved this position exists, so the lookup
                    // cannot fail — but it is written as a search rather than an index because
                    // `pool` shrinks as fields are taken out of it.
                    let Some(position) = pool.iter().position(|field| field.id == id) else {
                        continue;
                    };
                    let mut field = pool.remove(position);
                    if let Some(value) = value
                        && field.value != value
                    {
                        self.history.push(HistoryEntry {
                            changed_at: now,
                            field_id: field.id,
                            value: core::mem::replace(&mut field.value, value),
                        });
                        changed = true;
                    }
                    changed |= field.label != label
                        || field.kind != kind
                        || field.secret != secret
                        || field.custom != custom;
                    field.label = label;
                    field.kind = kind;
                    field.secret = secret;
                    field.custom = custom;
                    rebuilt.push(field);
                }
                FieldEdit::New {
                    label,
                    kind,
                    value,
                    secret,
                    custom,
                } => {
                    // Constructed rather than built through `Field::new`, so the value is
                    // moved into place instead of copied out of one `SecretString` and into
                    // another — the discarded copy would be zeroized, but it would exist.
                    rebuilt.push(Field {
                        id: Uuid::new_v4(),
                        label,
                        value,
                        kind,
                        secret,
                        custom,
                        unknown: Unknown::new(),
                    });
                    changed = true;
                }
            }
        }

        // Fields left in the pool were omitted, which deletes them. Dropping `pool` zeroizes
        // their values.
        changed |= !pool.is_empty();
        changed |= rebuilt
            .iter()
            .map(|field| field.id)
            .ne(order_before.iter().copied());

        self.fields = rebuilt;
        if changed {
            self.updated_at = now;
        }
        Ok(())
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

/// Who the vault belongs to, as a label — `docs/vault-format.md` §6.6, D-70.
///
/// **Not an account.** D-03 put sync out of scope and there is no server to authenticate
/// against, so these two strings are never sent anywhere, never checked, and never used to
/// unlock: they label the sidebar footer, the Settings card and the printed recovery kit, and
/// that is their whole job. Storing them was the alternative to inventing them on screen.
///
/// It lives **inside the sealed body** rather than beside the settings (D-33), and the
/// difference is the point: a name and an e-mail address identify a person, and the file whose
/// whole purpose is that its contents are unreadable without a key is the right place for them.
/// The cost is that they cannot be read while locked, which is why the lock screen names the
/// vault and not its owner.
///
/// Both strings may be empty, which is the state every vault starts in — onboarding does not
/// ask (the design's three steps are vault name, master password, recovery kit) and a profile
/// nobody filled in must serialize to nothing at all. See [`Profile::is_empty`].
/// `Eq` is absent for the reason it is absent on [`Field`]: [`Unknown`] holds a
/// `ciborium::Value`, which can contain a float. Preserving a newer version's data (N-09) is
/// worth more than a total equality relation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Profile {
    /// Full name, as the user typed it. The avatar is its initials, computed for display.
    #[serde(default)]
    pub name: String,
    /// E-mail address. **Never used to sign in** — there is nothing to sign in to.
    #[serde(default)]
    pub email: String,
    /// Keys written by a newer version, preserved untouched (N-09).
    #[serde(flatten)]
    pub unknown: Unknown,
}

impl Profile {
    /// Whether nothing has been filled in, which is what stops the key from being written.
    ///
    /// `unknown` counts: a profile that holds only a key this build does not recognize is not
    /// empty, and skipping it would drop the newer version's data — N-09's whole point.
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.email.is_empty() && self.unknown.is_empty()
    }
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
    /// Who the vault belongs to, as a label — §6.6, D-70.
    ///
    /// `skip_serializing_if` for the reason [`VaultBody::audit`] has one, and it matters more
    /// here because every existing vault has an empty profile: an untouched one writes **no key
    /// at all**, so a vault that predates this field encodes to exactly the bytes it did before
    /// — which is what keeps the known-answer vectors in `tests/vectors/` valid.
    #[serde(default, skip_serializing_if = "Profile::is_empty")]
    pub profile: Profile,
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
            profile: Profile::default(),
            audit: Vec::new(),
            unknown: Unknown::new(),
        }
    }

    /// Sets the profile, reporting whether anything actually changed — §6.6, D-70.
    ///
    /// Both strings are trimmed, because the one thing this data is for is being *displayed*
    /// and a trailing space in a name is invisible in every place it renders. The return value
    /// is what lets the caller skip a save: this is reached from a dialog whose Save button is
    /// pressed whether or not the fields were touched, and a write per press would rewrite the
    /// whole vault file to store the string it already held.
    ///
    /// `unknown` is left alone: keys a newer version wrote are not this build's to clear.
    pub(crate) fn set_profile(&mut self, name: &str, email: &str) -> bool {
        let name = name.trim();
        let email = email.trim();
        if self.profile.name == name && self.profile.email == email {
            return false;
        }
        self.profile.name = name.to_owned();
        self.profile.email = email.to_owned();
        self.touch();
        true
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
    fn custom_is_stored_not_inferred() {
        // §6.5, D-43. The label says nothing here either — a field called "Username" can be
        // the login's own or one an import brought along, and only the flag distinguishes
        // them.
        let own = Field::new("Username", "octocat", false);
        let brought = Field::new("Username", "octocat@work", false).with_custom(true);
        assert!(!own.custom);
        assert!(brought.custom);
    }

    #[test]
    fn setting_a_field_never_reaches_a_custom_one() {
        // The import collision D-43 exists to prevent: a custom "Username" beside the
        // login's own. Without the filter, `set_field` would find whichever came first,
        // overwrite it, and push a real credential into `history` where no v1 UI can reach.
        let mut item = Item::new(ItemKind::Login, "GitHub");
        let brought =
            item.push_field(Field::new("Username", "from-import", false).with_custom(true));
        let own = item.set_field("Username", "octocat", false);

        assert_ne!(
            own, brought,
            "a new field was created, not the custom one reused"
        );
        assert_eq!(item.fields.len(), 2);
        assert!(item.history.is_empty(), "nothing was overwritten");
        assert_eq!(
            item.field(brought).unwrap().value,
            SecretString::new("from-import")
        );
        assert_eq!(item.custom_fields().count(), 1);
    }

    #[test]
    fn pushed_fields_keep_duplicate_labels() {
        // Bitwarden permits two custom fields with the same name. Merging them would be a
        // field dropped in silence, which is exactly what R-29 refuses to call an import.
        let mut item = Item::new(ItemKind::Login, "GitHub");
        let first = item.push_field(Field::new("Note", "one", false).with_custom(true));
        let second = item.push_field(Field::new("Note", "two", false).with_custom(true));

        assert_ne!(first, second);
        assert_eq!(item.custom_fields().count(), 2);
    }

    #[test]
    fn an_ordinary_field_serialises_without_the_custom_key() {
        // Not cosmetic: the known-answer vectors in tests/vectors/ were generated before this
        // key existed, and they stay valid only because a vault with no custom field anywhere
        // encodes to the same bytes it did then.
        let field = Field::new("Username", "octocat", false);
        let encoded = serde_json::to_string(&field).unwrap();
        assert!(
            !encoded.contains("custom"),
            "a non-custom field must write no `custom` key: {encoded}"
        );

        let custom = field.clone().with_custom(true);
        assert!(
            serde_json::to_string(&custom)
                .unwrap()
                .contains(r#""custom":true"#)
        );
    }

    #[test]
    fn a_field_written_before_custom_existed_reads_as_not_custom() {
        let json = format!(
            r#"{{"id":"{}","label":"Username","value":"octocat","kind":"username","secret":false}}"#,
            Uuid::new_v4()
        );
        let field: Field = serde_json::from_str(&json).unwrap();
        assert!(!field.custom);
        assert!(
            field.unknown.is_empty(),
            "`custom` is a known key, not one that falls through to `unknown`"
        );
    }

    #[test]
    fn field_kind_can_be_set_explicitly() {
        let field = Field::new("Site", "https://example.com", false).with_kind(FieldKind::Url);
        assert_eq!(field.kind, FieldKind::Url);
        // `new` only guesses when nothing better is said.
        assert_eq!(Field::new("Password", "x", true).kind, FieldKind::Password);
    }

    /// A login with a public username and a secret password, in that order.
    fn a_login() -> (Item, FieldId, FieldId) {
        let mut item = Item::new(ItemKind::Login, "GitHub");
        let username = item.set_field("Username", "octocat", false);
        let password = item.set_field("Password", "correct-horse", true);
        (item, username, password)
    }

    /// The edit an unchanged field submits: everything as it was, `value: None`.
    fn unchanged(field: &Field) -> FieldEdit {
        FieldEdit::Existing {
            id: field.id,
            label: field.label.clone(),
            kind: field.kind,
            value: None,
            secret: field.secret,
            custom: field.custom,
        }
    }

    #[test]
    fn a_none_value_keeps_the_stored_one() {
        // The single most load-bearing rule in the edit path — §6.4. The form never held the
        // secrets, so an edit that renames the item submits `None` for every value. If that
        // meant "set it to nothing", renaming an item would destroy every password in it and
        // push the real values into `history`, where no v1 surface can reach them.
        let (mut item, _, password) = a_login();
        let edits: Vec<FieldEdit> = item.fields.iter().map(unchanged).collect();

        item.set_title("GitHub (work)");
        item.apply_edits(edits).unwrap();

        assert_eq!(
            item.field(password).unwrap().value,
            SecretString::new("correct-horse")
        );
        assert!(
            item.history.is_empty(),
            "an unchanged field is not a history entry"
        );
    }

    #[test]
    fn a_changed_value_pushes_the_previous_onto_history() {
        let (mut item, username, password) = a_login();
        let edits = vec![
            unchanged(item.field(username).unwrap()),
            FieldEdit::Existing {
                id: password,
                label: "Password".to_owned(),
                kind: FieldKind::Password,
                value: Some(SecretString::new("new-horse")),
                secret: true,
                custom: false,
            },
        ];
        item.apply_edits(edits).unwrap();

        assert_eq!(
            item.field(password).unwrap().value,
            SecretString::new("new-horse")
        );
        assert_eq!(item.history.len(), 1);
        assert_eq!(item.history[0].field_id, password);
        assert_eq!(item.history[0].value, SecretString::new("correct-horse"));
    }

    #[test]
    fn submitting_the_same_value_again_is_not_history() {
        let (mut item, username, password) = a_login();
        let edits = vec![
            unchanged(item.field(username).unwrap()),
            FieldEdit::Existing {
                id: password,
                label: "Password".to_owned(),
                kind: FieldKind::Password,
                value: Some(SecretString::new("correct-horse")),
                secret: true,
                custom: false,
            },
        ];
        item.apply_edits(edits).unwrap();
        assert!(item.history.is_empty());
    }

    #[test]
    fn omission_deletes_and_does_not_reach_history() {
        // The asymmetry is deliberate: an overwrite is usually a mistake worth recovering
        // from, a deletion is an instruction to stop holding the value.
        let (mut item, username, password) = a_login();
        let edits = vec![unchanged(item.field(username).unwrap())];
        item.apply_edits(edits).unwrap();

        assert!(item.field(password).is_none());
        assert!(item.field(username).is_some());
        assert!(item.history.is_empty());
    }

    #[test]
    fn the_edit_list_is_the_display_order() {
        // Reordering therefore needs no command of its own — §6.4.
        let (mut item, username, password) = a_login();
        let edits = vec![
            unchanged(item.field(password).unwrap()),
            unchanged(item.field(username).unwrap()),
        ];
        item.apply_edits(edits).unwrap();

        let order: Vec<FieldId> = item.fields.iter().map(|field| field.id).collect();
        assert_eq!(order, vec![password, username]);
    }

    #[test]
    fn a_new_edit_creates_a_field_where_the_list_puts_it() {
        let (mut item, username, password) = a_login();
        let edits = vec![
            unchanged(item.field(username).unwrap()),
            FieldEdit::New {
                label: "Recovery email".to_owned(),
                kind: FieldKind::Text,
                value: SecretString::new("octocat@example.com"),
                secret: false,
                custom: true,
            },
            unchanged(item.field(password).unwrap()),
        ];
        item.apply_edits(edits).unwrap();

        assert_eq!(item.fields.len(), 3);
        assert_eq!(item.fields[1].label, "Recovery email");
        assert!(item.fields[1].custom);
        assert_eq!(item.custom_fields().count(), 1);
        assert!(item.history.is_empty());
    }

    #[test]
    fn an_edit_naming_a_field_the_item_does_not_have_changes_nothing() {
        // Validated before anything is written: a half-applied edit is the worst outcome
        // available here, because the caller cannot tell which half landed.
        let (mut item, username, _) = a_login();
        let before = item.fields.clone();
        let edits = vec![
            FieldEdit::Existing {
                id: username,
                label: "Handle".to_owned(),
                kind: FieldKind::Username,
                value: Some(SecretString::new("someone-else")),
                secret: false,
                custom: false,
            },
            FieldEdit::Existing {
                id: Uuid::new_v4(),
                label: "Ghost".to_owned(),
                kind: FieldKind::Text,
                value: None,
                secret: false,
                custom: false,
            },
        ];

        assert!(matches!(item.apply_edits(edits), Err(Error::NoSuchField)));
        assert_eq!(item.fields, before);
        assert!(item.history.is_empty());
    }

    #[test]
    fn an_edit_naming_one_field_twice_is_refused() {
        // Two edits for one field cannot both be applied, and choosing one silently is how a
        // stale form overwrites what the user is looking at.
        let (mut item, username, _) = a_login();
        let before = item.fields.clone();
        let edits = vec![
            unchanged(item.field(username).unwrap()),
            unchanged(item.field(username).unwrap()),
        ];

        assert!(matches!(item.apply_edits(edits), Err(Error::NoSuchField)));
        assert_eq!(item.fields, before);
    }

    #[test]
    fn an_edit_that_changes_nothing_leaves_updated_at_alone() {
        // The list is sorted by `updated_at`, so a no-op edit that bumped it would reorder
        // the item list every time a dialog was opened and cancelled.
        let (mut item, _, _) = a_login();
        let edits: Vec<FieldEdit> = item.fields.iter().map(unchanged).collect();
        item.updated_at = 0;
        item.apply_edits(edits).unwrap();
        assert_eq!(item.updated_at, 0);
    }

    #[test]
    fn a_rename_is_a_change_and_a_re_titling_to_the_same_string_is_not() {
        let (mut item, _, _) = a_login();
        item.updated_at = 0;
        item.set_title("GitHub");
        item.set_favourite(false);
        assert_eq!(item.updated_at, 0, "nothing actually changed");

        item.set_title("GitLab");
        assert_ne!(item.updated_at, 0);
        assert_eq!(item.title, "GitLab");

        item.updated_at = 0;
        item.set_favourite(true);
        assert_ne!(item.updated_at, 0);
        assert!(item.favourite);
    }

    #[test]
    fn an_untouched_profile_writes_no_key_at_all() {
        // Not cosmetic, and the same argument as `custom` and `audit`: every vault that exists
        // today has an empty profile, and the known-answer vectors in tests/vectors/ stay valid
        // only because such a vault encodes to the bytes it did before this field existed.
        let body = VaultBody::new("Personal Vault");
        let encoded = serde_json::to_string(&body).unwrap();
        assert!(
            !encoded.contains("profile"),
            "an empty profile must write no `profile` key: {encoded}"
        );
    }

    #[test]
    fn a_body_written_before_profile_existed_reads_as_empty() {
        let json = format!(
            r#"{{"vault_id":"{}","name":"Personal Vault","created_at":0,"updated_at":0,"items":[]}}"#,
            Uuid::new_v4()
        );
        let body: VaultBody = serde_json::from_str(&json).unwrap();
        assert!(body.profile.is_empty());
        assert!(
            body.unknown.is_empty(),
            "`profile` is a known key, not one that falls through to `unknown`"
        );
    }

    #[test]
    fn setting_a_profile_trims_and_reports_whether_it_changed() {
        let mut body = VaultBody::new("Personal Vault");
        body.updated_at = 0;

        assert!(body.set_profile("  Budi Santoso  ", " budi@warungpintar.id "));
        assert_eq!(body.profile.name, "Budi Santoso");
        assert_eq!(body.profile.email, "budi@warungpintar.id");
        assert_ne!(body.updated_at, 0, "a real change is a change");

        body.updated_at = 0;
        assert!(
            !body.set_profile("Budi Santoso", "budi@warungpintar.id"),
            "the dialog's Save is pressed whether or not anything was typed"
        );
        assert_eq!(body.updated_at, 0, "and that must not rewrite the file");
    }

    #[test]
    fn a_profile_holding_only_an_unknown_key_is_not_empty() {
        // N-09 read the other way round: skipping it on write would drop what a newer version
        // stored, which is the silent corruption the whole `unknown` mechanism exists to stop.
        let mut profile = Profile::default();
        assert!(profile.is_empty());
        profile
            .unknown
            .insert("avatar_colour".to_owned(), Value::Text("brass".to_owned()));
        assert!(!profile.is_empty());
    }

    #[test]
    fn tags_are_trimmed_deduplicated_and_kept_verbatim() {
        // Verbatim is the D-43 half: a Bitwarden folder path becomes one tag, because
        // splitting `Work/Clients` in two would claim a hierarchy tags do not have.
        let (mut item, _, _) = a_login();
        item.set_tags([
            "  Work/Clients  ".to_owned(),
            "personal".to_owned(),
            "Work/Clients".to_owned(),
            "   ".to_owned(),
        ]);
        assert_eq!(item.tags, vec!["Work/Clients", "personal"]);

        item.updated_at = 0;
        item.set_tags(["Work/Clients".to_owned(), "personal".to_owned()]);
        assert_eq!(item.updated_at, 0, "the same tags are not a change");
    }
}
