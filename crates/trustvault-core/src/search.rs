//! Fuzzy matching for the command palette — R-16, D-46.
//!
//! The matching runs **here**, against plaintext that never leaves the core, and not in the
//! webview. D-46 records why: R-16 asks the palette to search titles, usernames, URLs and tags,
//! and only two of those four are in the summary the item list already receives. Getting the
//! other two into JavaScript is a *permitted* crossing — a non-secret field's value may cross
//! freely — and it is still the wrong trade, because it would put the entire identifying
//! surface of the vault into a heap that cannot be wiped, on every render, to save one round
//! trip against a budget (S-04) with room for it.
//!
//! # What is matched, and what is deliberately not
//!
//! The title, the tags, and the value of every field that is **not secret**. R-16 names four
//! haystacks — titles, usernames, URLs and tags — and this is deliberately one rule rather than
//! that list.
//!
//! **A secret field's value is never matched, and that is a security property rather than a
//! scoping choice.** A palette that matched stored passwords would answer the question "is
//! *this* string the password for one of these items?" for anyone typing into it — the vault
//! would confirm a guess through the ranking, with nothing revealed and no audit entry written.
//! The rule is absolute here so that nobody has to remember it at the call site.
//!
//! Written first as *non-secret fields of kind [`FieldKind::Username`], [`FieldKind::Url`] or
//! [`FieldKind::Email`]*, to match R-16 word for word, and falsified within the hour by the IPC
//! harness's own fixture: [`crate::Item::set_field`] guesses `kind` from `secret`, so a username
//! stored through it is a [`FieldKind::Text`] and was not searchable. `kind` is how a field
//! *renders* and is only as accurate as whoever created the field; `secret` is what the user
//! declared and is the thing that must not be matched. Filtering on the first would make
//! searchability quietly wrong in a way no user could diagnose — "why does this item not come
//! up when I type its account name?" — so the gate is the second, alone.

use crate::model::Item;

/// Added to a match found in the item's title — the strongest signal of intent.
const TITLE_WEIGHT: u32 = 300;
/// Added to a match found in one of the item's tags.
const TAG_WEIGHT: u32 = 200;
/// Added to a match found in a searchable field's value.
const FIELD_WEIGHT: u32 = 100;

/// What any match is worth before the shape of it is considered.
const BASE: u32 = 100;
/// The haystack begins with the query.
const PREFIX: u32 = 25;
/// The match begins at a word boundary, which reads as intentional to a user typing initials.
const WORD_START: u32 = 15;
/// The haystack *is* the query.
const EXACT: u32 = 40;
/// How far a late first character can push a match down before the penalty stops growing.
const MAX_LEAD_PENALTY: usize = 20;
/// How far scattered characters can push a match down before the penalty stops growing.
const MAX_GAP_PENALTY: usize = 30;

/// Characters a match may start after and still count as starting a word.
fn is_separator(character: char) -> bool {
    character.is_whitespace() || matches!(character, '-' | '_' | '.' | '/' | ':' | '@' | ',')
}

/// Lowercases one character without allocating.
///
/// No allocation is not a micro-optimization here: every haystack this walks is plaintext out
/// of an open vault, and `to_lowercase()` on each of them would leave a copy of the vault's
/// identifying surface on the heap once per keystroke, for the whole session.
fn fold(character: char) -> char {
    character.to_lowercase().next().unwrap_or(character)
}

/// Scores `needle` against `haystack`, or `None` if it does not match at all.
///
/// The match is a **subsequence**: the needle's characters appear in order, not necessarily
/// together, which is what makes `gh` find `GitHub` and `chase` find `Chase Sapphire`. The
/// score then rewards the shapes a person means — a prefix, a word start, characters close
/// together — so that `git` ranks `GitHub` above `Digital Ocean`, which it also matches.
///
/// `needle` must already be lowercased and non-empty; both are the caller's job because it
/// does them once per query rather than once per haystack.
fn fuzzy(haystack: &str, needle: &str) -> Option<u32> {
    let mut wanted = needle.chars();
    let mut next = wanted.next()?;

    let mut score = BASE;
    let mut first = None;
    let mut last = None;
    let mut matched = 0usize;
    let mut complete = false;
    let mut previous = None;
    let mut length = 0usize;

    for (position, raw) in haystack.chars().enumerate() {
        length = position + 1;
        if !complete && fold(raw) == next {
            if first.is_none() {
                first = Some(position);
                if position == 0 {
                    score += PREFIX;
                } else if previous.is_some_and(is_separator) {
                    score += WORD_START;
                }
            }
            last = Some(position);
            matched += 1;
            match wanted.next() {
                Some(character) => next = character,
                // Every character is placed. The walk continues only to learn how long the
                // haystack is, which the exact-match bonus below needs.
                None => complete = true,
            }
        }
        previous = Some(raw);
    }

    if !complete {
        return None;
    }

    let start = first.unwrap_or(0);
    let end = last.unwrap_or(0);
    // Characters skipped *inside* the match. A tight run scores above a scattered one even
    // when both start in the same place.
    let gaps = (end + 1).saturating_sub(start).saturating_sub(matched);
    if start == 0 && matched == length {
        score += EXACT;
    }
    Some(
        score
            .saturating_sub(u32::try_from(start.min(MAX_LEAD_PENALTY)).unwrap_or(0))
            .saturating_sub(u32::try_from(gaps.min(MAX_GAP_PENALTY)).unwrap_or(0)),
    )
}

/// Scores one item against an already-lowercased, non-empty query.
///
/// The item's score is its **best** haystack, not the sum of them: an item matching in three
/// places is not three times the answer, and summing would rank a note that mentions a word
/// twice above the item actually called it.
pub(crate) fn score_item(item: &Item, needle: &str) -> Option<u32> {
    let mut best = fuzzy(&item.title, needle).map(|points| points + TITLE_WEIGHT);

    for tag in &item.tags {
        if let Some(points) = fuzzy(tag, needle) {
            let scored = points + TAG_WEIGHT;
            best = Some(best.map_or(scored, |current: u32| current.max(scored)));
        }
    }

    // `secret` is the only gate, and the filter is written inline so there is no helper anyone
    // could later add a second condition to. See the module documentation.
    for field in item.fields.iter().filter(|field| !field.secret) {
        if let Some(points) = fuzzy(field.value.expose(), needle) {
            let scored = points + FIELD_WEIGHT;
            best = Some(best.map_or(scored, |current: u32| current.max(scored)));
        }
    }

    best
}

/// Ranks `items` against `query`, best first, and keeps at most `limit`.
///
/// An empty query is not an empty result: it returns the first `limit` items in vault order,
/// which is what the palette shows the moment it opens, before anything is typed.
///
/// Ties break by title and then by identifier, so the order is total and does not depend on
/// the sort's stability. A palette whose rows swap places between two identical queries reads
/// as a bug, and Enter is bound to the row under the cursor.
pub(crate) fn rank<'a>(
    items: impl Iterator<Item = &'a Item>,
    query: &str,
    limit: usize,
) -> Vec<&'a Item> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return items.take(limit).collect();
    }

    let mut hits: Vec<(u32, &Item)> = items
        .filter_map(|item| score_item(item, &needle).map(|score| (score, item)))
        .collect();

    hits.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });

    hits.into_iter().take(limit).map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Field, FieldKind, ItemKind};

    fn login(title: &str) -> Item {
        Item::new(ItemKind::Login, title)
    }

    fn scored(item: &Item, query: &str) -> Option<u32> {
        score_item(item, &query.to_lowercase())
    }

    #[test]
    fn a_subsequence_matches_and_a_missing_character_does_not() {
        let item = login("GitHub");
        assert!(scored(&item, "gh").is_some());
        assert!(scored(&item, "ghb").is_some());
        assert!(scored(&item, "ghz").is_none());
    }

    #[test]
    fn matching_ignores_case_in_both_directions() {
        assert!(scored(&login("GitHub"), "github").is_some());
        assert!(scored(&login("github"), "GITHUB").is_some());
    }

    #[test]
    fn a_prefix_outranks_a_match_in_the_middle() {
        let prefix = scored(&login("Git Hub"), "git").unwrap();
        let middle = scored(&login("Digital Git"), "git").unwrap();
        assert!(prefix > middle, "{prefix} should beat {middle}");
    }

    #[test]
    fn a_tight_match_outranks_a_scattered_one() {
        let tight = scored(&login("Chase"), "chs").unwrap();
        let scattered = scored(&login("Cooperative House Society"), "chs").unwrap();
        assert!(tight > scattered, "{tight} should beat {scattered}");
    }

    #[test]
    fn a_title_outranks_a_tag_and_a_tag_outranks_a_field() {
        let by_title = login("Acme");

        let mut by_tag = login("Something else");
        by_tag.set_tags(["acme".to_owned()]);

        let mut by_field = login("Something else again");
        by_field.push_field(Field::new("Username", "acme", false).with_kind(FieldKind::Username));

        let title = scored(&by_title, "acme").unwrap();
        let tag = scored(&by_tag, "acme").unwrap();
        let field = scored(&by_field, "acme").unwrap();
        assert!(title > tag && tag > field, "{title} > {tag} > {field}");

        // The item's own score is its best haystack, not the sum: adding a second place the
        // query matches must not lift it above an item whose *title* is the query.
        by_tag.push_field(Field::new("Username", "acme", false).with_kind(FieldKind::Username));
        assert!(scored(&by_tag, "acme").unwrap() < title);
    }

    /// Every non-secret field is searched, whatever its kind says it is.
    ///
    /// `Text` is in this list on purpose: it is what [`Item::set_field`] produces for any
    /// non-secret field, so a rule that searched only the three "identifying" kinds would miss
    /// most of the fields this application actually creates.
    #[test]
    fn every_non_secret_field_is_searched_whatever_its_kind() {
        for kind in [
            FieldKind::Username,
            FieldKind::Url,
            FieldKind::Email,
            FieldKind::Text,
            FieldKind::Note,
        ] {
            let mut item = login("Untitled");
            item.push_field(Field::new("Field", "octocat", false).with_kind(kind));
            assert!(scored(&item, "octocat").is_some(), "{kind:?} is searchable");
        }
    }

    /// The rule stated the other way round, on the path the application really uses.
    #[test]
    fn a_field_added_through_set_field_is_searchable() {
        let mut item = login("Untitled");
        item.set_field("Username", "octocat", false);
        item.set_field("Password", "correct-horse", true);
        assert!(scored(&item, "octocat").is_some(), "the username is found");
        assert!(
            scored(&item, "correct-horse").is_none(),
            "the password is not"
        );
    }

    /// The one that is a security property rather than a ranking preference.
    ///
    /// A palette that matched stored passwords would confirm a guessed password through the
    /// ranking alone — no reveal, no audit entry, no secret crossing IPC, and the answer on
    /// screen. The rule holds whatever the field's kind is, because `secret` is what the user
    /// declared and `kind` is only how it renders.
    #[test]
    fn a_secret_value_is_never_matched_whatever_its_kind() {
        for kind in [
            FieldKind::Password,
            FieldKind::Username,
            FieldKind::Url,
            FieldKind::Email,
            FieldKind::Note,
        ] {
            let mut item = login("Untitled");
            item.push_field(
                Field::new("Field", "correct-horse-battery-staple", true).with_kind(kind),
            );
            assert!(
                scored(&item, "correct-horse-battery-staple").is_none(),
                "a secret {kind:?} field must not answer the query that guessed it"
            );
        }
    }

    /// A field match must not outrank a title match, however many words the field holds.
    ///
    /// The weights are what keeps a note body from deciding the order of a list whose rows are
    /// titles — now that every non-secret field is searched, they are the only thing that does.
    #[test]
    fn a_field_match_never_outranks_a_title_match() {
        let by_title = login("Renewal");
        let mut by_note = login("Untitled");
        by_note.push_field(
            Field::new("Note", "renewal in March, renewal again in April", false)
                .with_kind(FieldKind::Note),
        );
        assert!(scored(&by_title, "renewal").unwrap() > scored(&by_note, "renewal").unwrap());
    }

    #[test]
    fn an_empty_query_returns_the_first_items_in_vault_order() {
        let items = [login("One"), login("Two"), login("Three")];
        let ranked = rank(items.iter(), "   ", 2);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked.first().map(|item| item.title.as_str()), Some("One"));
        assert_eq!(ranked.get(1).map(|item| item.title.as_str()), Some("Two"));
    }

    #[test]
    fn ranking_orders_by_score_and_respects_the_limit() {
        let items = [login("Digital Ocean"), login("GitHub"), login("Git LFS")];
        let ranked = rank(items.iter(), "git", 2);
        assert_eq!(ranked.len(), 2);
        assert!(
            ranked
                .first()
                .is_some_and(|item| item.title.starts_with("Git")),
            "a prefix match leads"
        );
        assert!(
            !ranked.iter().any(|item| item.title == "Digital Ocean"),
            "the weakest of three matches is the one the limit drops"
        );
    }

    /// Equal scores must produce one order, not the order the input happened to be in.
    #[test]
    fn equal_scores_break_the_tie_the_same_way_every_time() {
        let forwards = [login("Alpha"), login("Alpha")];
        let backwards: Vec<Item> = forwards.iter().rev().cloned().collect();

        let one = rank(forwards.iter(), "alpha", 2);
        let two = rank(backwards.iter(), "alpha", 2);
        assert_eq!(
            one.iter().map(|item| item.id).collect::<Vec<_>>(),
            two.iter().map(|item| item.id).collect::<Vec<_>>(),
            "the tie-break is on the items, not on the order they arrived in"
        );
    }
}
