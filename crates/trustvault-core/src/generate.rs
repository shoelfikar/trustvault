//! Password generation — R-15, D-44.
//!
//! The generator lives here rather than in the webview for one reason, and it is the same
//! reason the nonces do: randomness that protects a stored credential belongs on the single
//! path R-06 and D-23 already constrain — [`crate::aead::random`], the OS CSPRNG, with no
//! seedable generator anywhere in this crate for a tired afternoon to reach for. A second
//! generator in JavaScript, with one of the two being "the real one", is a distinction that
//! survives exactly as long as the person who remembers it.
//!
//! Two properties are easy to lose in a refactor and both are pinned by tests below:
//!
//! * **Rejection sampling, never modulo.** `byte % alphabet.len()` is biased towards the
//!   first characters of the alphabet whenever 256 is not a multiple of its length, which it
//!   never is here. Discarding the tail of the byte range costs a few extra draws and removes
//!   the bias entirely.
//! * **Every selected set appears in the output**, which is R-15's acceptance criterion. The
//!   guaranteed characters are placed first and then shuffled, because a generator that always
//!   opens with a lowercase letter and a digit has published two positions of every password
//!   it has ever produced.

use zeroize::Zeroizing;

use crate::aead;
use crate::secret::SecretString;
use crate::{Error, Result};

/// Shortest password the generator will produce — R-15.
pub const MIN_PASSWORD_LENGTH: usize = 8;
/// Longest password the generator will produce — R-15.
pub const MAX_PASSWORD_LENGTH: usize = 64;

/// The full lowercase alphabet, before ambiguity filtering.
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
/// The full uppercase alphabet, before ambiguity filtering.
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// The full digit set, before ambiguity filtering.
const DIGITS: &[u8] = b"0123456789";
/// The symbol set.
///
/// No quotes, no backslash, no backtick, no space. Not a typography preference: those are the
/// characters a shell, a form, or a CSV export is most likely to mangle or strip, and a
/// generated password that cannot be typed back in is a lockout rather than an inconvenience.
const SYMBOLS: &[u8] = b"!@#$%&*-_=+?";

/// The glyphs `MASTER.md` §3 names as indistinguishable when transcribed by hand.
///
/// Exactly these five, and no more. Dropping lowercase `o` as well — which the retired
/// webview preview did — is unnecessary once `0` and `O` are both gone, and every character
/// removed from the alphabet is entropy paid for nothing.
const AMBIGUOUS: &[u8] = b"0O1lI";

/// Which character classes a generated password may draw from.
///
/// Deserialized straight from the IPC request, which is why it is a plain struct of booleans
/// rather than a bitflag set: the wire shape in `docs/ipc-contract.md` §7 is four named
/// booleans, and a shape that reads the same in the document and in the source is one fewer
/// place for the two to disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub struct CharSets {
    /// `a`–`z`.
    pub lowercase: bool,
    /// `A`–`Z`.
    pub uppercase: bool,
    /// `0`–`9`.
    pub digits: bool,
    /// Punctuation, from a set chosen to survive a shell and a web form.
    pub symbols: bool,
}

impl CharSets {
    /// All four classes, which is what every surface in the app asks for today.
    pub const ALL: Self = Self {
        lowercase: true,
        uppercase: true,
        digits: true,
        symbols: true,
    };
}

/// A validated generation request.
///
/// The fields are private and the only constructor validates, so an out-of-range length or a
/// request with no character class at all cannot be represented — the caller handles the
/// refusal once, at the boundary, instead of every generation path having to consider it. It
/// is the same shape `EditField::into_edit` uses at the IPC layer and for the same reason.
#[derive(Debug, Clone, Copy)]
pub struct PasswordRecipe {
    length: usize,
    sets: CharSets,
    exclude_ambiguous: bool,
}

impl PasswordRecipe {
    /// Validates a request, or refuses it.
    ///
    /// Returns `None` when `length` is outside [`MIN_PASSWORD_LENGTH`]..=[`MAX_PASSWORD_LENGTH`] or when every
    /// character class is off. Both are bugs in the caller rather than something a user did —
    /// the surfaces clamp the slider and keep one class on — so there is nothing to
    /// distinguish and nothing to report beyond the refusal itself.
    pub fn new(length: usize, sets: CharSets, exclude_ambiguous: bool) -> Option<Self> {
        let any = sets.lowercase || sets.uppercase || sets.digits || sets.symbols;
        if !(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).contains(&length) || !any {
            return None;
        }
        Some(Self {
            length,
            sets,
            exclude_ambiguous,
        })
    }

    /// How long the generated password will be.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Generates one password.
    ///
    /// Fails only if the operating system refuses entropy, which is [`Error::Entropy`] and has
    /// no fallback by design: a PRNG standing in for the OS CSPRNG is exactly what R-06 exists
    /// to prevent.
    pub fn generate(&self) -> Result<SecretString> {
        let alphabets = self.alphabets();
        let mut draw = Draw::new();

        // Capacity up front, so no push reallocates. A reallocation would copy the
        // half-built password into a fresh allocation and leave the old one in freed heap
        // that nothing zeroizes — the one way this function could leak without ever
        // returning the value to anybody.
        let mut bytes: Vec<u8> = Vec::with_capacity(self.length);

        // One character from each selected class first: R-15 asks that every set the user
        // chose appears, and rejection-sampling the whole password from the union satisfies
        // that only on average.
        for alphabet in &alphabets {
            bytes.push(draw.pick(alphabet)?);
        }
        let union: Vec<u8> = alphabets.concat();
        while bytes.len() < self.length {
            bytes.push(draw.pick(&union)?);
        }

        // Fisher-Yates over the whole string, so the guaranteed characters are not sitting in
        // known positions. Modern Fisher-Yates: index `i` swaps with a uniform index in
        // `0..=i`, which is the version that produces every permutation with equal
        // probability. The naive variant — a uniform index over the whole array — does not.
        for i in (1..bytes.len()).rev() {
            let j = draw.below(i + 1)?;
            bytes.swap(i, j);
        }

        // Every alphabet is ASCII, so this cannot fail; `Error::Encode` is the honest answer
        // if that ever stops being true rather than an unwrap that would take the process
        // down with a user's vault open. The `Vec`'s buffer is moved, not copied, so the
        // plaintext exists in exactly one allocation and `SecretString` zeroizes that one.
        String::from_utf8(bytes)
            .map(SecretString::from)
            .map_err(|_| Error::Encode)
    }

    /// The alphabets this recipe draws from, in a stable order.
    fn alphabets(&self) -> Vec<Vec<u8>> {
        [
            (self.sets.lowercase, LOWERCASE),
            (self.sets.uppercase, UPPERCASE),
            (self.sets.digits, DIGITS),
            (self.sets.symbols, SYMBOLS),
        ]
        .into_iter()
        .filter(|(selected, _)| *selected)
        .map(|(_, alphabet)| {
            alphabet
                .iter()
                .copied()
                .filter(|byte| !self.exclude_ambiguous || !AMBIGUOUS.contains(byte))
                .collect()
        })
        .collect()
    }
}

/// A buffered source of uniform random numbers.
///
/// Buffered because the alternative is one syscall per character, and 64 bytes at a time is
/// enough for most passwords in a single draw. The buffer is `Zeroizing`: it holds the raw
/// bytes the password was derived from, which are as good as the password itself until they
/// are gone.
struct Draw {
    buffer: Zeroizing<[u8; Self::CHUNK]>,
    next: usize,
}

impl Draw {
    /// Bytes drawn from the OS at a time.
    const CHUNK: usize = 64;

    /// An empty source; the first byte asked for fills it.
    fn new() -> Self {
        Self {
            buffer: Zeroizing::new([0u8; Self::CHUNK]),
            next: Self::CHUNK,
        }
    }

    /// One random byte, refilling from the OS when the buffer runs out.
    fn byte(&mut self) -> Result<u8> {
        if self.next >= Self::CHUNK {
            *self.buffer = aead::random::<{ Self::CHUNK }>()?;
            self.next = 0;
        }
        // `get` rather than an index: this crate may not panic, and the bound above is a
        // property of this function rather than of the type.
        let byte = self.buffer.get(self.next).copied().ok_or(Error::Entropy)?;
        self.next += 1;
        Ok(byte)
    }

    /// A uniform value in `0..len`, by rejection sampling.
    ///
    /// `len` is at most 256 at both call sites by construction — the widest alphabet is 74
    /// characters and the longest password 64 — and a wider one is refused rather than served
    /// with a modulo. That refusal is not defensive tidiness: a silently biased generator is
    /// the failure this module is written to avoid, and it is invisible from the outside.
    fn below(&mut self, len: usize) -> Result<usize> {
        if !(1..=256).contains(&len) {
            return Err(Error::Entropy);
        }
        // The largest multiple of `len` that fits in a byte. Values at or above it are
        // discarded, which is what removes the modulo bias.
        let limit = 256 - (256 % len);
        loop {
            let byte = usize::from(self.byte()?);
            if byte < limit {
                return Ok(byte % len);
            }
        }
    }

    /// One character drawn uniformly from `alphabet`.
    fn pick(&mut self, alphabet: &[u8]) -> Result<u8> {
        let index = self.below(alphabet.len())?;
        alphabet.get(index).copied().ok_or(Error::Entropy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every recipe in these tests, generated once.
    fn generated(length: usize, sets: CharSets, exclude_ambiguous: bool) -> String {
        PasswordRecipe::new(length, sets, exclude_ambiguous)
            .expect("a valid recipe")
            .generate()
            .expect("the OS has entropy")
            .expose()
            .to_owned()
    }

    #[test]
    fn a_password_is_the_length_that_was_asked_for() {
        for length in [MIN_PASSWORD_LENGTH, 20, MAX_PASSWORD_LENGTH] {
            assert_eq!(
                generated(length, CharSets::ALL, true).chars().count(),
                length
            );
        }
    }

    #[test]
    fn a_length_outside_the_range_is_refused_rather_than_clamped() {
        // Clamping is the tempting alternative and it is worse: a slider bug that asks for 4
        // would then produce a four-character password the surface reports as eight.
        assert!(PasswordRecipe::new(MIN_PASSWORD_LENGTH - 1, CharSets::ALL, true).is_none());
        assert!(PasswordRecipe::new(MAX_PASSWORD_LENGTH + 1, CharSets::ALL, true).is_none());
        assert!(PasswordRecipe::new(0, CharSets::ALL, true).is_none());
        assert_eq!(
            PasswordRecipe::new(MIN_PASSWORD_LENGTH, CharSets::ALL, true).map(|r| r.length()),
            Some(MIN_PASSWORD_LENGTH)
        );
    }

    #[test]
    fn a_request_with_no_character_class_is_refused() {
        // The alternative implementations both fail silently: falling back to lowercase
        // produces a password far weaker than the surface claims, and an empty alphabet
        // divides by zero.
        let none = CharSets {
            lowercase: false,
            uppercase: false,
            digits: false,
            symbols: false,
        };
        assert!(PasswordRecipe::new(20, none, true).is_none());
    }

    #[test]
    fn every_selected_set_appears_in_the_output() {
        // R-15's acceptance criterion. Run repeatedly because the failure it guards against
        // is probabilistic: drawing the whole password from the union satisfies this most of
        // the time, which is how it survives a single-run test.
        for _ in 0..64 {
            let password = generated(MIN_PASSWORD_LENGTH, CharSets::ALL, true);
            assert!(
                password.bytes().any(|b| LOWERCASE.contains(&b)),
                "{password}"
            );
            assert!(
                password.bytes().any(|b| UPPERCASE.contains(&b)),
                "{password}"
            );
            assert!(password.bytes().any(|b| DIGITS.contains(&b)), "{password}");
            assert!(password.bytes().any(|b| SYMBOLS.contains(&b)), "{password}");
        }
    }

    #[test]
    fn only_the_selected_sets_appear_in_the_output() {
        let digits_only = CharSets {
            lowercase: false,
            uppercase: false,
            digits: true,
            symbols: false,
        };
        let password = generated(MAX_PASSWORD_LENGTH, digits_only, true);
        assert!(password.bytes().all(|b| DIGITS.contains(&b)), "{password}");
    }

    #[test]
    fn ambiguous_glyphs_are_dropped_when_asked_and_kept_otherwise() {
        // MASTER.md §3. The second half matters as much as the first: if the flag did
        // nothing, the first assertion would pass forever on an alphabet that never had them.
        let filtered = generated(MAX_PASSWORD_LENGTH, CharSets::ALL, true);
        assert!(
            !filtered.bytes().any(|b| AMBIGUOUS.contains(&b)),
            "{filtered}"
        );

        let mut seen = false;
        for _ in 0..64 {
            seen |= generated(MAX_PASSWORD_LENGTH, CharSets::ALL, false)
                .bytes()
                .any(|b| AMBIGUOUS.contains(&b));
        }
        assert!(
            seen,
            "the flag has to change the alphabet, or it is theatre"
        );
    }

    #[test]
    fn two_generations_differ() {
        // Not a randomness test — no test here can be one — but it does catch the whole class
        // of failures where the buffer is never refilled or the draw index never advances.
        let first = generated(MAX_PASSWORD_LENGTH, CharSets::ALL, true);
        let second = generated(MAX_PASSWORD_LENGTH, CharSets::ALL, true);
        assert_ne!(first, second);
    }

    #[test]
    fn the_guaranteed_characters_are_not_left_in_fixed_positions() {
        // Without the shuffle, position 0 is always lowercase and position 3 always a symbol,
        // which publishes four positions of every password the app has ever produced. Over 64
        // draws the un-shuffled version fails this on the first one.
        let mut first_positions = std::collections::HashSet::new();
        for _ in 0..64 {
            let password = generated(20, CharSets::ALL, true);
            let head = password.bytes().next().unwrap_or(b'?');
            first_positions.insert(
                [LOWERCASE, UPPERCASE, DIGITS, SYMBOLS]
                    .iter()
                    .position(|set| set.contains(&head)),
            );
        }
        assert!(
            first_positions.len() > 1,
            "every password started with the same character class"
        );
    }

    #[test]
    fn the_sampler_covers_its_whole_range_and_refuses_a_bad_one() {
        let mut draw = Draw::new();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..512 {
            let value = draw.below(3).expect("the OS has entropy");
            assert!(value < 3);
            seen.insert(value);
        }
        assert_eq!(seen.len(), 3, "every value in the range is reachable");

        // The bounds the call sites guarantee, asserted here because nothing else can reach
        // them: a zero-length alphabet would divide by zero and a wider one would be biased.
        assert!(draw.below(0).is_err());
        assert!(draw.below(257).is_err());
        assert!(draw.pick(&[]).is_err());
    }

    #[test]
    fn the_generated_value_never_prints_itself() {
        // N-01 at this layer: the recipe is `Debug` because it is logged in tests and error
        // paths, and it must stay a description of the request rather than of the answer.
        let recipe = PasswordRecipe::new(20, CharSets::ALL, true).expect("a valid recipe");
        let password = recipe.generate().expect("the OS has entropy");
        assert!(!format!("{recipe:?}").contains(password.expose()));
        assert_eq!(format!("{password:?}"), "SecretString(***)");
    }
}
