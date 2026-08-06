//! Importing a foreign vault — R-29, D-42.
//!
//! One format, and adding a second is a decision rather than a patch: Bitwarden's
//! **unencrypted JSON export**. The survey behind D-42 found that the documented failure of
//! every importer looked at is the same one — *a field dropped in silence* — so the shape of
//! this module is dictated by the report and not by the parser. R-29 is met only when every
//! field is either **mapped** or **named in a refusal**, which is why nothing here is allowed
//! to fall off the end of a `match`.
//!
//! # Three outcomes, not two
//!
//! | Outcome | Meaning |
//! |---------|---------|
//! | mapped | Landed in a field of the target item, unchanged. Counted, listed nowhere. |
//! | converted | Landed with a shape change worth telling the user about. In [`ImportReport::converted`]. |
//! | refused | Has no home here. In [`ImportReport::refusals`], with the reason. |
//!
//! # Neither a refusal nor a conversion may carry a value
//!
//! `docs/ipc-contract.md` §6.8 states it and it is enforced by construction: [`Refusal`] and
//! [`Converted`] hold an item title, a field label and a sentence, all of which are metadata
//! that already crosses IPC in the item list. A report that quoted the values it could not
//! import would be a plaintext dump of exactly the parts of the foreign vault we understood
//! least — the worst possible thing to hand to a UI that cannot wipe its own heap.

pub mod bitwarden;

use serde::Serialize;

use crate::model::ItemKind;

/// A field with no home in TrustVault's model, named so it is not lost in silence.
///
/// Carries no value — see the module documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Refusal {
    /// The title of the item it came from, so the user can find it in the foreign vault.
    pub item_title: String,
    /// Which field, in the foreign vault's own vocabulary.
    pub field: String,
    /// Why it could not be imported, in a sentence a user can act on.
    pub reason: String,
}

/// A field that landed, but with a shape change worth telling the user about.
///
/// Carries no value — see the module documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Converted {
    /// The title of the item it landed in.
    pub item_title: String,
    /// Which field, in the foreign vault's own vocabulary.
    pub field: String,
    /// What was done to it.
    pub note: String,
}

/// How many items of one kind an import produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct KindCount {
    /// The kind.
    pub kind: ItemKind,
    /// How many.
    pub count: usize,
}

/// What an import did, or what a preview says it would do.
///
/// The same type serves both, because a preview that reported less than the commit would be a
/// preview of something else. `docs/ipc-contract.md` §6.8 makes the report returned *after* the
/// import the authoritative one, since the file may change between the two reads.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ImportReport {
    /// Items that would be added, or were.
    pub total: usize,
    /// The same number broken down by kind, in [`ItemKind::ALL`] order.
    pub per_kind: Vec<KindCount>,
    /// Tags this import introduces to the vault, sorted.
    pub tags_created: Vec<String>,
    /// Tags the import uses that the vault already had, sorted — D-43's folders-become-tags
    /// rule merges rather than duplicating.
    pub tags_merged: Vec<String>,
    /// Fields that landed with a shape change.
    pub converted: Vec<Converted>,
    /// Fields that could not land at all.
    pub refusals: Vec<Refusal>,
}

impl ImportReport {
    /// Records a refusal.
    fn refuse(&mut self, item_title: &str, field: impl Into<String>, reason: impl Into<String>) {
        self.refusals.push(Refusal {
            item_title: item_title.to_owned(),
            field: field.into(),
            reason: reason.into(),
        });
    }

    /// Records a conversion.
    fn convert(&mut self, item_title: &str, field: impl Into<String>, note: impl Into<String>) {
        self.converted.push(Converted {
            item_title: item_title.to_owned(),
            field: field.into(),
            note: note.into(),
        });
    }

    /// Counts one imported item against its kind, keeping [`ItemKind::ALL`] order.
    fn count(&mut self, kind: ItemKind) {
        self.total += 1;
        if let Some(entry) = self.per_kind.iter_mut().find(|entry| entry.kind == kind) {
            entry.count += 1;
            return;
        }
        self.per_kind.push(KindCount { kind, count: 1 });
        self.per_kind
            .sort_by_key(|entry| ItemKind::ALL.iter().position(|kind| *kind == entry.kind));
    }
}

/// Days from 1970-01-01 to the given proleptic-Gregorian date.
///
/// Howard Hinnant's `days_from_civil`, which is the standard answer to this and is here rather
/// than as a dependency because a calendar crate for one function is a supply-chain cost with
/// no upside (N-04). Valid for any date this format can express; the import only ever feeds it
/// dates a foreign vault wrote.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Parses the UTC ISO-8601 timestamp Bitwarden writes into Unix milliseconds.
///
/// Deliberately strict, and deliberately narrow: it accepts `YYYY-MM-DDTHH:MM:SS[.fff]Z` and
/// nothing else. A timestamp is the one part of an import where a lenient parser is worse than
/// no parser — guessing an offset silently shifts every date in the vault by hours, and the
/// caller's answer to `None` is a refusal the user can see.
fn iso8601_utc_ms(text: &str) -> Option<i64> {
    let text = text.strip_suffix('Z')?;
    let (date, time) = text.split_once('T')?;

    let mut date = date.split('-');
    let year: i64 = date.next()?.parse().ok()?;
    let month: i64 = date.next()?.parse().ok()?;
    let day: i64 = date.next()?.parse().ok()?;
    if date.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (time, fraction) = match time.split_once('.') {
        Some((time, fraction)) => (time, fraction),
        None => (time, ""),
    };
    let mut time = time.split(':');
    let hour: i64 = time.next()?.parse().ok()?;
    let minute: i64 = time.next()?.parse().ok()?;
    let second: i64 = time.next()?.parse().ok()?;
    if time.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    // Milliseconds, padded or truncated to three digits. `.9` is 900 ms, not 9.
    let millis: i64 = if fraction.is_empty() {
        0
    } else {
        let mut digits = String::from(fraction);
        digits.truncate(3);
        while digits.len() < 3 {
            digits.push('0');
        }
        digits.parse().ok()?
    };

    let days = days_from_civil(year, month, day);
    Some(((days * 24 + hour) * 60 + minute) * 60_000 + second * 1_000 + millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_and_a_leap_day_convert() {
        assert_eq!(iso8601_utc_ms("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(
            days_from_civil(2024, 2, 29),
            days_from_civil(2024, 3, 1) - 1
        );
        // Computed outside this crate rather than from the code it checks, which is the only
        // way a hand-written date routine is worth testing at all.
        assert_eq!(
            iso8601_utc_ms("2026-08-06T12:34:56.789Z"),
            Some(1_786_019_696_789)
        );
    }

    #[test]
    fn a_missing_fraction_is_zero_and_a_short_one_is_padded() {
        // `.9` is nine hundred milliseconds. Read as a plain integer it would be nine, which
        // is the kind of wrong nobody notices and nothing detects.
        assert_eq!(
            iso8601_utc_ms("2026-08-06T12:34:56Z"),
            iso8601_utc_ms("2026-08-06T12:34:56.000Z")
        );
        let base = iso8601_utc_ms("2026-08-06T12:34:56Z").unwrap_or_default();
        assert_eq!(iso8601_utc_ms("2026-08-06T12:34:56.9Z"), Some(base + 900));
        assert_eq!(
            iso8601_utc_ms("2026-08-06T12:34:56.1234Z"),
            Some(base + 123)
        );
    }

    #[test]
    fn anything_that_is_not_utc_iso_8601_is_refused_rather_than_guessed() {
        // Each of these is a shape some exporter emits. Guessing at any of them shifts every
        // date in the imported vault, silently.
        for text in [
            "2026-08-06T12:34:56+02:00",
            "2026-08-06 12:34:56Z",
            "06/08/2026",
            "2026-13-01T00:00:00Z",
            "2026-08-06T25:00:00Z",
            "2026-08-06T00:00Z",
            "",
        ] {
            assert_eq!(iso8601_utc_ms(text), None, "{text} must not parse");
        }
    }

    #[test]
    fn counts_are_kept_in_the_dialog_s_own_order() {
        let mut report = ImportReport::default();
        report.count(ItemKind::Identity);
        report.count(ItemKind::Login);
        report.count(ItemKind::Login);
        report.count(ItemKind::Note);

        assert_eq!(report.total, 3 + 1);
        let kinds: Vec<ItemKind> = report.per_kind.iter().map(|entry| entry.kind).collect();
        assert_eq!(
            kinds,
            vec![ItemKind::Login, ItemKind::Note, ItemKind::Identity]
        );
        assert_eq!(report.per_kind.first().map(|entry| entry.count), Some(2));
    }
}
