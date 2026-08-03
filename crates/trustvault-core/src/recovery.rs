//! The recovery kit: 120 bits of CSPRNG output, printable and hand-typable.
//!
//! Specified in `docs/vault-format.md` §5. It exists so that losing the master password is not
//! the same as losing the vault, and it is shown exactly once, at creation (R-07).

use core::fmt;

use data_encoding::BASE32_NOPAD;
use zeroize::Zeroizing;

use crate::aead::random;
use crate::{Error, Result};

/// Number of random bytes behind a recovery code — 120 bits.
const SECRET_LEN: usize = 15;
/// Number of base32 characters those bytes encode to.
const CODE_LEN: usize = 24;
/// Characters per printed group.
const GROUP_LEN: usize = 4;

/// A recovery code: six groups of four base32 characters.
///
/// The printed form is never hashed. The KDF input is always the 15 decoded bytes, which is
/// what makes dashes, spacing, and case irrelevant to whether a transcribed code works.
#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryCode(Zeroizing<[u8; SECRET_LEN]>);

impl RecoveryCode {
    /// Draws a fresh code from the OS CSPRNG.
    pub fn generate() -> Result<Self> {
        Ok(Self(Zeroizing::new(random::<SECRET_LEN>()?)))
    }

    /// Parses a code as the user typed it.
    ///
    /// Anything outside the base32 alphabet is discarded first, so `abcd-efgh …`,
    /// `ABCD EFGH …`, and a copy-pasted block with line breaks all parse identically.
    ///
    /// The four digits the alphabet excludes — `0`, `1`, `8`, `9` — are *not* mapped onto
    /// their look-alikes. A code containing them is a transcription error and is rejected as
    /// one. Silently reading `0` as `O` would mean two different printed kits open the same
    /// vault, which quietly widens the thing this code is protecting.
    pub fn parse(input: &str) -> Result<Self> {
        let mut filtered = Zeroizing::new(String::with_capacity(CODE_LEN));
        for ch in input.chars() {
            let upper = ch.to_ascii_uppercase();
            if upper.is_ascii_whitespace() || upper == '-' || upper == '_' {
                continue;
            }
            if !BASE32_NOPAD.specification().symbols.contains(upper) {
                return Err(Error::MalformedRecoveryCode);
            }
            filtered.push(upper);
        }

        if filtered.len() != CODE_LEN {
            return Err(Error::MalformedRecoveryCode);
        }

        let decoded = BASE32_NOPAD
            .decode(filtered.as_bytes())
            .map_err(|_| Error::MalformedRecoveryCode)?;
        let bytes = <[u8; SECRET_LEN]>::try_from(decoded.as_slice())
            .map_err(|_| Error::MalformedRecoveryCode)?;
        Ok(Self(Zeroizing::new(bytes)))
    }

    /// The printable form: `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`.
    ///
    /// Returns a `Zeroizing<String>` because this is the one moment the secret exists as text.
    /// The caller renders it, the user writes it down, and the buffer is wiped on drop.
    pub fn display(&self) -> Zeroizing<String> {
        let encoded = Zeroizing::new(BASE32_NOPAD.encode(self.0.as_slice()));
        let mut out = String::with_capacity(CODE_LEN + CODE_LEN / GROUP_LEN - 1);
        for (index, ch) in encoded.chars().enumerate() {
            if index > 0 && index % GROUP_LEN == 0 {
                out.push('-');
            }
            out.push(ch);
        }
        Zeroizing::new(out)
    }

    /// The 15 bytes fed to the KDF.
    pub(crate) fn secret(&self) -> &[u8; SECRET_LEN] {
        &self.0
    }
}

impl fmt::Debug for RecoveryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A recovery code printed into a log is a vault handed over. See `SecretString`.
        f.write_str("RecoveryCode(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printed_form_is_six_groups_of_four() {
        let code = RecoveryCode::generate().unwrap();
        let printed = code.display();
        let groups: Vec<&str> = printed.split('-').collect();
        assert_eq!(groups.len(), 6);
        assert!(groups.iter().all(|group| group.len() == GROUP_LEN));
        assert!(
            printed
                .chars()
                .all(|ch| ch == '-' || ch.is_ascii_uppercase() || ('2'..='7').contains(&ch))
        );
    }

    #[test]
    fn the_alphabet_excludes_the_ambiguous_digits() {
        // The reason for base32 rather than base58 or hex: a printed secret is typed by hand,
        // and 0/O and 1/l/I are where that goes wrong.
        let symbols = BASE32_NOPAD.specification().symbols.clone();
        for ch in ['0', '1', '8', '9'] {
            assert!(!symbols.contains(ch), "{ch} must not be in the alphabet");
        }
    }

    #[test]
    fn round_trips_through_the_printed_form() {
        let code = RecoveryCode::generate().unwrap();
        assert_eq!(RecoveryCode::parse(&code.display()).unwrap(), code);
    }

    #[test]
    fn formatting_does_not_change_the_secret() {
        let code = RecoveryCode::generate().unwrap();
        let printed = code.display().to_string();
        let variants = [
            printed.clone(),
            printed.replace('-', ""),
            printed.replace('-', " "),
            printed.to_lowercase(),
            format!("  {}\n", printed.replace('-', "\n")),
            printed.replace('-', "_"),
        ];
        for variant in variants {
            assert_eq!(
                RecoveryCode::parse(&variant).unwrap(),
                code,
                "variant {variant:?} should parse to the same secret"
            );
        }
    }

    #[test]
    fn transcription_errors_are_rejected_not_guessed() {
        let code = RecoveryCode::generate().unwrap();
        let printed = code.display().to_string();

        // A digit outside the alphabet is a mistake, and mapping it would let two different
        // printed kits open one vault.
        for ch in ['0', '1', '8', '9'] {
            let mut broken = printed.clone();
            broken.replace_range(0..1, &ch.to_string());
            assert!(matches!(
                RecoveryCode::parse(&broken),
                Err(Error::MalformedRecoveryCode)
            ));
        }

        // Wrong length, in both directions, and empty.
        assert!(RecoveryCode::parse("").is_err());
        assert!(RecoveryCode::parse(&printed[..printed.len() - 1]).is_err());
        assert!(RecoveryCode::parse(&format!("{printed}A")).is_err());
    }

    #[test]
    fn two_generated_codes_differ() {
        let a = RecoveryCode::generate().unwrap();
        let b = RecoveryCode::generate().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.secret().len(), 15);
    }

    #[test]
    fn debug_never_prints_the_code() {
        let code = RecoveryCode::generate().unwrap();
        let printed = code.display().to_string();
        let debug = format!("{code:?}");
        assert_eq!(debug, "RecoveryCode(***)");
        assert!(!debug.contains(printed.get(..4).unwrap_or_default()));
    }
}
