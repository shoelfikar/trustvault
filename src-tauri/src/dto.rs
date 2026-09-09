//! The shapes that cross IPC, specified in `docs/ipc-contract.md` §6.1.
//!
//! Everything here is built by **eliding** a core type: taking the metadata and leaving the
//! plaintext behind. There is no `From<Item>` and there must not be one — a blanket conversion
//! is how "just send the whole item, it's simpler" gets implemented by accident.

use serde::{Deserialize, Serialize};
use trustvault_core::{
    Field, FieldEdit, FieldId, FieldKind, Item, ItemId, ItemKind, ItemStatus, SecretString,
};

/// The mask stood in for a secret value in a list — `docs/ipc-contract.md` §6.2, D-32.
///
/// **Fixed width, deliberately not the secret's length.** `"•".repeat(value.len())` would put
/// the exact length of every password in the vault into a heap that cannot be wiped, for every
/// row in the list, with no user action and nothing revealed. Twelve bullets is a lie the UI
/// must never present as a length.
pub const MASK: &str = "••••••••••••";

/// A field with its value elided if it is secret.
#[derive(Debug, Clone, Serialize)]
pub struct FieldSummary {
    /// Stable identifier, used to address this field in `reveal_field` and `copy_field`.
    pub id: FieldId,
    /// The label, which is metadata and not a secret.
    pub label: String,
    /// What the value is.
    pub kind: FieldKind,
    /// Whether the value is hidden by default.
    pub secret: bool,
    /// The real value — present only when `secret` is false.
    ///
    /// This is not a compromise. `secret` is stored, never inferred
    /// (`docs/vault-format.md` §6.3), so a field with `secret == false` is one the user
    /// declared is not a secret. The username on a login row is metadata the design draws in
    /// the list, and treating it as a secret would mean the list could not be rendered.
    pub value: Option<String>,
    /// The mask — present only when `secret` is true.
    pub mask: Option<&'static str>,
    /// Whether the user or an import added this field — D-43.
    ///
    /// Crosses because the detail pane renders custom fields as their own group below the
    /// type's own fields, and the webview cannot derive this: inferring it from the label is
    /// the mistake §6.3 already rejects for `secret`, one step along.
    pub custom: bool,
}

impl FieldSummary {
    /// Elides one field.
    fn elide(field: &Field) -> Self {
        Self {
            id: field.id,
            label: field.label.clone(),
            kind: field.kind,
            secret: field.secret,
            value: if field.secret {
                None
            } else {
                Some(field.value.expose().to_owned())
            },
            mask: if field.secret { Some(MASK) } else { None },
            custom: field.custom,
        }
    }
}

/// An item without any of its fields.
#[derive(Debug, Clone, Serialize)]
pub struct ItemSummary {
    /// Stable identifier.
    pub id: ItemId,
    /// Which of the seven types this is.
    pub kind: ItemKind,
    /// The title shown in the list.
    pub title: String,
    /// Free-form tags.
    pub tags: Vec<String>,
    /// The last Watchtower verdict — a cache, stale by definition until the next scan.
    pub status: ItemStatus,
    /// Whether the user pinned this item.
    pub favourite: bool,
    /// Creation time, Unix milliseconds UTC.
    pub created_at: i64,
    /// Last modification time, Unix milliseconds UTC.
    pub updated_at: i64,
}

impl ItemSummary {
    /// Elides one item down to what the list needs.
    pub fn elide(item: &Item) -> Self {
        Self {
            id: item.id,
            kind: item.kind,
            title: item.title.clone(),
            tags: item.tags.clone(),
            status: item.status,
            favourite: item.favourite,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

/// An item with its fields elided.
///
/// `history` is **absent, not elided**. It holds previous values of secret fields, so a user's
/// last five passwords for one site is a worse leak than any single one of them. A field that
/// is present and masked invites someone to add the reveal later; a field that is not in the
/// shape at all does not.
#[derive(Debug, Clone, Serialize)]
pub struct ItemDetail {
    /// Everything in the summary.
    #[serde(flatten)]
    pub summary: ItemSummary,
    /// The fields, in display order, secrets elided.
    pub fields: Vec<FieldSummary>,
}

impl ItemDetail {
    /// Elides one item and its fields.
    pub fn elide(item: &Item) -> Self {
        Self {
            summary: ItemSummary::elide(item),
            fields: item.fields.iter().map(FieldSummary::elide).collect(),
        }
    }
}

/// One field of an item being created — `docs/ipc-contract.md` §6.4.
///
/// The one shape in this file that carries plaintext **inbound**. §5 already concedes that
/// direction: a password the user typed is in the webview heap before any command is called,
/// and nothing this layer does can take it back out. What this layer can do is not make a
/// second copy — the `String` is *moved* into the core's `SecretString`, so the only plaintext
/// this process holds after the conversion is one that zeroizes on drop.
#[derive(Debug, Deserialize)]
pub struct NewField {
    /// The label.
    pub label: String,
    /// What the value is.
    pub kind: FieldKind,
    /// The value the user typed.
    pub value: String,
    /// Whether it is masked by default. Stored, never inferred (§6.3).
    pub secret: bool,
    /// Whether it is a user-added field rather than one of the type's own — D-43.
    pub custom: bool,
}

impl NewField {
    /// Builds the core field. Consumes `self` so the plaintext is moved, not cloned.
    pub fn into_field(self) -> Field {
        Field::new(self.label, self.value, self.secret)
            .with_kind(self.kind)
            .with_custom(self.custom)
    }
}

/// One field as an edit form submits it — `docs/ipc-contract.md` §6.4.
///
/// Two nullable fields that mean different things, and both are load-bearing: `id: null`
/// **creates**, and `value: null` means **unchanged**. The second is the one that destroys a
/// vault when it is got wrong — the form never received the secret values, so an edit that
/// treated `null` as "set it to nothing" would overwrite every password in the item with its
/// own mask on a rename.
#[derive(Debug, Deserialize)]
pub struct EditField {
    /// Which field, or `None` to create one.
    pub id: Option<FieldId>,
    /// The label, which may be renamed.
    pub label: String,
    /// What the value is.
    pub kind: FieldKind,
    /// The new value, or `None` to keep the stored one.
    pub value: Option<String>,
    /// Whether it is masked by default.
    pub secret: bool,
    /// Whether it is a user-added field — D-43.
    pub custom: bool,
}

impl EditField {
    /// Converts to the core's edit, or `None` if the request means nothing.
    ///
    /// The meaningless request is `id: null` with `value: null` — *create a field whose value
    /// is unchanged*. [`FieldEdit`] cannot express it, which is why it is two variants rather
    /// than one struct with two `Option`s, and this is where the wire shape's extra freedom is
    /// refused rather than given an invented meaning.
    pub fn into_edit(self) -> Option<FieldEdit> {
        match self.id {
            Some(id) => Some(FieldEdit::Existing {
                id,
                label: self.label,
                kind: self.kind,
                value: self.value.map(SecretString::new),
                secret: self.secret,
                custom: self.custom,
            }),
            None => self.value.map(|value| FieldEdit::New {
                label: self.label,
                kind: self.kind,
                value: SecretString::new(value),
                secret: self.secret,
                custom: self.custom,
            }),
        }
    }
}

/// What `add_item` returns: the identifier, and nothing else.
///
/// The caller re-reads through `get_item`, which keeps **one** elision path rather than two.
/// A response that carried the item back would be a second place where the decision "what may
/// cross" is made, and the two would drift.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AddedItem {
    /// The new item's identifier.
    pub item_id: ItemId,
}

/// Whether a vault is open, and what can be said about it if it is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultState {
    /// No vault has been chosen yet — a fresh install.
    NoVault,
    /// A vault file is known but not open.
    Locked,
    /// A vault is open and its body is in memory.
    Unlocked,
}

/// Who the vault belongs to, as a label — D-70.
///
/// **Not an account and not a credential.** Both strings are labels the user typed for their
/// own benefit; nothing authenticates against them and nothing is sent anywhere (D-03). They
/// are not `Secret` in R-10's sense for the same reason `display_name` is not — but they do
/// come out of the sealed body, which is why they are only knowable while unlocked.
#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    /// Full name. Empty until the user fills it in; onboarding does not ask.
    pub name: String,
    /// E-mail address. Empty by default, and never used to sign in.
    pub email: String,
}

/// The answer to "what is going on", which the whole frontend routes on.
#[derive(Debug, Clone, Serialize)]
pub struct VaultStatus {
    /// Open, closed, or nothing chosen.
    pub state: VaultState,
    /// Absolute path of the vault file, if one is known.
    pub path: Option<String>,
    /// What to call the vault on screen.
    ///
    /// **While locked this is the file stem, not the name the user typed at onboarding** —
    /// the real name lives inside the sealed body and cannot be read until the vault opens.
    /// The lock screen must not imply otherwise.
    pub display_name: String,
    /// How many items, or `None` while locked, because it cannot be known.
    pub item_count: Option<usize>,
    /// Who the vault belongs to, or `None` while locked — D-70.
    ///
    /// It rides on the status rather than having a `get_profile` of its own, and `item_count`
    /// one line up is the precedent: both are read out of the sealed body, both are `None`
    /// exactly when there is no key to read them with, and both are wanted by the first render
    /// after an unlock. A second command would be a second round trip for the same moment.
    ///
    /// `None` is the honest shape rather than an empty `Profile`, because "locked, so unknown"
    /// and "unlocked, and nobody has filled it in" are different facts and the footer draws
    /// them differently.
    pub profile: Option<Profile>,
    /// When the last local Watchtower scan finished, or `None` — §6.9.
    ///
    /// Two reasons for `None` and the surface must not merge them: the vault is **locked**, so
    /// the timestamp cannot be read out of the sealed body (`item_count`'s reason, D-70's
    /// precedent), or it is unlocked and **has never been scanned**. `state` is what tells them
    /// apart.
    ///
    /// It rides on the status rather than the scan returning it, so that the screen can say
    /// when the last scan ran **without running one** — a "last checked" that only appears after
    /// you check is not a last-checked.
    pub last_scan_at: Option<i64>,
    /// When the last breach check finished, or `None` — §6.9.
    ///
    /// Always `None` today: nothing writes it until `watchtower_breach_check` ships. It is here
    /// with its sibling because the two must never be collapsed into one — a local scan from
    /// this morning would otherwise vouch for a breach check that has never run.
    pub last_breach_check_at: Option<i64>,
}

/// Argon2id parameters, measured on this machine (R-02).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KdfSummary {
    /// Memory cost, KiB.
    pub m_cost: u32,
    /// Time cost, passes.
    pub t_cost: u32,
    /// Parallelism, lanes.
    pub p_cost: u8,
}

/// What `reveal_field` returns: one secret and when it stops being shown.
#[derive(Debug, Clone, Serialize)]
pub struct Revealed {
    /// The plaintext. The one `Secret` this response may carry (R-10).
    pub value: String,
    /// Wall-clock instant the core will emit `field-remasked`, Unix milliseconds UTC.
    pub remask_at: i64,
}

/// What `copy_field` returns: **no value**, only when the clipboard will be cleared.
///
/// The absence of a value field is the requirement (R-10). Rust owns the clipboard write, so
/// the secret never enters the webview at all.
#[derive(Debug, Clone, Serialize)]
pub struct Copied {
    /// Wall-clock instant the clipboard will be cleared, Unix milliseconds UTC.
    pub clears_at: i64,
}

/// One password field found in a breach corpus — `docs/ipc-contract.md` §6.9, R-25.
///
/// `count` is a property of the **corpus**, not of the password: it is how many times the value
/// appears in the records HIBP holds, which is what the row on screen shows. Narrowing a password
/// from it would need the range response, and that never leaves the host.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct BreachHit {
    /// The item the field belongs to.
    pub item_id: ItemId,
    /// Which password field — an item may carry more than one.
    pub field_id: FieldId,
    /// Occurrences in the breach corpus.
    pub count: u64,
}

/// One password field the check could not answer for — §6.9, R-25.
///
/// **This is the type that carries "not checked, never safe".** A field here has a status left
/// exactly as the local scan wrote it; nothing about it was proven either way, and no surface may
/// render its absence from [`BreachReport::breached`] as a pass.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct UncheckedField {
    /// The item the field belongs to.
    pub item_id: ItemId,
    /// Which password field.
    pub field_id: FieldId,
    /// `"off"`, `"offline"` or `"http"` — §6.9's vocabulary, and it has exactly two producers.
    ///
    /// A `&'static str` rather than an enum because the words are already defined once, in
    /// [`crate::hibp::RangeError::reason`], and the client cannot produce `"off"`: that one comes
    /// from the command that declined to call it at all. An enum here would be a second
    /// definition of the same three words, in the crate that has the first.
    pub reason: &'static str,
}

/// What `watchtower_breach_check` returns — §6.9, R-25, R-26.
///
/// Carries no prefix, no suffix, no hash and no password. `requested` is a **count** of range
/// requests, deliberately not a list of prefixes: a list would be a description of the vault's
/// password distribution in the one heap this whole architecture exists not to trust.
#[derive(Debug, Clone, Serialize)]
pub struct BreachReport {
    /// When the check finished, Unix milliseconds UTC.
    ///
    /// Not the same thing as the vault's `last_breach_check_at`, which only a **complete** pass
    /// writes (D-86). This field says when this call ran; that one says when the vault last had a
    /// check it can still stand behind after a relaunch.
    pub checked_at: i64,
    /// Range requests actually made — one per distinct value, never one per item (S-07b).
    ///
    /// Zero when the setting is off, and zero is the number S-10's packet capture predicts.
    pub requested: usize,
    /// Every password field whose value was found in the corpus.
    pub breached: Vec<BreachHit>,
    /// Every password field that was not checked, and why.
    pub unchecked: Vec<UncheckedField>,
}

/// Build and format information, for the About surface and bug reports.
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    /// Application version, from `Cargo.toml`.
    pub version: &'static str,
    /// On-disk vault format version this build reads and writes.
    pub format_version: u16,
    /// Vault file extension.
    pub extension: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use trustvault_core::{ItemKind, KdfParams, Vault};

    fn item_with_both_kinds_of_field() -> Item {
        let (mut vault, _) = Vault::create("Personal", "correct horse", KdfParams::TESTING)
            .expect("test parameters are valid");
        let id = vault.add_item(ItemKind::Login, "GitHub");
        let item = vault.item_mut(id).expect("just added");
        item.set_field("Username", "octocat", false);
        item.set_field("Password", "hunter2", true);
        item.clone()
    }

    #[test]
    fn a_secret_field_crosses_as_a_mask_and_a_public_one_as_its_value() {
        let detail = ItemDetail::elide(&item_with_both_kinds_of_field());

        let username = &detail.fields[0];
        assert_eq!(username.value.as_deref(), Some("octocat"));
        assert_eq!(username.mask, None);

        let password = &detail.fields[1];
        assert_eq!(password.value, None);
        assert_eq!(password.mask, Some(MASK));
    }

    #[test]
    fn the_mask_is_a_fixed_width_and_not_the_secret_length() {
        // D-32. The failing version of this is `"•".repeat(value.len())`, which looks correct
        // and leaks the length of every password in the vault.
        let (mut vault, _) = Vault::create("Personal", "correct horse", KdfParams::TESTING)
            .expect("test parameters are valid");
        let id = vault.add_item(ItemKind::Login, "GitHub");
        let item = vault.item_mut(id).expect("just added");
        item.set_field("Short", "a", true);
        item.set_field("Long", "a-very-long-passphrase-indeed", true);

        let detail = ItemDetail::elide(vault.item(id).expect("just added"));
        assert_eq!(detail.fields[0].mask, detail.fields[1].mask);
        assert_eq!(
            detail.fields[0].mask.map(str::chars).map(Iterator::count),
            Some(12)
        );
    }

    #[test]
    fn no_serialized_shape_carries_a_secret_or_a_history() {
        let item = item_with_both_kinds_of_field();

        let detail = serde_json::to_string(&ItemDetail::elide(&item)).expect("serializable");
        assert!(!detail.contains("hunter2"), "the secret must not cross");
        assert!(!detail.contains("history"), "history is absent, not elided");

        let summary = serde_json::to_string(&ItemSummary::elide(&item)).expect("serializable");
        assert!(!summary.contains("hunter2"));
        assert!(
            !summary.contains("octocat"),
            "the list carries no field values"
        );
    }
}
