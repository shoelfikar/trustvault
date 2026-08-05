//! The shapes that cross IPC, specified in `docs/ipc-contract.md` §6.1.
//!
//! Everything here is built by **eliding** a core type: taking the metadata and leaving the
//! plaintext behind. There is no `From<Item>` and there must not be one — a blanket conversion
//! is how "just send the whole item, it's simpler" gets implemented by accident.

use serde::{Deserialize, Serialize};
use trustvault_core::{Field, FieldId, FieldKind, Item, ItemId, ItemKind, ItemStatus};

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
