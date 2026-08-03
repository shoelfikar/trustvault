//! XChaCha20-Poly1305 sealing, and the only source of randomness in the crate.
//!
//! Specified in `docs/vault-format.md` §4. Three seals exist — two key wraps and the body —
//! and they differ only in key, nonce, and associated data.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

use crate::kdf::Key;
use crate::{Error, Result};

/// Nonce length for XChaCha20-Poly1305, in bytes.
pub(crate) const NONCE_LEN: usize = 24;
/// Poly1305 tag length, in bytes.
pub(crate) const TAG_LEN: usize = 16;
/// A sealed 32-byte key: the key itself plus its tag.
pub(crate) const WRAP_LEN: usize = 32 + TAG_LEN;

/// Fills an array from the operating system's CSPRNG.
///
/// The **only** place this crate produces randomness, and it goes straight to the OS. There is
/// deliberately no seeded generator anywhere in the crate: R-06 forbids counter-derived nonces,
/// and the reliable way to keep that true is to have nothing available that could become a
/// counter. If the OS refuses, the operation fails — it does not fall back.
pub(crate) fn random<const N: usize>() -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).map_err(|_| Error::Entropy)?;
    Ok(buf)
}

/// Seals `plaintext` with `key` and `nonce`, authenticating `aad` alongside it.
///
/// The output is `ciphertext || tag`, so it is 16 bytes longer than the plaintext.
pub(crate) fn seal(
    key: &Key,
    nonce: &[u8; NONCE_LEN],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(&(*key.expose()).into());
    cipher
        .encrypt(
            &XNonce::from(*nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        // Encryption fails only on a length the format cannot produce; there is nothing
        // actionable to report and nothing to distinguish.
        .map_err(|_| Error::Unreadable)
}

/// Opens a sealed message, or fails.
///
/// Every failure — wrong key, wrong nonce, wrong associated data, a flipped byte anywhere —
/// is the same [`Error::Unreadable`]. That is R-03 and R-04 in one line: the caller cannot
/// learn *which* of those went wrong, because knowing is exactly what an attacker holding a
/// stolen vault wants.
pub(crate) fn open(
    key: &Key,
    nonce: &[u8; NONCE_LEN],
    sealed: &[u8],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>> {
    let cipher = XChaCha20Poly1305::new(&(*key.expose()).into());
    cipher
        .decrypt(&XNonce::from(*nonce), Payload { msg: sealed, aad })
        .map(Zeroizing::new)
        .map_err(|_| Error::Unreadable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret::SecretBytes;

    fn key(byte: u8) -> Key {
        SecretBytes::new([byte; 32])
    }

    #[test]
    fn round_trips_with_matching_key_nonce_and_aad() {
        let sealed = seal(&key(1), &[2u8; NONCE_LEN], b"plaintext", b"header").unwrap();
        assert_eq!(sealed.len(), b"plaintext".len() + TAG_LEN);

        let opened = open(&key(1), &[2u8; NONCE_LEN], &sealed, b"header").unwrap();
        assert_eq!(opened.as_slice(), b"plaintext");
    }

    #[test]
    fn every_input_is_authenticated() {
        let sealed = seal(&key(1), &[2u8; NONCE_LEN], b"plaintext", b"header").unwrap();

        assert!(open(&key(9), &[2u8; NONCE_LEN], &sealed, b"header").is_err());
        assert!(open(&key(1), &[9u8; NONCE_LEN], &sealed, b"header").is_err());
        assert!(open(&key(1), &[2u8; NONCE_LEN], &sealed, b"HEADER").is_err());

        let mut damaged = sealed.clone();
        damaged[0] ^= 1;
        assert!(open(&key(1), &[2u8; NONCE_LEN], &damaged, b"header").is_err());

        // A truncated message: the tag is gone, so there is nothing to verify against.
        assert!(open(&key(1), &[2u8; NONCE_LEN], &sealed[..4], b"header").is_err());
    }

    #[test]
    fn randomness_differs_between_draws() {
        // Not a randomness test — a wiring test. It catches the case where the CSPRNG call is
        // replaced by something that returns a constant, which is the mistake that matters.
        let a: [u8; 24] = random().unwrap();
        let b: [u8; 24] = random().unwrap();
        assert_ne!(a, b);
        assert_ne!(a, [0u8; 24]);
    }
}
