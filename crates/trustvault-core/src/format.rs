//! The `.tvault` header: 228 fixed bytes, little-endian, no varints.
//!
//! The byte layout is specified in `docs/vault-format.md` §2 and that document is
//! authoritative. This module is its implementation, written second on purpose.

use crate::aead::{NONCE_LEN, WRAP_LEN};
use crate::kdf::KdfParams;
use crate::{AEAD_ID_XCHACHA20POLY1305, Error, FORMAT_VERSION, KDF_ID_ARGON2ID, MAGIC, Result};

/// Length of the salts, in bytes.
pub(crate) const SALT_LEN: usize = 16;

/// Total header length in bytes — everything before the sealed body.
pub const HEADER_LEN: usize = 228;

/// Length of the parameter block: magic, version, algorithm ids, and the Argon2id costs.
///
/// This is the associated data for the two key wraps (`docs/vault-format.md` §4). It stops at
/// the salts on purpose, and the reason is not obvious: the salts must **not** be shared AAD,
/// because rotating one of them would then invalidate the *other* wrap — and changing the
/// master password cannot re-wrap under the recovery code, which the user is not holding.
///
/// Nothing is lost by stopping here. A salt is already a KDF input, so editing it produces a
/// wrong key and the unwrap fails anyway; and both salts are covered by the body's tag, whose
/// associated data is the entire header.
pub const WRAP_AAD_LEN: usize = 20;

/// The parsed header of a vault file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Format version recorded in the file.
    pub version: u16,
    /// Argon2id cost parameters used for both key wraps.
    pub kdf: KdfParams,
    /// Salt for the password-derived key.
    pub salt_pw: [u8; SALT_LEN],
    /// Salt for the recovery-code-derived key.
    pub salt_rk: [u8; SALT_LEN],
    /// Nonce for the master key sealed under the password key.
    pub nonce_pw: [u8; NONCE_LEN],
    /// Master key sealed under the password key.
    pub wrap_pw: [u8; WRAP_LEN],
    /// Nonce for the master key sealed under the recovery key.
    pub nonce_rk: [u8; NONCE_LEN],
    /// Master key sealed under the recovery key.
    pub wrap_rk: [u8; WRAP_LEN],
    /// Nonce for the sealed body.
    pub nonce_body: [u8; NONCE_LEN],
    /// Length of the sealed body in bytes, tag included.
    pub body_len: u64,
}

impl Header {
    /// Serializes the header to its 228 bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut out = Vec::with_capacity(HEADER_LEN);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.version.to_le_bytes());
        out.push(KDF_ID_ARGON2ID);
        out.push(AEAD_ID_XCHACHA20POLY1305);
        out.extend_from_slice(&self.kdf.m_cost.to_le_bytes());
        out.extend_from_slice(&self.kdf.t_cost.to_le_bytes());
        out.push(self.kdf.p_cost);
        out.extend_from_slice(&[0u8; 3]); // reserved, written zero, ignored on read
        debug_assert_eq!(out.len(), WRAP_AAD_LEN);
        out.extend_from_slice(&self.salt_pw);
        out.extend_from_slice(&self.salt_rk);
        out.extend_from_slice(&self.nonce_pw);
        out.extend_from_slice(&self.wrap_pw);
        out.extend_from_slice(&self.nonce_rk);
        out.extend_from_slice(&self.wrap_rk);
        out.extend_from_slice(&self.nonce_body);
        out.extend_from_slice(&self.body_len.to_le_bytes());

        // `out` is built from fixed-size pieces that sum to HEADER_LEN, so the conversion
        // cannot fail. Falling back to zeroes rather than unwrapping keeps the Tier-1 promise
        // that this crate contains no panic: a zero header fails to parse, loudly.
        <[u8; HEADER_LEN]>::try_from(out.as_slice()).unwrap_or([0u8; HEADER_LEN])
    }

    /// Parses a header from the start of `bytes`.
    ///
    /// Performs steps 1–3 of `docs/vault-format.md` §7: the checks that decide whether this is
    /// one of our files at all. They happen before any key material exists, so reporting them
    /// precisely leaks nothing — unlike the wrong-password/corrupt-file distinction, which
    /// R-03 forbids.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        if cursor.take(4) != Some(MAGIC.as_slice()) {
            return Err(Error::NotAVault);
        }

        let version = cursor.u16().ok_or(Error::NotAVault)?;
        if version > FORMAT_VERSION {
            return Err(Error::UnsupportedVersion {
                found: version,
                supported: FORMAT_VERSION,
            });
        }
        if version == 0 {
            return Err(Error::NotAVault);
        }

        let kdf_id = cursor.u8().ok_or(Error::NotAVault)?;
        let aead_id = cursor.u8().ok_or(Error::NotAVault)?;
        if kdf_id != KDF_ID_ARGON2ID || aead_id != AEAD_ID_XCHACHA20POLY1305 {
            return Err(Error::NotAVault);
        }

        let kdf = KdfParams {
            m_cost: cursor.u32().ok_or(Error::NotAVault)?,
            t_cost: cursor.u32().ok_or(Error::NotAVault)?,
            p_cost: cursor.u8().ok_or(Error::NotAVault)?,
        };
        // Bounds first, allocation later (§3.3): a header claiming 4 TiB of memory must not
        // get as far as asking for it.
        kdf.validate().map_err(|_| Error::NotAVault)?;

        let _reserved = cursor.take(3).ok_or(Error::NotAVault)?;

        let header = Self {
            version,
            kdf,
            salt_pw: cursor.array().ok_or(Error::NotAVault)?,
            salt_rk: cursor.array().ok_or(Error::NotAVault)?,
            nonce_pw: cursor.array().ok_or(Error::NotAVault)?,
            wrap_pw: cursor.array().ok_or(Error::NotAVault)?,
            nonce_rk: cursor.array().ok_or(Error::NotAVault)?,
            wrap_rk: cursor.array().ok_or(Error::NotAVault)?,
            nonce_body: cursor.array().ok_or(Error::NotAVault)?,
            body_len: cursor.u64().ok_or(Error::NotAVault)?,
        };

        Ok(header)
    }
}

/// A forward-only reader over a byte slice.
///
/// Exists so that the parser contains no indexing and no slicing: every read is bounds-checked
/// and returns `None` past the end, which is what turns a truncated file into `NotAVault`
/// instead of a panic. The core is Tier 1 — a panic here is a crash on a file the user was
/// trying to open.
struct Cursor<'a> {
    rest: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let (head, tail) = self.rest.split_at_checked(n)?;
        self.rest = tail;
        Some(head)
    }

    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        <[u8; N]>::try_from(self.take(N)?).ok()
    }

    fn u8(&mut self) -> Option<u8> {
        self.array::<1>().map(|b| b[0])
    }

    fn u16(&mut self) -> Option<u16> {
        self.array().map(u16::from_le_bytes)
    }

    fn u32(&mut self) -> Option<u32> {
        self.array().map(u32::from_le_bytes)
    }

    fn u64(&mut self) -> Option<u64> {
        self.array().map(u64::from_le_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Header {
        Header {
            version: FORMAT_VERSION,
            kdf: KdfParams::TESTING,
            salt_pw: [1; SALT_LEN],
            salt_rk: [2; SALT_LEN],
            nonce_pw: [3; NONCE_LEN],
            wrap_pw: [4; WRAP_LEN],
            nonce_rk: [5; NONCE_LEN],
            wrap_rk: [6; WRAP_LEN],
            nonce_body: [7; NONCE_LEN],
            body_len: 4242,
        }
    }

    #[test]
    fn layout_matches_the_specification() {
        // These offsets are the table in docs/vault-format.md §2. A change here without a
        // change there is how the document stops being a specification.
        let bytes = sample().to_bytes();
        assert_eq!(bytes.len(), 228);
        assert_eq!(&bytes[0..4], b"TVLT");
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 1);
        assert_eq!(bytes[6], KDF_ID_ARGON2ID);
        assert_eq!(bytes[7], AEAD_ID_XCHACHA20POLY1305);
        assert_eq!(
            &bytes[17..20],
            &[0, 0, 0],
            "reserved bytes are written zero"
        );
        assert_eq!(&bytes[20..36], &[1u8; 16]);
        assert_eq!(&bytes[36..52], &[2u8; 16]);
        assert_eq!(&bytes[52..76], &[3u8; 24]);
        assert_eq!(&bytes[76..124], &[4u8; 48]);
        assert_eq!(&bytes[124..148], &[5u8; 24]);
        assert_eq!(&bytes[148..196], &[6u8; 48]);
        assert_eq!(&bytes[196..220], &[7u8; 24]);
        assert_eq!(
            u64::from_le_bytes(bytes[220..228].try_into().unwrap()),
            4242
        );
    }

    #[test]
    fn round_trips() {
        let header = sample();
        assert_eq!(Header::parse(&header.to_bytes()).unwrap(), header);
    }

    #[test]
    fn trailing_bytes_after_the_header_are_ignored_by_the_parser() {
        // The parser reads a prefix; the *file* length check belongs to Vault::open, which is
        // where the body length is known.
        let mut bytes = sample().to_bytes().to_vec();
        bytes.extend_from_slice(b"body would go here");
        assert_eq!(Header::parse(&bytes).unwrap(), sample());
    }

    #[test]
    fn reserved_bytes_are_ignored_on_read() {
        // §2: written zero, but a non-zero value is not a rejection — that is the extension
        // point a future flag uses without a format break.
        let mut bytes = sample().to_bytes();
        bytes[17] = 0xFF;
        assert_eq!(Header::parse(&bytes).unwrap(), sample());
    }

    #[test]
    fn a_short_file_is_not_a_vault_rather_than_a_panic() {
        let bytes = sample().to_bytes();
        for len in 0..HEADER_LEN {
            assert!(
                matches!(Header::parse(&bytes[..len]), Err(Error::NotAVault)),
                "truncated to {len} bytes should parse as NotAVault"
            );
        }
    }

    #[test]
    fn foreign_files_are_rejected() {
        assert!(matches!(
            Header::parse(b"not a vault at all"),
            Err(Error::NotAVault)
        ));
        assert!(matches!(
            Header::parse(&[0u8; HEADER_LEN]),
            Err(Error::NotAVault)
        ));
    }

    #[test]
    fn a_newer_version_says_so_instead_of_lying() {
        let mut bytes = sample().to_bytes();
        bytes[4] = 2;
        assert!(matches!(
            Header::parse(&bytes),
            Err(Error::UnsupportedVersion {
                found: 2,
                supported: 1
            })
        ));

        // Version 0 has never existed, so it is a malformed file, not a future one.
        bytes[4] = 0;
        assert!(matches!(Header::parse(&bytes), Err(Error::NotAVault)));
    }

    #[test]
    fn unknown_algorithm_ids_are_rejected() {
        for offset in [6usize, 7] {
            let mut bytes = sample().to_bytes();
            bytes[offset] = 2;
            assert!(matches!(Header::parse(&bytes), Err(Error::NotAVault)));
        }
    }

    #[test]
    fn absurd_kdf_parameters_are_rejected_before_allocation() {
        let mut header = sample();
        header.kdf.m_cost = KdfParams::MAX_M_COST + 1;
        assert!(matches!(
            Header::parse(&header.to_bytes()),
            Err(Error::NotAVault)
        ));

        header.kdf = KdfParams {
            m_cost: 8,
            t_cost: 0,
            p_cost: 1,
        };
        assert!(matches!(
            Header::parse(&header.to_bytes()),
            Err(Error::NotAVault)
        ));
    }
}
