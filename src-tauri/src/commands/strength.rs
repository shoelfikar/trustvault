//! Password strength scoring for onboarding's meter — R-08, D-12.
//!
//! **The scoring itself moved to `trustvault-core::watchtower` on 2026-08-15, D-78.** This file
//! is now the command and nothing else. The reason for the move is not the reason this file
//! existed: it lives in the host rather than in the webview because zxcvbn's dictionaries are
//! ~400 kB and S-01 budgets 800 ms for cold start, and that argument is untouched — the
//! dictionaries are still in the Rust binary and still never parsed by the frontend. What moved
//! and why is in `watchtower.rs`'s own header.
//!
//! Nothing here re-derives a score, a label, or a threshold. Two definitions of what "Weak"
//! means agree until the day one of them is edited, which is D-44's argument about the two
//! password generators arriving one phase later.

use trustvault_core::Strength;

use crate::error::IpcResult;

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
    let context: Vec<&str> = inputs.iter().map(String::as_str).collect();
    Ok(trustvault_core::score(&password, &context))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The command is a pass-through, and this is the test that says so.
    ///
    /// The scoring behaviour itself is pinned in `trustvault-core::watchtower` — including the
    /// measured contradiction of D-12's own example, which travelled with the code rather than
    /// being left behind pointing at a function that is no longer here.
    #[test]
    fn the_command_hands_its_context_to_the_core_unchanged() {
        let blind = score_password("Jakarta2019!".to_owned(), Vec::new()).expect("never fails");
        let informed = score_password("Jakarta2019!".to_owned(), vec!["Jakarta".to_owned()])
            .expect("never fails");
        assert!(
            informed.score < blind.score,
            "the inputs have to reach zxcvbn, or this command drops the argument silently"
        );
    }

    /// The meter draws a word beside every score — `MASTER.md` §2, status is never colour alone.
    #[test]
    fn a_scored_password_comes_back_with_its_word() {
        let strength: Strength =
            score_password("correct horse battery staple".to_owned(), Vec::new())
                .expect("never fails");
        assert!(!strength.label.is_empty());
        assert!(!strength.crack_time.is_empty());
    }
}
