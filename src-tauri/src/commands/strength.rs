//! Password strength scoring for onboarding's meter — R-08, D-12.
//!
//! Lives in the host, not the webview, for one measured reason: zxcvbn's frequency
//! dictionaries are a few hundred kilobytes, and S-01 budgets 800 ms for cold start to an
//! interactive lock screen. A dictionary the frontend has to parse before it can draw is paid
//! for on every launch, to serve a screen the user sees once.

use serde::Serialize;

use crate::error::IpcResult;

/// A score and the words that go with it.
#[derive(Debug, Clone, Serialize)]
pub struct Strength {
    /// zxcvbn's 0–4 score, which drives the four meter segments.
    pub score: u8,
    /// Weak / Fair / Strong / Excellent — `MASTER.md` §2.
    ///
    /// Status is never colour alone, so the meter always renders this word beside the
    /// segments. The colour is a second channel, not the only one.
    pub label: &'static str,
    /// Human crack-time estimate, in zxcvbn's own phrasing.
    pub crack_time: String,
}

/// **Ambient.** Scores a password for the strength meter.
///
/// The password crosses inbound, which is unavoidable — the user typed it into the webview, so
/// it is already in a heap that cannot be wiped. It is never returned, stored, or logged.
///
/// `inputs` is the context the caller knows about: at onboarding, the vault name. zxcvbn
/// penalizes a password built out of words the user is surrounded by, and it cannot know what
/// those are unless it is told. Passing them is what stops "PersonalVault2026" scoring well on
/// a vault called Personal.
#[tauri::command(rename_all = "snake_case")]
pub fn score_password(password: String, inputs: Vec<String>) -> IpcResult<Strength> {
    Ok(score(&password, &inputs))
}

/// The scoring itself, split out so it is testable without a Tauri runtime.
pub fn score(password: &str, inputs: &[String]) -> Strength {
    // An empty box is not a weak password, it is no password. Scoring it as 0/"Weak" would
    // paint the meter red before the user has typed anything, which reads as a failure state.
    if password.is_empty() {
        return Strength {
            score: 0,
            label: "",
            crack_time: String::new(),
        };
    }

    let context: Vec<&str> = inputs.iter().map(String::as_str).collect();
    let entropy = zxcvbn::zxcvbn(password, &context);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// No context, the common case for a test.
    const NONE: &[String] = &[];

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

    /// **Measured, and it contradicts D-12's example.**
    ///
    /// D-12 justified zxcvbn with `Jakarta2019!` — the password entropy-only scoring calls
    /// strong and zxcvbn supposedly catches. It does not: with the default dictionaries it
    /// scores **3 / Strong**, because "Jakarta" is not in zxcvbn's English-centric frequency
    /// lists. The decision stands, but its example was aspirational rather than measured, and
    /// this test exists so nobody re-derives the claim from the decision log and believes it.
    ///
    /// What actually closes the gap is `inputs`: told the vault is called "Jakarta", zxcvbn
    /// scores the same password far lower. That is why the parameter exists.
    #[test]
    fn a_local_word_is_only_caught_when_zxcvbn_is_told_about_it() {
        let blind = score("Jakarta2019!", NONE);
        assert_eq!(
            blind.score, 3,
            "the default dictionaries do not know the city"
        );

        let informed = score("Jakarta2019!", &["Jakarta".to_owned()]);
        assert!(
            informed.score < blind.score,
            "context has to change the answer, or passing it is theatre"
        );
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
}
