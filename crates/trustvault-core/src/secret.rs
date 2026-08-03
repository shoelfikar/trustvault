//! Wrappers that make a secret hard to leak by accident.
//!
//! Two things are being defended against, and only one of them is memory. The other is the
//! `Debug` impl: a `String` inside a struct that derives `Debug` ends up in a log line, a panic
//! message, or an error report, and no amount of zeroizing helps once it is on disk in a log
//! file. Both wrappers here print a placeholder and never their contents (N-01).

use core::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

/// A secret string that zeroizes on drop and never prints itself.
///
/// Used for every field value in the model, secret or not. Applying it uniformly costs a
/// memset per drop and removes the question "was this field marked secret when it was
/// written" from the memory-hygiene story entirely.
#[derive(Clone, Default)]
pub struct SecretString(Zeroizing<String>);

impl SecretString {
    /// Wraps a string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    /// Borrows the plaintext.
    ///
    /// Every call site is a place where a secret can escape, which is why this is a named
    /// method and not a `Deref`: the boundary should be visible when reading the code.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Length in bytes of the plaintext.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Not the length either: for a password, the length is most of what an attacker
        // reading a log wants.
        f.write_str("SecretString(***)")
    }
}

impl PartialEq for SecretString {
    /// Constant-time. Comparing secrets with `==` on `str` short-circuits at the first
    /// differing byte, which is a timing oracle wherever the comparison is attacker-driven.
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes().ct_eq(other.0.as_bytes()).into()
    }
}

impl Eq for SecretString {}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
}

impl Serialize for SecretString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialization is how a secret legitimately reaches the sealed body. It must never
        // reach anything else; that is enforced at the IPC boundary in Phase 2, not here.
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}

/// A fixed-size secret byte array that zeroizes on drop and never prints itself.
///
/// Key material: the master key, the derived key-encryption keys, the recovery secret.
#[derive(Clone)]
pub struct SecretBytes<const N: usize>([u8; N]);

impl<const N: usize> SecretBytes<N> {
    /// Wraps an array. The caller's copy is not zeroized — pass a temporary.
    pub fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// All zeroes. Used as the stand-in key on the failed-unwrap path so that a wrong
    /// password performs the same work as a corrupt file (R-03).
    pub fn zeroed() -> Self {
        Self([0u8; N])
    }

    /// Borrows the bytes.
    pub fn expose(&self) -> &[u8; N] {
        &self.0
    }

    /// Borrows the bytes mutably, for filling from the CSPRNG or the KDF.
    pub(crate) fn expose_mut(&mut self) -> &mut [u8; N] {
        &mut self.0
    }
}

impl<const N: usize> fmt::Debug for SecretBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBytes<{N}>(***)")
    }
}

impl<const N: usize> PartialEq for SecretBytes<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl<const N: usize> Eq for SecretBytes<N> {}

impl<const N: usize> Zeroize for SecretBytes<N> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<const N: usize> Drop for SecretBytes<N> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_prints_the_secret() {
        // The whole point of the type. If this test is ever "fixed" by printing the value,
        // every log line in the application becomes a potential disclosure.
        let s = SecretString::new("hunter2");
        assert_eq!(format!("{s:?}"), "SecretString(***)");
        assert!(!format!("{s:?}").contains("hunter2"));

        let k = SecretBytes::new([9u8; 32]);
        assert_eq!(format!("{k:?}"), "SecretBytes<32>(***)");
    }

    #[test]
    fn secret_strings_compare_by_value() {
        assert_eq!(SecretString::new("a"), SecretString::new("a"));
        assert_ne!(SecretString::new("a"), SecretString::new("b"));
        // Different lengths must not panic in the constant-time path.
        assert_ne!(SecretString::new("a"), SecretString::new("aa"));
    }

    #[test]
    fn secret_strings_round_trip_through_serde() {
        let json = serde_json::to_string(&SecretString::new("hunter2")).ok();
        assert_eq!(json.as_deref(), Some(r#""hunter2""#));

        let back: Option<SecretString> = serde_json::from_str(r#""hunter2""#).ok();
        assert_eq!(back, Some(SecretString::new("hunter2")));
    }

    #[test]
    fn owned_strings_are_taken_by_value_not_copied() {
        // `From<String>` moves the allocation into the `Zeroizing` wrapper instead of
        // copying it. A copy would leave the original heap buffer un-zeroized behind us,
        // which is the exact failure N-01 exists to prevent.
        let owned = String::from("hunter2");
        assert_eq!(SecretString::from(owned), SecretString::new("hunter2"));
        assert_eq!(SecretString::from("hunter2"), SecretString::new("hunter2"));
    }

    #[test]
    fn secret_bytes_zeroed_is_all_zero() {
        assert_eq!(SecretBytes::<32>::zeroed().expose(), &[0u8; 32]);
    }

    #[test]
    fn secret_bytes_zeroize_wipes_in_place() {
        // N-01. `Drop` does this too, but a dropped value cannot be inspected — this is the
        // only way to assert that the bytes are actually gone rather than merely forgotten.
        let mut key = SecretBytes::new([0xAB; 32]);
        assert_ne!(key.expose(), &[0u8; 32]);
        key.zeroize();
        assert_eq!(key.expose(), &[0u8; 32]);
        assert_eq!(key, SecretBytes::<32>::zeroed());
    }

    #[test]
    fn secret_bytes_clone_is_independent() {
        let key = SecretBytes::new([7u8; 32]);
        let copy = key.clone();
        drop(key);
        assert_eq!(copy.expose(), &[7u8; 32]);
    }

    #[test]
    fn secret_string_reports_length_and_emptiness() {
        assert!(SecretString::default().is_empty());
        assert_eq!(SecretString::new("abc").len(), 3);
        assert!(!SecretString::from(String::from("abc")).is_empty());
    }
}
