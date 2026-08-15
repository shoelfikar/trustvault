//! The local half of Watchtower: zxcvbn strength scoring and cross-item reuse detection.
//!
//! R-23, R-24, and the local half of S-07a. The network half is `src-tauri/src/hibp.rs` and it
//! is not reachable from here — N-02 keeps the core free of sockets, and a vault-format crate
//! that can open one is a vault-format crate that will eventually be asked to.
//!
//! # Why the scoring lives here and not in the host — D-78
//!
//! `score_password` served onboarding's meter from `src-tauri` first, and its own doc comment
//! gives the reason: zxcvbn's dictionaries are hundreds of kilobytes, and parsing them in the
//! webview is paid for on every cold start. That argument is about the **webview**, and it is
//! untouched by this move. What decides between the host and the core is different: scoring
//! every password in the vault means reading every password in the vault, and doing that from
//! `src-tauri` would mean lifting a thousand plaintext values across a crate boundary to score
//! them and dropping them again. The core already holds them. The host now calls in here for
//! its meter too, so there is one definition of what "Weak" means rather than two that agree
//! until the day one of them is edited — D-44's argument, one phase later.
//!
//! # What may leave this module
//!
//! The grouping key is a hash of a password, and **a hash of a short secret is a secret**. It
//! never appears in a [`Finding`], an event, an error, or a shortened "group id" the webview
//! could colour rows by. [`Finding::shared_with`] names the other **items** — ids the item list
//! already carries — which is the information the user needs and none of the information an
//! offline attacker does. See `docs/ipc-contract.md` §6.9.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::model::{FieldId, FieldKind, ItemId, ItemStatus, now_ms};
use crate::vault::Vault;

/// The highest zxcvbn score still reported as weak — R-24, and the "not by length" half of the
/// task that asks for it.
///
/// **Two, and it was set by measuring rather than by choosing.** The first draft of this file
/// said one, on the assumption that "Weak" on the meter and "weak" in Watchtower were the same
/// line. Measured on 2026-08-15, at zxcvbn's offline-slow-hashing rate:
///
/// | Password | Score | Cracked in |
/// |---|---|---|
/// | `password` | 0 | less than a second |
/// | `hunter2` | 1 | less than a second |
/// | `Tr0ub4dour&3` | **2** | **31 minutes** |
/// | `Jakarta2019!`, told the vault is "Jakarta" | **2** | **16 minutes** |
/// | `Jakarta2019!`, told nothing | 3 | 3 hours |
/// | `correct horse battery staple` | 4 | centuries |
///
/// The Watchtower view has said *"Crackable in a matter of hours"* under its Weak group since
/// D-36 drew it, and that sentence describes the **31-minute** row, not the under-a-second
/// ones. The design's own copy is the evidence, and it predates this decision. zxcvbn agrees
/// from the other direction: its documented meaning for 3 is the first score that resists an
/// **offline** attack, and a breach corpus is offline by definition — a score of 2 buys
/// protection from unthrottled online guessing and nothing more.
///
/// A length threshold is what this constant exists instead of, and the table is why:
/// `Tr0ub4dour&3` is twelve characters with four character classes and every length-and-class
/// rule ever written passes it, while `correct horse battery staple` is lower case and spaces
/// and holds for centuries. Any rule counting characters gets both of them backwards.
///
/// **It leaves the meter and this disagreeing, and that is an open question rather than a
/// silent edit**: score 2 renders as *Fair* on the strength meter and lands in the *Weak
/// passwords* group here. See `trustvault-state.md`.
pub const WEAK_MAX_SCORE: u8 = 2;

/// A zxcvbn score, the word that goes beside it, and the crack time in zxcvbn's own phrasing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strength {
    /// zxcvbn's 0–4 score, which drives the meter's four segments.
    pub score: u8,
    /// Weak / Fair / Strong / Excellent — `MASTER.md` §2.
    ///
    /// Status is never colour alone, so every surface that draws the score draws this word with
    /// it. The colour is a second channel, never the only one.
    pub label: &'static str,
    /// Human crack-time estimate, in zxcvbn's own phrasing — R-24.
    pub crack_time: String,
}

impl Strength {
    /// Whether this score is one R-24 reports as weak.
    #[must_use]
    pub fn is_weak(&self) -> bool {
        self.score <= WEAK_MAX_SCORE
    }
}

/// What a [`Finding`] says about a password field.
///
/// One verdict per finding, and a field that is both weak and reused produces **two** findings
/// rather than one ranked verdict. The report is where nothing is lost; [`ItemStatus`] is the
/// cache, and a cache is where the ranking happens — see [`worst`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Verdict {
    /// zxcvbn scored it at or below [`WEAK_MAX_SCORE`] — R-24.
    Weak,
    /// Another item in this vault carries the same value — R-23.
    Reused,
    /// Found in a breach corpus. **Nothing in this module produces it** — it comes from the
    /// host's breach check, and the variant lives here so the two halves speak one language.
    Breached,
    /// Past a rotation date. **Nothing in this project produces it yet**: no requirement in
    /// R-23…R-26 defines one, and the view's group for it stays empty rather than implying a
    /// check that is not running.
    Expired,
}

impl Verdict {
    /// The [`ItemStatus`] this verdict caches as.
    #[must_use]
    pub fn status(self) -> ItemStatus {
        match self {
            Self::Weak => ItemStatus::Weak,
            Self::Reused => ItemStatus::Reused,
            Self::Breached => ItemStatus::Breached,
            Self::Expired => ItemStatus::Expired,
        }
    }

    /// How loudly this verdict speaks, for the one-status-per-item cache.
    ///
    /// Higher wins. Breached outranks reused outranks weak because that is the order the user
    /// should act in, and because the item list draws one pip and has to pick.
    const fn severity(self) -> u8 {
        match self {
            Self::Weak => 1,
            Self::Expired => 2,
            Self::Reused => 3,
            Self::Breached => 4,
        }
    }
}

/// One verdict about one password field.
///
/// Carries no password and nothing derived from one. `score` and `crack_time` are properties of
/// the value that the user is being asked to act on; neither narrows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The item the field belongs to.
    pub item_id: ItemId,
    /// Which password field the verdict is about — an item may carry more than one.
    pub field_id: FieldId,
    /// What this finding says.
    pub verdict: Verdict,
    /// zxcvbn's 0–4 score for the value — R-24.
    pub score: u8,
    /// The crack-time estimate in words — R-24.
    pub crack_time: String,
    /// The **other** items carrying the same value — R-23. Empty unless `verdict` is
    /// [`Verdict::Reused`].
    pub shared_with: Vec<ItemId>,
}

/// The result of a local scan.
///
/// `distinct` is the count a breach check would cost: **one range request per distinct value,
/// never per item** (S-07b). A vault where twelve items share a password costs one request, and
/// the reference vault is the worst case by construction rather than the typical one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// When the scan ran, in milliseconds since the Unix epoch.
    pub scanned_at: i64,
    /// How many password fields were examined.
    pub passwords: usize,
    /// How many distinct values were among them.
    pub distinct: usize,
    /// Everything worth acting on. **A clean field is the absence of a row**, never a row
    /// saying `strong` — that lives in the item's cached [`ItemStatus`] instead of being
    /// carried twice.
    pub findings: Vec<Finding>,
}

impl Report {
    /// The worst verdict recorded against each item, which is what the status cache stores.
    ///
    /// Items with no finding are absent: the caller writes [`ItemStatus::Strong`] for those,
    /// because "scanned and clean" is not something this map can distinguish from "not in this
    /// report at all".
    #[must_use]
    pub fn worst(&self) -> HashMap<ItemId, ItemStatus> {
        let mut worst: HashMap<ItemId, Verdict> = HashMap::new();
        for finding in &self.findings {
            worst
                .entry(finding.item_id)
                .and_modify(|current| {
                    if finding.verdict.severity() > current.severity() {
                        *current = finding.verdict;
                    }
                })
                .or_insert(finding.verdict);
        }
        worst
            .into_iter()
            .map(|(id, verdict)| (id, verdict.status()))
            .collect()
    }
}

/// The grouping key: SHA-256 of a password, and **it never leaves this module**.
///
/// Hashed rather than grouped on the value itself for a reason that is about memory rather than
/// about secrecy: a `HashMap` keyed on `String` would copy every plaintext password into an
/// allocation nothing zeroizes. A fixed-size digest copies nothing, and this one is wiped when
/// the map is dropped. It is still a secret — SHA-256 of a short password is a dictionary
/// attack away from the password — which is why it is private, unprintable, and never returned.
#[derive(PartialEq, Eq, Hash, Zeroize, ZeroizeOnDrop)]
struct GroupKey([u8; 32]);

impl GroupKey {
    fn of(password: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        Self(hasher.finalize().into())
    }
}

/// Scores a password, with `context` as the words zxcvbn should treat as local knowledge.
///
/// `context` is what the caller knows and zxcvbn cannot: the vault name at onboarding, and the
/// item's own title and username during a scan. zxcvbn penalizes a password built out of the
/// words its owner is surrounded by, and it can only do that when it is told what they are —
/// D-12's amendment is the measurement behind that sentence.
#[must_use]
pub fn score(password: &str, context: &[&str]) -> Strength {
    // An empty box is not a weak password, it is no password. Scoring it 0/"Weak" paints the
    // meter red before the user has typed anything, which reads as a failure state.
    if password.is_empty() {
        return Strength {
            score: 0,
            label: "",
            crack_time: String::new(),
        };
    }

    let entropy = zxcvbn::zxcvbn(password, context);
    let score = u8::from(entropy.score());
    Strength {
        score,
        label: match score {
            0 | 1 => "Weak",
            2 => "Fair",
            3 => "Strong",
            _ => "Excellent",
        },
        crack_time: entropy
            .crack_times()
            .offline_slow_hashing_1e4_per_second()
            .to_string(),
    }
}

/// Scans every password field in the vault — R-23, R-24.
///
/// Touches no network and cannot: this crate has no socket in its dependency tree, which is the
/// N-02 boundary doing the work D-76 split the commands for. Called with breach checking off —
/// the default — this is the whole scan.
///
/// Every value is scored once even when several items share it, because the crack-time string
/// is the same for all of them and re-deriving it per item is the only cost that would grow
/// with reuse rather than with the vault.
#[must_use]
pub fn scan(vault: &Vault) -> Report {
    // Two passes over one collected list rather than two walks of the vault: the first pass has
    // to finish before any reuse verdict can be issued, because a group is only visible once
    // every member has been seen.
    let mut passwords: Vec<(ItemId, FieldId, &str)> = Vec::new();
    let mut context: HashMap<ItemId, Vec<&str>> = HashMap::new();

    for item in vault.items() {
        let mut words: Vec<&str> = vec![item.title.as_str()];
        for field in &item.fields {
            if field.kind == FieldKind::Password {
                passwords.push((item.id, field.id, field.value.expose()));
            } else if matches!(field.kind, FieldKind::Username | FieldKind::Email) {
                // The username is exactly the local knowledge zxcvbn has no way to guess, and
                // a password built out of it is the case D-12's amendment showed the default
                // dictionaries miss.
                words.push(field.value.expose());
            }
        }
        context.insert(item.id, words);
    }

    let mut groups: HashMap<GroupKey, Vec<usize>> = HashMap::new();
    for (index, (_, _, password)) in passwords.iter().enumerate() {
        groups
            .entry(GroupKey::of(password))
            .or_default()
            .push(index);
    }

    let mut findings = Vec::new();
    for members in groups.values() {
        // One scoring per distinct value, not per item — see the doc comment.
        let Some(&first) = members.first() else {
            continue;
        };
        let Some(&(_, _, password)) = passwords.get(first) else {
            continue;
        };
        let owners: Vec<ItemId> = members
            .iter()
            .filter_map(|&index| passwords.get(index).map(|&(item_id, _, _)| item_id))
            .collect();

        for &index in members {
            let Some(&(item_id, field_id, _)) = passwords.get(index) else {
                continue;
            };
            let words = context.get(&item_id).cloned().unwrap_or_default();
            let strength = score(password, &words);

            if strength.is_weak() {
                findings.push(Finding {
                    item_id,
                    field_id,
                    verdict: Verdict::Weak,
                    score: strength.score,
                    crack_time: strength.crack_time.clone(),
                    shared_with: Vec::new(),
                });
            }

            // Reuse is counted across **items**, not across fields: an item that stores the
            // same password in two of its own fields is untidy, not reused, and reporting it
            // would put a row on the screen naming the item as sharing with itself.
            let shared_with: Vec<ItemId> = owners
                .iter()
                .copied()
                .filter(|&other| other != item_id)
                .collect();
            if !shared_with.is_empty() {
                findings.push(Finding {
                    item_id,
                    field_id,
                    verdict: Verdict::Reused,
                    score: strength.score,
                    crack_time: strength.crack_time,
                    shared_with,
                });
            }
        }
    }

    // Deterministic output: `HashMap` iteration order is not, and a report that reorders itself
    // between two scans of an unchanged vault would make every diff of the evidence useless.
    findings.sort_by(|a, b| {
        (a.item_id, a.field_id, a.verdict.severity()).cmp(&(
            b.item_id,
            b.field_id,
            b.verdict.severity(),
        ))
    });

    Report {
        scanned_at: now_ms(),
        passwords: passwords.len(),
        distinct: groups.len(),
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::KdfParams;
    use crate::model::{Field, ItemKind};

    /// No context, the common case for a scoring test.
    const NONE: &[&str] = &[];

    fn vault() -> Vault {
        let (vault, _) = Vault::create("Test", "correct horse battery staple", KdfParams::TESTING)
            .expect("testing parameters are valid");
        vault
    }

    fn with_password(vault: &mut Vault, title: &str, password: &str) -> ItemId {
        let id = vault.add_item(ItemKind::Login, title.to_owned());
        let item = vault.item_mut(id).expect("just added");
        item.push_field(Field::new("Password", password, true).with_kind(FieldKind::Password));
        id
    }

    #[test]
    fn an_empty_password_is_not_scored_as_a_weak_one() {
        let strength = score("", NONE);
        assert_eq!(strength.label, "", "no word before anything is typed");
        assert!(strength.crack_time.is_empty());
    }

    #[test]
    fn common_passwords_score_low_and_a_passphrase_scores_high() {
        assert_eq!(score("password", NONE).score, 0);
        assert!(score("hunter2", NONE).score <= 1);
        assert!(score("correct horse battery staple", NONE).score >= 4);
    }

    #[test]
    fn every_score_has_a_word_beside_it() {
        // MASTER.md: status is never colour alone. A segment count with no label would be
        // exactly that.
        for password in ["a", "aa11", "Tr0ub4dour&3", "correct horse battery staple"] {
            assert!(
                !score(password, NONE).label.is_empty(),
                "{password} had no label"
            );
        }
    }

    /// **Measured, and it contradicts D-12's example.**
    ///
    /// D-12 justified zxcvbn with `Jakarta2019!` — the password entropy-only scoring calls
    /// strong and zxcvbn supposedly catches. It does not: with the default dictionaries it
    /// scores **3 / Strong**, because "Jakarta" is not in zxcvbn's English-centric frequency
    /// lists. The decision stands, but its example was aspirational rather than measured, and
    /// this test exists so nobody re-derives the claim from the decision log and believes it.
    ///
    /// Moved here from `src-tauri/src/commands/strength.rs` with the scoring itself — D-78.
    #[test]
    fn a_local_word_is_only_caught_when_zxcvbn_is_told_about_it() {
        let blind = score("Jakarta2019!", NONE);
        assert_eq!(
            blind.score, 3,
            "the default dictionaries do not know the city"
        );

        let informed = score("Jakarta2019!", &["Jakarta"]);
        assert!(
            informed.score < blind.score,
            "context has to change the answer, or passing it is theatre"
        );
    }

    /// The weak threshold is a score, not a length — the second half of R-24's task.
    #[test]
    fn the_weak_threshold_is_a_score_and_not_a_length() {
        // Twelve characters, mixed case, digits and a symbol — every length-and-class rule
        // ever written passes it, and zxcvbn puts it 31 minutes from cracked.
        let short_and_bad = score("Tr0ub4dour&3", NONE);
        assert_eq!(short_and_bad.score, 2);
        assert!(
            short_and_bad.is_weak(),
            "a length rule would have passed this one"
        );

        // Longer, all lower case, no digits, no symbols — and it holds for centuries.
        let long_and_good = score("correct horse battery staple", NONE);
        assert_eq!(long_and_good.score, 4);
        assert!(!long_and_good.is_weak());
        assert!(
            long_and_good.crack_time.len() > 1,
            "R-24 asks for the estimate in words"
        );
    }

    /// The threshold's own boundary, pinned by measurement so that moving [`WEAK_MAX_SCORE`]
    /// has to be a decision rather than an edit that no test notices.
    #[test]
    fn the_boundary_between_reported_and_not_is_where_it_was_measured() {
        assert_eq!(WEAK_MAX_SCORE, 2);
        assert!(score("Tr0ub4dour&3", NONE).is_weak(), "2 is reported");
        assert!(
            !score("Jakarta2019!", NONE).is_weak(),
            "3 is not reported — it is the first score zxcvbn calls offline-resistant"
        );
    }

    /// R-23's acceptance criterion, verbatim: three items sharing a password are all reported.
    #[test]
    fn three_items_sharing_a_password_are_all_reported() {
        let mut vault = vault();
        let a = with_password(&mut vault, "GitHub", "shared-Ka9!wqm2");
        let b = with_password(&mut vault, "GitLab", "shared-Ka9!wqm2");
        let c = with_password(&mut vault, "Fastmail", "shared-Ka9!wqm2");
        let alone = with_password(&mut vault, "Stripe", "qX7#vn2Lp!4dRt");

        let report = scan(&vault);
        assert_eq!(report.passwords, 4);
        assert_eq!(report.distinct, 2, "one request per distinct value, S-07b");

        for id in [a, b, c] {
            let finding = report
                .findings
                .iter()
                .find(|f| f.item_id == id && f.verdict == Verdict::Reused)
                .unwrap_or_else(|| panic!("{id} was not reported"));
            let mut others = finding.shared_with.clone();
            others.sort();
            let mut expected: Vec<ItemId> = [a, b, c].into_iter().filter(|&x| x != id).collect();
            expected.sort();
            assert_eq!(others, expected, "each one names the other two, not itself");
        }

        assert!(
            !report
                .findings
                .iter()
                .any(|f| f.item_id == alone && f.verdict == Verdict::Reused),
            "a password used once is not reuse"
        );
    }

    /// An item is not "sharing with itself" when it stores one password in two of its fields.
    #[test]
    fn an_item_sharing_a_password_with_itself_is_not_reuse() {
        let mut vault = vault();
        let id = with_password(&mut vault, "Router", "qX7#vn2Lp!4dRt");
        let item = vault.item_mut(id).expect("just added");
        item.push_field(
            Field::new("Admin password", "qX7#vn2Lp!4dRt", true).with_kind(FieldKind::Password),
        );

        let report = scan(&vault);
        assert_eq!(report.passwords, 2);
        assert_eq!(report.distinct, 1);
        assert!(
            !report.findings.iter().any(|f| f.verdict == Verdict::Reused),
            "one item, one password, two fields — untidy, not reused"
        );
    }

    /// A field can be both, and the report says both rather than picking one.
    #[test]
    fn a_weak_and_reused_password_produces_two_findings() {
        let mut vault = vault();
        with_password(&mut vault, "Forum", "password");
        let second = with_password(&mut vault, "Wiki", "password");

        let report = scan(&vault);
        let verdicts: Vec<Verdict> = report
            .findings
            .iter()
            .filter(|f| f.item_id == second)
            .map(|f| f.verdict)
            .collect();
        assert!(verdicts.contains(&Verdict::Weak));
        assert!(verdicts.contains(&Verdict::Reused));

        // The cache picks one, and it picks the louder.
        assert_eq!(report.worst().get(&second), Some(&ItemStatus::Reused));
    }

    /// A clean field is the absence of a row — §6.9, and the reason `worst()` cannot report
    /// `Strong` on its own.
    #[test]
    fn a_clean_vault_produces_no_findings_at_all() {
        let mut vault = vault();
        with_password(&mut vault, "Stripe", "qX7#vn2Lp!4dRt");
        with_password(&mut vault, "Vercel", "hZ3@mkw8Tf!2yB");

        let report = scan(&vault);
        assert_eq!(report.passwords, 2);
        assert_eq!(report.distinct, 2);
        assert!(report.findings.is_empty(), "no row saying strong");
        assert!(report.worst().is_empty());
    }

    /// Only password fields are scanned. A username that happens to look weak is not a finding,
    /// and a note is not a password.
    #[test]
    fn only_password_fields_are_scanned() {
        let mut vault = vault();
        let id = vault.add_item(ItemKind::Login, "Notes".to_owned());
        let item = vault.item_mut(id).expect("just added");
        item.push_field(Field::new("Username", "admin", false).with_kind(FieldKind::Username));
        item.push_field(Field::new("Note", "password", false).with_kind(FieldKind::Note));

        let report = scan(&vault);
        assert_eq!(report.passwords, 0);
        assert_eq!(report.distinct, 0);
        assert!(report.findings.is_empty());
    }

    /// The item's own words are handed to zxcvbn, and that is what moves this password across
    /// the threshold rather than merely nudging its number.
    ///
    /// `Jakarta2019!` scores **3** blind and **2** when zxcvbn is told the item is called
    /// Jakarta, and [`WEAK_MAX_SCORE`] sits between them. Scanned without context it would be
    /// reported as nothing at all — which is the entire case D-12's amendment was written for,
    /// now on the surface that acts on it rather than on the meter that displays it.
    #[test]
    fn the_items_own_words_are_what_move_it_across_the_threshold() {
        let mut vault = vault();
        let id = vault.add_item(ItemKind::Login, "Jakarta".to_owned());
        let item = vault.item_mut(id).expect("just added");
        item.push_field(
            Field::new("Password", "Jakarta2019!", true).with_kind(FieldKind::Password),
        );

        assert!(
            !score("Jakarta2019!", NONE).is_weak(),
            "blind, this password is reported as nothing"
        );

        let report = scan(&vault);
        let finding = report
            .findings
            .iter()
            .find(|f| f.verdict == Verdict::Weak)
            .expect("the title is the context that catches it");
        assert_eq!(finding.score, 2);
    }

    /// The username is context too, and it is the half a title cannot cover.
    #[test]
    fn a_password_built_from_the_username_is_caught() {
        let mut vault = vault();
        let id = vault.add_item(ItemKind::Login, "Mail".to_owned());
        let item = vault.item_mut(id).expect("just added");
        item.push_field(Field::new("Username", "wintermute", false).with_kind(FieldKind::Username));
        item.push_field(
            Field::new("Password", "wintermute88", true).with_kind(FieldKind::Password),
        );

        let report = scan(&vault);
        let scored = report
            .findings
            .iter()
            .find(|f| f.verdict == Verdict::Weak)
            .expect("a password that is the username plus two digits is not a strong one");
        assert!(scored.score <= WEAK_MAX_SCORE);
    }

    /// Two scans of an unchanged vault produce the same list in the same order.
    #[test]
    fn the_report_is_ordered_the_same_way_twice() {
        let mut vault = vault();
        for index in 0..12 {
            with_password(&mut vault, &format!("Item {index}"), "password");
        }

        let one = scan(&vault);
        let two = scan(&vault);
        assert_eq!(one.findings, two.findings, "HashMap order must not show");
    }
}
