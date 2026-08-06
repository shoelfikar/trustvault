//! The reference vault every timing in `trustvault-requirements.md` is measured against —
//! **1 000 items, 4 fields each**.
//!
//! The requirements have named "`trustvault-core`'s `benchfixture` helper" since kickoff and
//! four criteria depend on it (S-02 unlock, S-04 palette, S-07 Watchtower scan, R-11 scroll),
//! but nothing implemented it until S-04 needed a number. Written now, in the crate the
//! requirements said it lives in, so that the four measurements are taken against one vault
//! rather than four vaults each benchmark invented for itself.
//!
//! Behind a feature so it is absent from the shipped library. A fixture generator compiled into
//! the application is a way to create a thousand fake items in a real vault.
//!
//! # No random number generator, deliberately
//!
//! Every value here is derived from the item's index. That is not a style preference: the CI
//! job behind R-06 fails the build if anything in this crate mentions a seedable RNG, and the
//! way to satisfy both that rule and a reproducible fixture is to have nothing to seed. A
//! hand-rolled generator written to slip past the grep would be exactly what the rule forbids,
//! with the evidence removed.

use crate::kdf::KdfParams;
use crate::model::{Field, FieldKind, ItemKind};
use crate::vault::Vault;
use crate::{RecoveryCode, Result};

/// How many items the reference vault holds.
pub const ITEMS: usize = 1_000;

/// How many fields each of them carries.
pub const FIELDS_PER_ITEM: usize = 4;

/// The master password of every fixture vault.
pub const PASSWORD: &str = "correct horse battery staple";

/// Service names the titles are drawn from, cycled and numbered.
///
/// Real-looking rather than `item-0001`: a fuzzy search over a thousand identical titles
/// measures the wrong thing, because every haystack would match at the same position and the
/// ranking would never be exercised.
const SERVICES: [&str; 25] = [
    "GitHub",
    "GitLab",
    "Digital Ocean",
    "Chase Sapphire",
    "Fastmail",
    "Cloudflare",
    "Hetzner",
    "Linode",
    "Notion",
    "Figma",
    "Slack",
    "Sentry",
    "Stripe",
    "Vercel",
    "Netlify",
    "Backblaze",
    "Proton Mail",
    "Tailscale",
    "Grafana",
    "Postmark",
    "Twilio",
    "Cloudsmith",
    "JetBrains",
    "Framer",
    "Zoom",
];

/// Tags the items are spread across.
const TAGS: [&str; 8] = [
    "Work",
    "Personal",
    "Finance",
    "Infrastructure",
    "Archive",
    "Family",
    "Clients/Acme",
    "Clients/Globex",
];

/// A vault of [`ITEMS`] items with [`FIELDS_PER_ITEM`] fields each.
///
/// The parameters are the caller's, because the two things measured against this vault want
/// opposite ones: S-04 measures matching and wants [`KdfParams::TESTING`] so the fixture builds
/// in milliseconds, while S-02 measures the unlock itself and must use the real defaults.
///
/// # Errors
///
/// Whatever [`Vault::create`] returns — in practice only parameters out of range.
pub fn reference_vault(params: KdfParams) -> Result<(Vault, RecoveryCode)> {
    let (mut vault, recovery) = Vault::create("Reference", PASSWORD, params)?;

    for index in 0..ITEMS {
        // `get` rather than `[]` throughout: this crate warns on indexing and CI denies
        // warnings, which is the Tier-1 rule doing its job in a file that only builds fixtures.
        let service = SERVICES
            .get(index % SERVICES.len())
            .copied()
            .unwrap_or("Service");
        let kind = ItemKind::ALL
            .get(index % ItemKind::ALL.len())
            .copied()
            .unwrap_or(ItemKind::Login);
        let id = vault.add_item(kind, format!("{service} {}", index / SERVICES.len()));
        let Some(item) = vault.item_mut(id) else {
            // Unreachable: added one line above. Skipped rather than unwrapped, because this
            // crate denies the panicking constructs and a fixture is not the place to spend
            // the exception.
            continue;
        };

        let tag = |slot: usize| {
            TAGS.get(slot % TAGS.len())
                .copied()
                .unwrap_or("Untagged")
                .to_owned()
        };
        item.set_tags([tag(index), tag(index / 3)]);

        let handle = format!("user{index}");
        let host = service.to_lowercase().replace(' ', "");
        item.push_field(Field::new("Username", &handle, false).with_kind(FieldKind::Username));
        // The only secret in the fixture, and the reason it is one: the palette benchmark is
        // worth nothing if the thing it walks past is not a real secret field.
        item.push_field(Field::new("Password", format!("pw-{index}-xK9"), true));
        item.push_field(
            Field::new("Website", format!("https://{host}.example/login"), false)
                .with_kind(FieldKind::Url),
        );
        item.push_field(
            Field::new("Email", format!("{handle}@example.test"), false)
                .with_kind(FieldKind::Email),
        );
    }

    Ok((vault, recovery))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture is the shape the requirements describe, and the same one every time.
    #[test]
    fn the_reference_vault_is_a_thousand_items_of_four_fields() {
        let (one, _) = reference_vault(KdfParams::TESTING).expect("testing parameters are valid");
        assert_eq!(one.items().count(), ITEMS);
        assert!(
            one.items().all(|item| item.fields.len() == FIELDS_PER_ITEM),
            "every item carries {FIELDS_PER_ITEM} fields"
        );

        let (two, _) = reference_vault(KdfParams::TESTING).expect("testing parameters are valid");
        let titles = |vault: &Vault| {
            vault
                .items()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            titles(&one),
            titles(&two),
            "the fixture is derived from the index, so two builds are the same vault"
        );
    }
}
