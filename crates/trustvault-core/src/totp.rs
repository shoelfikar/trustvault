//! Time-based one-time passwords — RFC 6238, requirement R-20.
//!
//! Two halves, and the second is the one that would have gone wrong quietly.
//!
//! The **generator** is RFC 4226's truncation over an HMAC of the time step. Forty lines, and
//! every one of them is pinned by the vectors the RFC publishes itself ([`tests`] below), which
//! is why D-54 took the primitives from RustCrypto rather than a TOTP crate: there is no
//! judgement in this code for a library to have exercised better, only arithmetic upstream
//! already numbered.
//!
//! The **parser** is where the failures live. A seed reaches this module in whatever form the
//! user's previous manager wrote it, and `import/bitwarden.rs` already stores both shapes
//! Bitwarden emits — a bare base32 seed, or a whole `otpauth://` URI. A generator that read
//! only the first would compute codes from the *letters of a URL* for every imported item and
//! report success while doing it, which is a wrong six-digit number at a login prompt with
//! nothing on screen to explain it. So:
//!
//! * `otpauth://totp/…` is parsed, including its `digits`, `period` and `algorithm` parameters.
//!   Ignoring those would be the same failure one level down — a SHA-256 seed generates
//!   perfectly formed, permanently rejected codes.
//! * `otpauth://hotp/…` is **refused**, not treated as TOTP. HOTP counts logins, not seconds;
//!   reading one as the other produces a code that is wrong in a way no user can diagnose.
//! * Anything that will not base32-decode is refused *while the user is looking at the field*,
//!   which is what `totp_preview` is for — the alternative is discovering it a month later at a
//!   login prompt with the phone already wiped.
//!
//! # What crosses the boundary
//!
//! The **seed** is a secret and stays one: it lives in a `secret: true` field, is elided from
//! every list, and leaves through `reveal_field` like any other. The **code** is not marked
//! `Secret` — D-45, and the reasoning is in `docs/ipc-contract.md` §7.1 rather than repeated
//! here.

use core::fmt;

use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use zeroize::Zeroizing;

use crate::{Error, Result};

/// Digits per code, as RFC 4226 §5.3 bounds them.
const MIN_DIGITS: u32 = 6;
/// Upper bound from the same section. Above 8 the truncation has no more bits to give.
const MAX_DIGITS: u32 = 8;
/// Longest step this module will accept, in seconds — one day.
///
/// There is no upper bound in the RFC. This one exists because the surface has to draw the
/// step as a countdown, and a ring that takes a day to move is not a countdown; a value that
/// large is a typo in someone's URI rather than a configuration anybody runs.
const MAX_PERIOD: u64 = 86_400;

/// The default step, and the only one in practice — RFC 6238 §5.2.
const DEFAULT_PERIOD: u64 = 30;
/// The default code length. Six, because that is what authenticator apps show.
const DEFAULT_DIGITS: u32 = 6;

/// Which HMAC sits under the code.
///
/// SHA-1 is the default and is not a weakness here: RFC 4226 Appendix B.2 is explicit that
/// HMAC-SHA-1 is unaffected by SHA-1's collision resistance, and the construction depends on
/// the MAC rather than on the hash being collision-free. The other two exist because
/// `otpauth://` URIs in the wild carry them, and reading one as SHA-1 would produce a code
/// that is wrong every single time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TotpAlgorithm {
    /// HMAC-SHA-1 — the default, and what almost every issuer uses.
    #[default]
    Sha1,
    /// HMAC-SHA-256.
    Sha256,
    /// HMAC-SHA-512.
    Sha512,
}

impl TotpAlgorithm {
    /// Parses the `algorithm` parameter of an `otpauth://` URI.
    ///
    /// Unknown values are refused rather than defaulted to SHA-1. Defaulting would turn "this
    /// issuer uses something we do not implement" into "every code is wrong", and only the
    /// first of those can be reported.
    fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_uppercase().as_str() {
            "SHA1" => Ok(Self::Sha1),
            "SHA256" => Ok(Self::Sha256),
            "SHA512" => Ok(Self::Sha512),
            _ => Err(Error::MalformedTotpSecret),
        }
    }

    /// The MAC of `counter` under `key`, wiped when the caller drops it.
    fn sign(self, key: &[u8], counter: [u8; 8]) -> Result<Zeroizing<Vec<u8>>> {
        /// One body for three digests, so the three arms cannot drift apart.
        fn mac<D: Mac + hmac::digest::KeyInit>(
            key: &[u8],
            counter: [u8; 8],
        ) -> Result<Zeroizing<Vec<u8>>> {
            // The only failure `new_from_slice` has is a key length the algorithm rejects, and
            // HMAC accepts every length. Reported rather than unwrapped anyway: this crate is
            // Tier 1 and `panic` is denied by lint, not by convention.
            let mut mac = D::new_from_slice(key).map_err(|_| Error::MalformedTotpSecret)?;
            mac.update(&counter);
            Ok(Zeroizing::new(mac.finalize().into_bytes().to_vec()))
        }

        match self {
            Self::Sha1 => mac::<Hmac<Sha1>>(key, counter),
            Self::Sha256 => mac::<Hmac<Sha256>>(key, counter),
            Self::Sha512 => mac::<Hmac<Sha512>>(key, counter),
        }
    }
}

/// A parsed TOTP seed and the parameters that go with it.
///
/// The seed is held in a `Zeroizing<Vec<u8>>`, so the decoded bytes are wiped when the spec
/// drops (N-01). It is never exposed: this type generates codes and gives up nothing else,
/// which is what keeps a command that returns a code from becoming one that returns a seed.
#[derive(Clone)]
pub struct TotpSpec {
    seed: Zeroizing<Vec<u8>>,
    algorithm: TotpAlgorithm,
    digits: u32,
    period: u64,
}

impl TotpSpec {
    /// Parses a stored seed, in either shape the world writes them.
    ///
    /// Accepts a bare base32 seed — with or without spaces, dashes, lowercase letters and `=`
    /// padding, all of which appear on real setup pages — or a whole `otpauth://totp/…` URI.
    ///
    /// # Errors
    ///
    /// [`Error::MalformedTotpSecret`] for anything that is not one of those two: an empty
    /// seed, a character outside the base32 alphabet, an `otpauth://hotp/…` URI, a `digits` or
    /// `period` outside what can be generated and drawn, or an algorithm this build has no
    /// implementation for. All of it is decided before any vault content is involved, which is
    /// why the contract lets this error be distinguishable (`docs/ipc-contract.md` §4).
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        if let Some(rest) = strip_prefix_ignore_case(trimmed, "otpauth://") {
            Self::parse_uri(rest)
        } else {
            Self::from_parts(trimmed, TotpAlgorithm::Sha1, DEFAULT_DIGITS, DEFAULT_PERIOD)
        }
    }

    /// Parses the part of an `otpauth://` URI after the scheme.
    fn parse_uri(rest: &str) -> Result<Self> {
        let (kind, remainder) = rest.split_once('/').ok_or(Error::MalformedTotpSecret)?;
        // Refused, not reinterpreted. HOTP's counter is a login count and this module has no
        // access to one; generating from the clock instead would answer with a code that is
        // wrong on every call, in a way that looks exactly like a code that is right.
        if !kind.eq_ignore_ascii_case("totp") {
            return Err(Error::MalformedTotpSecret);
        }

        let query = remainder
            .split_once('?')
            .map(|(_label, query)| query)
            .ok_or(Error::MalformedTotpSecret)?;

        let mut secret = None;
        let mut algorithm = TotpAlgorithm::Sha1;
        let mut digits = DEFAULT_DIGITS;
        let mut period = DEFAULT_PERIOD;

        for pair in query.split('&') {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            let value = percent_decode(value);
            match key.to_ascii_lowercase().as_str() {
                "secret" => secret = Some(value),
                "algorithm" => algorithm = TotpAlgorithm::parse(&value)?,
                "digits" => digits = value.parse().map_err(|_| Error::MalformedTotpSecret)?,
                "period" => period = value.parse().map_err(|_| Error::MalformedTotpSecret)?,
                // `issuer`, `image`, and whatever else an issuer decorates the URI with. They
                // describe the account, not the algorithm, so ignoring them changes no code.
                _ => {}
            }
        }

        let secret = secret.ok_or(Error::MalformedTotpSecret)?;
        Self::from_parts(&secret, algorithm, digits, period)
    }

    /// Validates the parameters and decodes the seed.
    ///
    /// Every bound here **refuses** rather than clamping, for the reason `PasswordRecipe::new`
    /// does: a clamped `digits=9` would hand back an 8-digit code while the surface that asked
    /// says nine, and the disagreement surfaces as a rejected login rather than as an error.
    fn from_parts(
        secret: &str,
        algorithm: TotpAlgorithm,
        digits: u32,
        period: u64,
    ) -> Result<Self> {
        if !(MIN_DIGITS..=MAX_DIGITS).contains(&digits) || period == 0 || period > MAX_PERIOD {
            return Err(Error::MalformedTotpSecret);
        }

        // Normalized the way `RecoveryCode::parse` normalizes: this is a string a human copied
        // off a setup page, so spacing, case and padding are formatting rather than content.
        // Unlike a recovery code, a character outside the alphabet is still a refusal — there
        // is no group separator convention here to be tolerant of.
        let mut filtered = Zeroizing::new(String::with_capacity(secret.len()));
        for ch in secret.chars() {
            if ch.is_ascii_whitespace() || ch == '-' || ch == '=' {
                continue;
            }
            let upper = ch.to_ascii_uppercase();
            if !BASE32_NOPAD.specification().symbols.contains(upper) {
                return Err(Error::MalformedTotpSecret);
            }
            filtered.push(upper);
        }

        let seed = BASE32_NOPAD
            .decode(filtered.as_bytes())
            .map_err(|_| Error::MalformedTotpSecret)?;
        // An empty seed decodes cleanly and generates a perfectly well-formed code that no
        // server will ever accept. Refused here, where it is still a field the user is typing.
        if seed.is_empty() {
            return Err(Error::MalformedTotpSecret);
        }

        Ok(Self {
            seed: Zeroizing::new(seed),
            algorithm,
            digits,
            period,
        })
    }

    /// The code for the step containing `unix_seconds` — RFC 4226 §5.3, RFC 6238 §4.
    ///
    /// Takes the time rather than reading the clock, which is what lets the RFC's own vectors
    /// drive this function instead of a re-implementation of it.
    ///
    /// # Errors
    ///
    /// [`Error::MalformedTotpSecret`] if the MAC cannot be computed, which the parser has
    /// already ruled out.
    pub fn code_at(&self, unix_seconds: u64) -> Result<String> {
        let counter = unix_seconds / self.period;
        let mac = self.algorithm.sign(&self.seed, counter.to_be_bytes())?;

        // Dynamic truncation, RFC 4226 §5.4. The low nibble of the last byte picks the offset;
        // the high bit of the first selected byte is masked off so the result is positive in
        // languages without unsigned arithmetic.
        let offset = usize::from(mac.last().copied().ok_or(Error::MalformedTotpSecret)? & 0x0f);
        let selected: [u8; 4] = mac
            .get(offset..offset + 4)
            .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
            .ok_or(Error::MalformedTotpSecret)?;
        let binary = u32::from_be_bytes(selected) & 0x7fff_ffff;

        let modulus = 10_u32
            .checked_pow(self.digits)
            .ok_or(Error::MalformedTotpSecret)?;
        // Zero-padded to the full width: a code whose leading digit happens to be zero is
        // still that many digits, and trimming it is a login failure roughly one time in ten.
        Ok(format!(
            "{code:0width$}",
            code = binary % modulus,
            width = self.digits as usize
        ))
    }

    /// When the step containing `unix_seconds` ends, in Unix seconds.
    ///
    /// This is what the countdown ring draws, and it is the *step* boundary rather than
    /// "now plus the period" — every authenticator in the world rolls over at the same instant,
    /// and a ring that started counting when the pane opened would disagree with the phone
    /// beside it.
    pub fn expires_at(&self, unix_seconds: u64) -> u64 {
        (unix_seconds / self.period)
            .saturating_add(1)
            .saturating_mul(self.period)
    }

    /// Digits per code, after the URI has had its say.
    pub fn digits(&self) -> u32 {
        self.digits
    }

    /// Seconds per step, after the URI has had its say.
    pub fn period(&self) -> u64 {
        self.period
    }

    /// Which HMAC is underneath.
    pub fn algorithm(&self) -> TotpAlgorithm {
        self.algorithm
    }
}

impl fmt::Debug for TotpSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The parameters are not secret and are useful in a bug report; the seed is the
        // credential. Same rule as `SecretString`: a struct that derives `Debug` around a
        // secret is one panic message away from putting it in a log file.
        f.debug_struct("TotpSpec")
            .field("seed", &"***")
            .field("algorithm", &self.algorithm)
            .field("digits", &self.digits)
            .field("period", &self.period)
            .finish()
    }
}

/// Case-insensitive [`str::strip_prefix`], for the URI scheme.
fn strip_prefix_ignore_case<'a>(input: &'a str, prefix: &str) -> Option<&'a str> {
    let head = input.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| input.get(prefix.len()..))
        .flatten()
}

/// Decodes `%XX` escapes and `+` in a query parameter value.
///
/// Hand-written rather than pulled in: the whole of what needs decoding here is a base32 seed
/// that may arrive with its `=` padding escaped as `%3D`, and the three parameters beside it
/// are ASCII words and integers. A URL crate for that is a dependency chosen for eleven lines.
/// An invalid escape is left verbatim, which then fails the alphabet check with a message
/// about the seed rather than about percent-encoding.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'%' => {
                let hex = bytes.get(index + 1..index + 3).and_then(|pair| {
                    core::str::from_utf8(pair)
                        .ok()
                        .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                });
                match hex {
                    Some(decoded) => {
                        out.push(decoded);
                        index += 3;
                    }
                    None => {
                        out.push(byte);
                        index += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            _ => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The RFC 6238 Appendix B seed for HMAC-SHA-1: ASCII "12345678901234567890".
    const SEED_SHA1: &[u8] = b"12345678901234567890";
    /// The SHA-256 seed. **32 bytes, not 20** — the table in Appendix B prints the 20-byte
    /// seed for every row, and errata 2866 records that the SHA-256 and SHA-512 rows were
    /// generated with the seed repeated to the hash's block size. A reader who takes the table
    /// at face value gets an implementation that fails its own vectors and "fixes" the
    /// truncation until it passes, which is how a wrong TOTP ships with a green test.
    const SEED_SHA256: &[u8] = b"12345678901234567890123456789012";
    /// The SHA-512 seed, 64 bytes, same erratum.
    const SEED_SHA512: &[u8] = b"1234567890123456789012345678901234567890123456789012345678901234";

    /// Base32-encodes a raw seed, which is how a real one arrives.
    fn encode(seed: &[u8]) -> String {
        BASE32_NOPAD.encode(seed)
    }

    #[test]
    fn rfc_6238_appendix_b_vectors() {
        // The whole reason this module is 40 lines of arithmetic rather than a dependency:
        // upstream published the numbers. Eight digits, T0 = 0, X = 30 — Appendix B's own
        // parameters, not ours.
        let cases: [(u64, &str, &str, &str); 6] = [
            (59, "94287082", "46119246", "90693936"),
            (1_111_111_109, "07081804", "68084774", "25091201"),
            (1_111_111_111, "14050471", "67062674", "99943326"),
            (1_234_567_890, "89005924", "91819424", "93441116"),
            (2_000_000_000, "69279037", "90698825", "38618901"),
            (20_000_000_000, "65353130", "77737706", "47863826"),
        ];

        for (time, sha1, sha256, sha512) in cases {
            for (seed, algorithm, expected) in [
                (SEED_SHA1, TotpAlgorithm::Sha1, sha1),
                (SEED_SHA256, TotpAlgorithm::Sha256, sha256),
                (SEED_SHA512, TotpAlgorithm::Sha512, sha512),
            ] {
                let spec = TotpSpec::from_parts(&encode(seed), algorithm, 8, 30)
                    .expect("the RFC's own parameters");
                assert_eq!(
                    spec.code_at(time).expect("a code"),
                    expected,
                    "{algorithm:?} at t={time}"
                );
            }
        }
    }

    #[test]
    fn six_digit_codes_keep_their_leading_zeros() {
        // A code is a fixed-width string, not a number. Trimming the leading zero fails a
        // login about one time in ten and looks like a server problem when it does.
        let spec = TotpSpec::from_parts(&encode(SEED_SHA1), TotpAlgorithm::Sha1, 6, 30)
            .expect("valid parts");
        // t = 1111111109 gives 07081804 at eight digits, so 081804 at six.
        let code = spec.code_at(1_111_111_109).expect("a code");
        assert_eq!(code, "081804");
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn a_bare_base32_seed_parses_however_it_was_copied() {
        // Every one of these is a shape a setup page actually prints.
        let canonical = encode(SEED_SHA1);
        let variants = [
            canonical.clone(),
            canonical.to_lowercase(),
            format!("  {canonical}\n"),
            canonical
                .as_bytes()
                .chunks(4)
                .filter_map(|chunk| core::str::from_utf8(chunk).ok())
                .collect::<Vec<_>>()
                .join(" "),
            format!("{canonical}===="),
        ];
        for variant in variants {
            let spec = TotpSpec::parse(&variant).expect("a seed however it is spaced");
            assert_eq!(spec.code_at(59).expect("a code"), "287082");
        }
    }

    #[test]
    fn an_otpauth_uri_is_read_including_its_parameters() {
        // The shape `import/bitwarden.rs` stores verbatim. Reading only the base32 out of it
        // and ignoring `digits`/`algorithm` is the failure this test exists for: the code
        // below is well-formed and wrong under every other reading.
        let uri = format!(
            "otpauth://totp/TrustVault:demo@example.com?secret={}&issuer=TrustVault&algorithm=SHA256&digits=8&period=30",
            encode(SEED_SHA256)
        );
        let spec = TotpSpec::parse(&uri).expect("a URI Bitwarden emits");
        assert_eq!(spec.algorithm(), TotpAlgorithm::Sha256);
        assert_eq!(spec.digits(), 8);
        assert_eq!(spec.period(), 30);
        assert_eq!(spec.code_at(59).expect("a code"), "46119246");
    }

    #[test]
    fn a_uri_without_parameters_takes_the_defaults() {
        let uri = format!("otpauth://totp/demo?secret={}", encode(SEED_SHA1));
        let spec = TotpSpec::parse(&uri).expect("the common shape");
        assert_eq!(spec.algorithm(), TotpAlgorithm::Sha1);
        assert_eq!(spec.digits(), 6);
        assert_eq!(spec.period(), 30);
        assert_eq!(spec.code_at(59).expect("a code"), "287082");
    }

    #[test]
    fn an_escaped_seed_survives_percent_decoding() {
        let uri = format!("otpauth://totp/a%20b?secret={}%3D%3D", encode(SEED_SHA1));
        let spec = TotpSpec::parse(&uri).expect("padding may arrive escaped");
        assert_eq!(spec.code_at(59).expect("a code"), "287082");
    }

    #[test]
    fn hotp_is_refused_rather_than_read_as_totp() {
        // The single most dangerous input this parser can be handed: it decodes, it has a
        // seed, and it would generate codes forever. HOTP counts logins; nothing here has
        // that counter, so every code produced from one is wrong and looks right.
        let uri = format!("otpauth://hotp/demo?secret={}&counter=1", encode(SEED_SHA1));
        assert!(matches!(
            TotpSpec::parse(&uri),
            Err(Error::MalformedTotpSecret)
        ));
    }

    #[test]
    fn an_unknown_algorithm_is_refused_rather_than_defaulted() {
        let uri = format!(
            "otpauth://totp/demo?secret={}&algorithm=SHA3",
            encode(SEED_SHA1)
        );
        assert!(matches!(
            TotpSpec::parse(&uri),
            Err(Error::MalformedTotpSecret)
        ));
    }

    #[test]
    fn parameters_outside_the_bounds_are_refused_not_clamped() {
        let seed = encode(SEED_SHA1);
        for (digits, period) in [(5, 30), (9, 30), (6, 0), (6, MAX_PERIOD + 1)] {
            assert!(
                matches!(
                    TotpSpec::from_parts(&seed, TotpAlgorithm::Sha1, digits, period),
                    Err(Error::MalformedTotpSecret)
                ),
                "digits {digits}, period {period}"
            );
        }
    }

    #[test]
    fn nothing_that_is_not_a_seed_parses() {
        for input in [
            "",
            "   ",
            "not base32!",
            "0189",                         // the four digits base32 excludes
            "https://example.com/",         // a URL in the field, which imports do produce
            "otpauth://totp/demo",          // no query at all
            "otpauth://totp/demo?issuer=x", // a query with no seed
            "otpauth://totp",               // no path
        ] {
            assert!(
                matches!(TotpSpec::parse(input), Err(Error::MalformedTotpSecret)),
                "{input:?} must not parse"
            );
        }
    }

    #[test]
    fn the_step_boundary_is_absolute_not_relative_to_now() {
        // What makes the ring agree with the phone next to it. Every device in the world
        // rolls over at the same instant; a countdown started when the pane opened does not.
        let spec = TotpSpec::from_parts(&encode(SEED_SHA1), TotpAlgorithm::Sha1, 6, 30)
            .expect("valid parts");
        assert_eq!(spec.expires_at(0), 30);
        assert_eq!(spec.expires_at(1), 30);
        assert_eq!(spec.expires_at(29), 30);
        assert_eq!(spec.expires_at(30), 60);
        // And the code is constant across the step it belongs to, which is the same property
        // read from the other side.
        assert_eq!(spec.code_at(30).ok(), spec.code_at(59).ok());
        assert_ne!(spec.code_at(59).ok(), spec.code_at(60).ok());
    }

    #[test]
    fn debug_never_prints_the_seed() {
        let spec = TotpSpec::parse(&encode(SEED_SHA1)).expect("valid seed");
        let printed = format!("{spec:?}");
        assert!(printed.contains("***"));
        assert!(!printed.contains("12345"), "{printed}");
        assert!(!printed.contains(&encode(SEED_SHA1)), "{printed}");
    }
}
