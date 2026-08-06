//! Time-based one-time passwords — `docs/ipc-contract.md` §6.6 and §5, R-20, D-45.
//!
//! Two commands, and the split between them is the same one the generator has: one reads the
//! vault, one reads nothing.
//!
//! * [`totp_code`] is **vault-class**. It answers with a code for the selected item and never
//!   with the seed — the seed is a `secret: true` field and leaves, if ever, through
//!   `reveal_field` like every other secret. A command that returned both would be a
//!   sanctioned command pretending not to be one.
//! * [`totp_preview`] is **ambient**. It generates from a seed the user is currently typing
//!   into the Add dialog, where there is no item yet and so nothing to look up.
//!
//! Neither is sanctioned, and that is D-45 rather than an oversight: a TOTP code is not marked
//! `Secret`, because it is not the credential, it is single-use, and the protocol's own
//! operation is to type it into somebody else's form. The reasoning is in
//! `docs/ipc-contract.md` §7.1, next to the commands it is an exception to, and what it does
//! **not** license is stated there too — one item at a time, never a list.

use tauri::State;
use trustvault_core::{FieldKind, ItemId, SecretString, TotpSpec};

use crate::commands::with_vault;
use crate::error::{ErrorKind, IpcError, IpcResult};
use crate::state::{AppState, now_ms};

/// One code and what the countdown ring needs to draw itself.
///
/// `expires_at` is milliseconds since the epoch, like every other instant on this boundary,
/// and it is the **step** boundary rather than "now plus the period" — see
/// [`trustvault_core::TotpSpec::expires_at`]. A ring that started counting when the pane
/// opened would disagree with the phone lying beside the keyboard, and the user would believe
/// the phone.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TotpCode {
    /// The code, zero-padded to `digits`. Not a `Secret` — D-45.
    pub code: String,
    /// When this code stops being valid, in milliseconds since the Unix epoch.
    pub expires_at: i64,
    /// Seconds per step, so the ring knows what fraction has elapsed.
    pub period: u64,
    /// Digits per code, so the surface reserves the right width before the first code arrives.
    pub digits: u32,
}

impl TotpCode {
    /// Generates from a parsed spec at the current instant.
    fn now(spec: &TotpSpec) -> IpcResult<Self> {
        // One clock read, used for both, so the code and the expiry cannot come from opposite
        // sides of a step boundary — a one-in-a-thousand refresh that shows a code already
        // dead with a full ring next to it.
        let millis = now_ms();
        let seconds = u64::try_from(millis / 1000).unwrap_or(0);
        Ok(Self {
            code: spec.code_at(seconds)?,
            expires_at: i64::try_from(spec.expires_at(seconds).saturating_mul(1000))
                .unwrap_or(i64::MAX),
            period: spec.period(),
            digits: spec.digits(),
        })
    }
}

/// **Vault-class.** The current code for the selected item — R-20, §6.6.
#[tauri::command(rename_all = "snake_case")]
pub fn totp_code(state: State<'_, AppState>, item_id: ItemId) -> IpcResult<TotpCode> {
    totp_code_inner(&state, item_id)
}

/// The body of [`totp_code`], reachable without a Tauri runtime.
///
/// **Not audited**, unlike `reveal_field` and `copy_field`, and the reason is arithmetic rather
/// than principle: the detail pane refreshes this every step, so an entry per call would write
/// 120 entries an hour with the pane open and evict every genuine reveal from D-31's
/// 1000-entry cap before lunch. An audit log that is mostly its own noise is worse than none,
/// because it still looks complete. What is worth recording is the *seed* being revealed, and
/// that goes through `reveal_field`, which does record it.
pub fn totp_code_inner(state: &AppState, item_id: ItemId) -> IpcResult<TotpCode> {
    let spec = with_vault(state, |vault, _| {
        let item = vault
            .item(item_id)
            .ok_or_else(|| IpcError::new(ErrorKind::NoSuchItem))?;

        // Selected by `kind`, not by label and not by `secret`. D-53 is the warning here and
        // it points the other way for once: `secret` is the honest gate for *search*, because
        // it is what the user declared — but every password in the vault is secret too, so it
        // cannot pick out a seed. `kind` can, and both paths that write one set it
        // explicitly: the Add dialog's template (`itemFields.ts`) and the importer
        // (`import/bitwarden.rs`), neither of which lets `Field::new` guess.
        let field = item
            .fields
            .iter()
            .find(|field| field.kind == FieldKind::Otp && !field.value.is_empty())
            // `no_such_field` rather than an empty success, so the pane draws its ring from a
            // call that worked instead of from a guess about the item's type (§6.6). An item
            // whose seed field exists but is empty is an item with no TOTP, which is why the
            // emptiness is part of the search and not a separate error.
            .ok_or_else(|| IpcError::new(ErrorKind::NoSuchField))?;

        // Parsed inside the vault closure and dropped outside it: what leaves here is a
        // `TotpSpec`, which zeroizes its decoded seed and exposes no way to read it back.
        Ok(TotpSpec::parse(field.value.expose())?)
    })?;

    TotpCode::now(&spec)
}

/// **Ambient.** One code from a seed the user is still typing — R-20, §5.
///
/// It doubles as the seed's validator, which is the point of having it at all: a base32 string
/// that will not decode is caught while the field is on screen and the phone is still in the
/// user's hand, rather than a month later at a login prompt. A malformed seed answers
/// `malformed_totp_secret` (§4), which is safe to distinguish because it is decided before any
/// vault content is involved.
#[tauri::command(rename_all = "snake_case")]
pub fn totp_preview(state: State<'_, AppState>, secret: String) -> IpcResult<TotpCode> {
    totp_preview_inner(&state, secret)
}

/// The body of [`totp_preview`], reachable without a Tauri runtime.
pub fn totp_preview_inner(state: &AppState, secret: String) -> IpcResult<TotpCode> {
    // Moved, not copied, exactly as `copy_generated` does it: `From<String>` takes the
    // allocation the value arrived in, so the host's own copy of the seed is zeroized when
    // this function returns rather than left in a heap nothing wipes.
    let secret = SecretString::from(secret);
    // Typing a seed is real interaction, so it counts against the auto-lock clock. This does
    // not close the gap next-action 7 records — most of the New-item dialog still does not
    // touch anything — it just does not make it worse.
    state.touch();

    let spec = TotpSpec::parse(secret.expose())?;
    TotpCode::now(&spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trustvault_core::{ItemKind, KdfParams, Vault};

    /// The RFC 6238 seed, base32-encoded — `GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ`.
    const SEED: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    /// An unlocked state holding one login with a seed, and one with none.
    fn unlocked() -> (AppState, ItemId, ItemId) {
        let (mut vault, _recovery) =
            Vault::create("Test", "correct horse", KdfParams::TESTING).expect("a vault");

        let with_seed = vault.add_item(ItemKind::Login, "GitHub");
        if let Some(item) = vault.item_mut(with_seed) {
            let id = item.set_field("2FA secret", SEED, true);
            if let Some(field) = item.fields.iter_mut().find(|field| field.id == id) {
                // Explicitly, the way both real write paths do it — `set_field` would
                // otherwise guess `password` from `secret`, which is D-53's trap.
                field.kind = FieldKind::Otp;
            }
        }

        let without = vault.add_item(ItemKind::Login, "Tokopedia");
        if let Some(item) = vault.item_mut(without) {
            item.set_field("Password", "hunter2", true);
        }

        let state = AppState::default();
        state.with(|inner| inner.vault = Some(vault));
        (state, with_seed, without)
    }

    #[test]
    fn the_response_carries_a_code_and_no_seed() {
        // R-10 for this command, and the reason it is not sanctioned: the one value it
        // returns is not the credential. A response that helpfully echoed the seed — in a
        // "source" field, or in an error built from the input — would make it one.
        let (state, item_id, _) = unlocked();
        let response = totp_code_inner(&state, item_id).expect("an item with a seed");

        assert_eq!(response.code.len(), 6);
        assert!(response.code.chars().all(|ch| ch.is_ascii_digit()));
        assert_eq!(response.period, 30);
        assert_eq!(response.digits, 6);

        let encoded = serde_json::to_string(&response).expect("serializable");
        assert!(
            !encoded.contains(SEED),
            "the seed must not cross: {encoded}"
        );
        assert!(!encoded.to_uppercase().contains("GEZD"), "{encoded}");
    }

    #[test]
    fn an_item_without_a_seed_is_no_such_field_not_an_empty_code() {
        // What lets the detail pane draw the ring from a successful call rather than from a
        // guess about the item's type.
        let (state, _, without) = unlocked();
        assert_eq!(
            totp_code_inner(&state, without).err().map(|e| e.kind),
            Some(ErrorKind::NoSuchField)
        );
    }

    #[test]
    fn the_preview_needs_no_vault_and_refuses_a_bad_seed() {
        // Ambient in both directions: it answers with the vault locked, and it answers
        // `malformed_totp_secret` rather than something a user cannot act on.
        let state = AppState::default();
        let preview = totp_preview_inner(&state, SEED.to_owned()).expect("a valid seed");
        assert_eq!(preview.code.len(), 6);

        assert_eq!(
            totp_preview_inner(&state, "not base32!".to_owned())
                .err()
                .map(|e| e.kind),
            Some(ErrorKind::MalformedTotpSecret)
        );
    }

    #[test]
    fn the_code_and_its_expiry_come_from_one_clock_read() {
        // The failure this rules out is a refresh landing either side of a step boundary and
        // showing a dead code under a full ring, once in a few thousand refreshes — which is
        // exactly often enough to be reported as "sometimes the codes just don't work".
        let (state, item_id, _) = unlocked();
        let response = totp_code_inner(&state, item_id).expect("an item with a seed");

        let now = now_ms();
        assert!(response.expires_at > now, "already expired on arrival");
        let period_ms = i64::try_from(response.period * 1000).unwrap_or(i64::MAX);
        assert!(
            response.expires_at - now <= period_ms,
            "more than one step away: {} vs {period_ms}",
            response.expires_at - now
        );
    }

    #[test]
    fn a_locked_vault_answers_locked() {
        let state = AppState::default();
        assert_eq!(
            totp_code_inner(&state, ItemId::new_v4())
                .err()
                .map(|e| e.kind),
            Some(ErrorKind::Locked)
        );
    }
}
