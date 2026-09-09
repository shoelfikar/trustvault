//! A vault built to have something for Watchtower to find — R-23, R-24.
//!
//! `benchfixture` is the wrong vault for this and the reason is in its own source: every item
//! gets `pw-{index}-xK9`, so **no two items share a password** and every password is the same
//! unmatched shape. R-23 has nothing to group and R-24 has one score repeated a thousand times.
//! It is a timing floor, which is exactly what it was written to be, and it stays that — four
//! criteria are measured against it and changing it would move them (D-77).
//!
//! This one is the opposite trade. It is small enough that every number below was worked out by
//! hand, and it is built out of **named passwords with known zxcvbn scores** rather than derived
//! ones, because the thing being fixtured is the scoring.
//!
//! # The scores are measured, not asserted
//!
//! Every score in [`ENTRIES`] was read out of zxcvbn on 2026-08-15 rather than guessed, and
//! `the_table_is_what_zxcvbn_says_today` re-reads all of them on every test run. That test is
//! the point of the table: zxcvbn is a dictionary and a set of matchers, both of which change
//! between releases, and a fixture that claims a spread across five scores while quietly
//! holding three is a fixture that makes R-24's tests pass by having nothing in them. When the
//! dependency moves, this fails loudly and names the row.
//!
//! # One item is a canary for the context argument
//!
//! *Northwind Mail* carries `priya.raman.2024` and its username is `priya.raman`. That password
//! scores **4 with no context and 2 with the item's own words**, so it only counts as weak
//! because [`crate::scan`] passes the title and username through to zxcvbn. If that argument is
//! ever dropped — the D-12 amendment undone — this fixture's weak count falls by one and the
//! test says so. Nothing else in the project would notice.
//!
//! Behind the same feature as `benchfixture` and for the same reason: a fixture generator
//! compiled into the application is a way to put twenty-one fake items into somebody's real
//! vault. The feature is named for the first fixture that needed it, not for the only one.
//!
//! # No random number generator, deliberately
//!
//! `benchfixture`'s rule holds here for the same reason — the CI job behind R-06 fails the build
//! if anything in this crate mentions a seedable RNG. Nothing here is random: it is a written
//! table.

use crate::kdf::KdfParams;
use crate::model::{Field, FieldKind, ItemKind};
use crate::vault::Vault;
use crate::{RecoveryCode, Result};

/// The master password of every fixture vault — the same published constant `benchfixture`
/// uses, re-exported rather than re-declared so there is one fixture password in the project.
pub use crate::benchfixture::PASSWORD;

/// One item in the fixture: what it is called, who it belongs to, and what it stores.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// The item's title.
    pub title: &'static str,
    /// The username field's value — and, for one entry, the reason its password is weak.
    pub username: &'static str,
    /// The password field's value. Repeated across entries wherever a reuse group is wanted.
    pub password: &'static str,
    /// What zxcvbn scores `password` at **as [`crate::scan`] scores it**, with this entry's own
    /// title and username as context.
    pub score: u8,
}

/// The fixture, one row per item, in the order the vault is built.
///
/// Read it as five reuse groups and seven singletons:
///
/// | Password | Score | Items | What it exercises |
/// |---|---|---|---|
/// | `password` | 0 | 3 | the loudest case: weak **and** reused, three ways |
/// | `Tr0ub4dour&3` | 2 | 2 | the [`crate::WEAK_MAX_SCORE`] boundary itself — D-79's row |
/// | `mangotree84` | 3 | 5 | **reused without being weak**, the case a length rule gets wrong |
/// | `correct horse battery staple` | 4 | 2 | reused at the top score — reuse is not a strength verdict |
/// | `brisk-otter-vault` | 4 | 1 item, **2 fields** | same value twice inside one item: untidy, not reused |
/// | `hunter2`, `priya.raman.2024` | 1, 2 | 1 each | weak without being reused |
/// | four uniques, `opal38thicket` | 4, 3 | 1 each | clean rows, so a finding count can be wrong in both directions |
///
/// *Vault Archive Note* carries no password field at all, which is the row that proves a scan
/// of a vault holding notes does not count them.
pub const ENTRIES: [Entry; 21] = [
    // The score-0 group of three.
    Entry {
        title: "Aurora Ledger",
        username: "casey.nwosu",
        password: "password",
        score: 0,
    },
    Entry {
        title: "Harbour Freight Portal",
        username: "c.nwosu",
        password: "password",
        score: 0,
    },
    Entry {
        title: "Tidewater Utilities",
        username: "casey",
        password: "password",
        score: 0,
    },
    // Weak, alone.
    Entry {
        title: "Old Forum Account",
        username: "casey1994",
        password: "hunter2",
        score: 1,
    },
    // The boundary group: score 2, which is weak here and *Fair* on the strength meter. The
    // disagreement is D-79's, and it is an open question rather than a bug in either surface.
    Entry {
        title: "Meridian Bank",
        username: "casey.nwosu",
        password: "Tr0ub4dour&3",
        score: 2,
    },
    Entry {
        title: "Meridian Bank — joint",
        username: "casey.nwosu",
        password: "Tr0ub4dour&3",
        score: 2,
    },
    // Reused five ways without being weak. Every length-and-class rule ever written passes
    // this password, and it is on five accounts.
    Entry {
        title: "Sandhill Hosting",
        username: "ops@sandhill.test",
        password: "mangotree84",
        score: 3,
    },
    Entry {
        title: "Sandhill Status Page",
        username: "ops@sandhill.test",
        password: "mangotree84",
        score: 3,
    },
    Entry {
        title: "Sandhill Billing",
        username: "billing@sandhill.test",
        password: "mangotree84",
        score: 3,
    },
    Entry {
        title: "Sandhill DNS",
        username: "ops@sandhill.test",
        password: "mangotree84",
        score: 3,
    },
    Entry {
        title: "Sandhill Backups",
        username: "ops@sandhill.test",
        password: "mangotree84",
        score: 3,
    },
    // Reused at the top score. A strong password on two accounts is still one breach from two
    // accounts, which is the whole of R-23 in one row.
    Entry {
        title: "Kestrel Router",
        username: "admin",
        password: "correct horse battery staple",
        score: 4,
    },
    Entry {
        title: "Kestrel Guest Wi-Fi",
        username: "guest",
        password: "correct horse battery staple",
        score: 4,
    },
    // The context canary — 4 without the item's own words, 2 with them.
    Entry {
        title: "Northwind Mail",
        username: "priya.raman",
        password: "priya.raman.2024",
        score: 2,
    },
    // Clean rows.
    Entry {
        title: "Cloud Vendor Console",
        username: "casey.nwosu",
        password: "zM8!qvRt2Ldx6Bn",
        score: 4,
    },
    Entry {
        title: "Payroll",
        username: "casey.nwosu",
        password: "quiet-lantern-drift-71",
        score: 4,
    },
    Entry {
        title: "Registrar",
        username: "casey.nwosu",
        password: "Rt4$wubneKla",
        score: 4,
    },
    Entry {
        title: "Analytics",
        username: "casey.nwosu",
        password: "vN3^rukmapheL",
        score: 4,
    },
    Entry {
        title: "Local NAS",
        username: "casey",
        password: "opal38thicket",
        score: 3,
    },
    // Two password fields carrying one value. `scan` counts reuse across items, so this is two
    // password fields, one distinct value, and no finding.
    Entry {
        title: "Workshop Server",
        username: "root",
        password: "brisk-otter-vault",
        score: 4,
    },
    // No password at all. The `password` here is never written — see `audit_vault`.
    Entry {
        title: "Vault Archive Note",
        username: "",
        password: "",
        score: 0,
    },
];

/// The title of the entry whose password is stored in **two** fields.
const TWO_PASSWORD_FIELDS: &str = "Workshop Server";

/// The title of the entry that carries no password field.
const NO_PASSWORD: &str = "Vault Archive Note";

/// How many items the fixture holds.
pub const ITEMS: usize = ENTRIES.len();

/// How many password fields a scan of it examines — one per entry, plus the second field on
/// [`TWO_PASSWORD_FIELDS`], minus the entry that has none.
pub const PASSWORD_FIELDS: usize = 21;

/// How many distinct values are among them — what a breach check of this vault would cost.
pub const DISTINCT: usize = 12;

/// How many `weak` findings a scan produces: three at score 0, one at 1, two at 2, and the
/// context canary.
pub const WEAK_FINDINGS: usize = 7;

/// How many `reused` findings: every member of every group of two or more, which is
/// 3 + 2 + 5 + 2.
pub const REUSED_FINDINGS: usize = 12;

/// How many items carry at least one finding, and so appear in [`crate::Report::worst`].
pub const FLAGGED_ITEMS: usize = 14;

/// A vault with reuse groups of known size and a password at every zxcvbn score.
///
/// The parameters are the caller's for [`crate::benchfixture::reference_vault`]'s reason: a test
/// wants [`KdfParams::TESTING`] and anything that writes a file a person opens wants the real
/// defaults.
///
/// # Errors
///
/// Whatever [`Vault::create`] returns — in practice only parameters out of range.
pub fn audit_vault(params: KdfParams) -> Result<(Vault, RecoveryCode)> {
    let (mut vault, recovery) = Vault::create("Audit", PASSWORD, params)?;

    for entry in ENTRIES {
        let kind = if entry.title == NO_PASSWORD {
            ItemKind::Note
        } else {
            ItemKind::Login
        };
        let id = vault.add_item(kind, entry.title);
        let Some(item) = vault.item_mut(id) else {
            // Unreachable: added one line above. Skipped rather than unwrapped, because this
            // crate denies the panicking constructs and a fixture is not where that exception
            // gets spent.
            continue;
        };

        item.set_tags(["Audit".to_owned()]);

        if entry.title == NO_PASSWORD {
            item.push_field(
                Field::new("Note", "Nothing secret in here.", false).with_kind(FieldKind::Text),
            );
            continue;
        }

        item.push_field(
            Field::new("Username", entry.username, false).with_kind(FieldKind::Username),
        );
        item.push_field(Field::new("Password", entry.password, true));
        if entry.title == TWO_PASSWORD_FIELDS {
            // The same value in a second field of the same item. `Field::new` derives
            // `FieldKind::Password` from `secret`, so this is a second password field rather
            // than a secret note.
            item.push_field(Field::new("Root password", entry.password, true));
        }
    }

    Ok((vault, recovery))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::watchtower::{Verdict, scan};
    use std::collections::HashSet;

    /// The table claims a score per row. zxcvbn is the authority on whether it still holds.
    ///
    /// Read through [`scan`] rather than through `score` directly, because the context argument
    /// is what makes the Northwind row a 2 — see the module docs.
    #[test]
    fn the_table_is_what_zxcvbn_says_today() {
        let (vault, _) = audit_vault(KdfParams::TESTING).expect("testing parameters are valid");
        let report = scan(&vault);

        for entry in ENTRIES {
            if entry.title == NO_PASSWORD {
                continue;
            }
            let Some(item) = vault.items().find(|item| item.title == entry.title) else {
                panic!("{} is in the table and not in the vault", entry.title);
            };
            let Some(finding) = report.findings.iter().find(|f| f.item_id == item.id) else {
                // A clean row produces no finding, so there is nothing to read a score off.
                // Those rows are covered by the spread test below.
                assert!(
                    entry.score > crate::WEAK_MAX_SCORE,
                    "{} claims score {} and would have produced a finding",
                    entry.title,
                    entry.score
                );
                continue;
            };
            assert_eq!(
                finding.score, entry.score,
                "{}: the table says {} and zxcvbn says {}",
                entry.title, entry.score, finding.score
            );
        }
    }

    /// The fixture exists to have a spread. If it ever holds four scores, it is a worse fixture
    /// than the reference vault pretending to be a better one.
    #[test]
    fn every_zxcvbn_score_is_represented() {
        let scores: HashSet<u8> = ENTRIES
            .iter()
            .filter(|entry| entry.title != NO_PASSWORD)
            .map(|entry| entry.score)
            .collect();
        for score in 0..=4 {
            assert!(scores.contains(&score), "no password scores {score}");
        }
    }

    /// Every count in this module's constants, against a real scan.
    #[test]
    fn the_counts_are_what_a_scan_finds() {
        let (vault, _) = audit_vault(KdfParams::TESTING).expect("testing parameters are valid");
        let report = scan(&vault);

        assert_eq!(vault.items().count(), ITEMS);
        assert_eq!(report.passwords, PASSWORD_FIELDS);
        assert_eq!(report.distinct, DISTINCT);

        let weak = report
            .findings
            .iter()
            .filter(|finding| finding.verdict == Verdict::Weak)
            .count();
        let reused = report
            .findings
            .iter()
            .filter(|finding| finding.verdict == Verdict::Reused)
            .count();
        assert_eq!(weak, WEAK_FINDINGS, "weak findings");
        assert_eq!(reused, REUSED_FINDINGS, "reused findings");
        assert_eq!(report.findings.len(), WEAK_FINDINGS + REUSED_FINDINGS);
        assert_eq!(report.worst().len(), FLAGGED_ITEMS);
    }

    /// The groups are the sizes the table says, read off the findings rather than off the
    /// passwords — `shared_with` is the only place a group is visible from outside the module.
    #[test]
    fn the_reuse_groups_are_the_sizes_the_table_claims() {
        let (vault, _) = audit_vault(KdfParams::TESTING).expect("testing parameters are valid");
        let report = scan(&vault);

        let mut sizes: Vec<usize> = report
            .findings
            .iter()
            .filter(|finding| finding.verdict == Verdict::Reused)
            .map(|finding| finding.shared_with.len() + 1)
            .collect();
        sizes.sort_unstable();
        sizes.dedup();
        sizes.sort_unstable();
        assert_eq!(
            sizes,
            vec![2, 3, 5],
            "the groups are one of 3, one of 5, and two of 2"
        );
    }

    /// The canary. Without the item's own words this password is a 4 and produces no finding at
    /// all; with them it is a 2 and lands in the weak group.
    #[test]
    fn the_context_canary_is_weak_only_because_of_its_own_username() {
        let bare = crate::score("priya.raman.2024", &[]);
        let with_context = crate::score("priya.raman.2024", &["Northwind Mail", "priya.raman"]);
        assert!(
            !bare.is_weak(),
            "the canary must be strong without context, or it measures nothing"
        );
        assert!(
            with_context.is_weak(),
            "the canary must be weak with context — scan passes the item's words to zxcvbn"
        );
    }

    /// Two password fields on one item are two fields and one value, and nobody is told they
    /// share a password with themselves.
    #[test]
    fn a_value_repeated_inside_one_item_is_not_reuse() {
        let (vault, _) = audit_vault(KdfParams::TESTING).expect("testing parameters are valid");
        let report = scan(&vault);

        let Some(item) = vault.items().find(|item| item.title == TWO_PASSWORD_FIELDS) else {
            panic!("{TWO_PASSWORD_FIELDS} is in the table and not in the vault");
        };
        assert_eq!(
            item.fields
                .iter()
                .filter(|field| field.kind == FieldKind::Password)
                .count(),
            2
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.item_id == item.id),
            "an item storing one value twice is untidy, not reused"
        );
    }

    /// The fixture is a written table, so two builds are the same vault.
    #[test]
    fn two_builds_are_the_same_vault() {
        let titles = |params| {
            let (vault, _) = audit_vault(params).expect("testing parameters are valid");
            vault
                .items()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(titles(KdfParams::TESTING), titles(KdfParams::TESTING));
    }
}
