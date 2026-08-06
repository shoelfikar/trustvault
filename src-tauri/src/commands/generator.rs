//! The password generator and its clipboard path — `docs/ipc-contract.md` §5 and §7, D-44.
//!
//! Two commands that look like one, and the split between them is the whole of D-37's
//! constraint being honoured rather than repealed:
//!
//! * [`generate_password`] is the **fourth sanctioned command**. It returns the value, because
//!   the surface the design draws shows the password with a regenerate button beside it — a
//!   generator whose output can only be pasted makes the length slider and the set chips into
//!   theatre, and it cannot fill the New-item dialog's password field at all.
//! * [`copy_generated`] takes a not-yet-stored password **inbound** and returns nothing. It is
//!   the one thing the webview cannot do for itself: a clipboard clear scheduled in Rust,
//!   which is exactly what D-37 said was missing when it closed the generator's copy paths.
//!
//! Neither command reads or writes the vault, so neither is vault-class and neither is refused
//! while locked. That is deliberate and it is safe for the same reason in both directions: a
//! password being generated protects nothing yet, and a password arriving here is one the
//! caller already holds.

use tauri::State;
use trustvault_core::{CharSets, PasswordRecipe, SecretString};

use crate::clipboard;
use crate::commands::items::clear_after;
use crate::commands::strength::{Strength, score};
use crate::dto::Copied;
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, now_ms};

/// What `generate_password` returns: one password and the meter that goes with it.
///
/// The score rides along rather than taking a second call, and that is a safety property
/// rather than a convenience — routing the generated value back through `score_password`
/// would send it across the boundary a second time, inbound, for a number this side already
/// has the inputs for.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Generated {
    /// The plaintext. The one `Secret` this response may carry (R-10).
    pub password: String,
    /// Score, word, and crack time — flattened, so the wire shape is the flat object
    /// `docs/ipc-contract.md` §7 prints rather than a nested one.
    #[serde(flatten)]
    pub strength: Strength,
}

/// **Sanctioned.** Mints one password and scores it — R-15, D-44.
///
/// The fourth command in the application permitted to return plaintext. What it buys is that
/// randomness for stored credentials moves onto the one path R-06 and D-23 already constrain:
/// `getrandom`, called from a single file in the core, with a CI grep forbidding any seedable
/// RNG in the crate.
///
/// A request outside R-15's bounds — a length below 8 or above 64, or every character class
/// off — is `internal` rather than a user-facing error. The only caller is our own webview,
/// whose slider is clamped and whose chips keep one class on, so a request in that shape is a
/// bug here and not something a user did.
#[tauri::command(rename_all = "snake_case")]
pub fn generate_password(
    length: usize,
    sets: CharSets,
    exclude_ambiguous: bool,
) -> IpcResult<Generated> {
    generate_password_inner(length, sets, exclude_ambiguous)
}

/// The body of [`generate_password`], reachable without a Tauri runtime.
pub fn generate_password_inner(
    length: usize,
    sets: CharSets,
    exclude_ambiguous: bool,
) -> IpcResult<Generated> {
    let recipe = PasswordRecipe::new(length, sets, exclude_ambiguous)
        .ok_or_else(|| IpcError::new(ErrorKind::Internal))?;
    let password = recipe.generate()?;
    // Scored with no context: there is no vault name to pass, because there may be no vault
    // open, and a generated password is not built out of words the user is surrounded by.
    let strength = score(password.expose(), &[]);
    Ok(Generated {
        password: password.expose().to_owned(),
        strength,
    })
}

/// **Ambient.** Copies a not-yet-stored password and schedules the clear — D-37, D-44.
///
/// The command that looks alarming and is not: it takes a secret inbound, returns none, and
/// reads nothing, so it cannot disclose anything the caller did not already supply. The string
/// it copies is one the webview is already holding — minted by [`generate_password`] moments
/// ago, or typed into the New-item dialog by hand.
///
/// It MUST NOT log, store, or retain the value beyond the clipboard write: a host-side "last
/// generated password" would be a lock-state hole with no lock around it. The value is moved
/// into a [`SecretString`] on arrival so that the host's own copy is zeroized when this
/// function returns, and nothing else here keeps a reference to it.
#[tauri::command(rename_all = "snake_case")]
pub fn copy_generated(state: State<'_, AppState>, password: String) -> IpcResult<Copied> {
    let (copied, seconds) = copy_generated_inner(&state, password)?;
    schedule_clear(seconds);
    Ok(copied)
}

/// The body of [`copy_generated`], minus the timer, reachable without a Tauri runtime.
pub fn copy_generated_inner(state: &AppState, password: String) -> IpcResult<(Copied, u64)> {
    // Moved, not copied: `From<String>` takes the allocation the webview's value arrived in,
    // so there is one buffer and `SecretString` zeroizes it on the way out of this function.
    let password = SecretString::from(password);
    state.touch();
    clipboard::set(password.expose())?;

    // The same interval `copy_field` uses, read from the same place. Two clipboard timers with
    // different durations is how one of them ends up wrong.
    let seconds = state
        .with(|inner| inner.settings.clipboard_clear_seconds)
        .unwrap_or_default();
    let clears_at = now_ms() + i64::try_from(seconds * 1000).unwrap_or(0);
    Ok((Copied { clears_at }, seconds))
}

/// Clears the clipboard after the configured window.
///
/// **No event follows**, unlike `copy_field`'s clear, which is why this takes no `AppHandle`.
/// `clipboard-cleared` names an item and a field (§8) and this password belongs to neither —
/// it is not in the vault and may never be. The response's `clears_at` is what the chip counts
/// down from, which is all the surface needs; inventing an event with null identifiers to
/// reuse a listener would put a shape on the boundary that means nothing.
///
/// The clear itself is not gated on anything. A secret on the clipboard is cleared whether or
/// not a vault has since locked — locking is a reason to clear sooner, never a reason to skip.
fn schedule_clear(seconds: u64) {
    std::thread::spawn(move || clear_after(seconds));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_password_carries_its_score_in_one_flat_object() {
        let generated = generate_password_inner(20, CharSets::ALL, true).expect("a valid recipe");
        assert_eq!(generated.password.chars().count(), 20);

        let encoded = serde_json::to_string(&generated).expect("serializable");
        // The shape docs/ipc-contract.md §7 prints: four keys, not a nested strength object.
        for key in ["password", "score", "label", "crack_time"] {
            assert!(encoded.contains(key), "missing {key} in {encoded}");
        }
        assert!(!encoded.contains("strength"), "flattened, not nested");
    }

    #[test]
    fn exactly_one_secret_crosses_per_invocation() {
        // R-10 for the fourth sanctioned command. The failure this guards against is a
        // response that helpfully echoes the value somewhere else too — in a "preview" field,
        // or in an error message built from the input.
        let generated = generate_password_inner(32, CharSets::ALL, true).expect("a valid recipe");
        let encoded = serde_json::to_string(&generated).expect("serializable");
        assert_eq!(encoded.matches(&generated.password).count(), 1);
    }

    #[test]
    fn a_request_outside_the_bounds_is_internal_and_not_a_short_password() {
        for (length, sets) in [
            (4, CharSets::ALL),
            (128, CharSets::ALL),
            (
                20,
                CharSets {
                    lowercase: false,
                    uppercase: false,
                    digits: false,
                    symbols: false,
                },
            ),
        ] {
            let error = generate_password_inner(length, sets, true)
                .err()
                .map(|error| error.kind);
            assert_eq!(error, Some(ErrorKind::Internal), "length {length}");
        }
    }

    #[test]
    fn the_generated_score_is_the_same_scorer_the_meter_uses() {
        // One scorer, so the number in the generator and the number under the New-item
        // dialog's password field cannot disagree about the same string.
        let generated = generate_password_inner(64, CharSets::ALL, true).expect("a valid recipe");
        assert_eq!(
            generated.strength.score,
            score(&generated.password, &[]).score
        );
        assert!(!generated.strength.label.is_empty(), "a word beside it");
    }
}
