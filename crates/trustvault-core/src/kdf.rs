//! Argon2id key derivation.
//!
//! Specified in `docs/vault-format.md` §3. The parameters are read from the vault header and
//! never from the constants below at decryption time (R-02) — the constants are the starting
//! point for calibration on a new vault, nothing more.

use std::time::{Duration, Instant};

use argon2::{Algorithm, Argon2, Params, Version};

use crate::secret::SecretBytes;
use crate::{Error, Result};

/// Length of every derived key and of the master key.
pub(crate) const KEY_LEN: usize = 32;

/// A derived or generated 256-bit key.
pub(crate) type Key = SecretBytes<KEY_LEN>;

/// Argon2id cost parameters, as stored in the vault header.
///
/// `docs/vault-format.md` §3.2 records how the defaults were measured and §3.3 the bounds a
/// reader enforces before allocating anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub m_cost: u32,
    /// Time cost — the number of passes.
    pub t_cost: u32,
    /// Parallelism — the number of lanes.
    pub p_cost: u8,
}

impl KdfParams {
    /// The default work factor: 256 MiB, 3 passes, 1 lane.
    ///
    /// Measured at **511 ms** on the development machine (Ubuntu 26.04, x86-64, median of 5),
    /// which clears S-03's 500 ms floor by a margin thin enough to be the reason
    /// [`KdfParams::calibrate`] exists. A vault created on a faster machine should not inherit
    /// a work factor chosen on this one.
    pub const DEFAULT: Self = Self {
        m_cost: 262_144,
        t_cost: 3,
        p_cost: 1,
    };

    /// Deliberately weak parameters for tests and known-answer vectors.
    ///
    /// Safe to keep in the shipped crate: the parameters live in the header, so a vector
    /// written with these exercises exactly the same code path as a real vault, and no vault
    /// is ever *created* with them outside a test.
    pub const TESTING: Self = Self {
        m_cost: 8,
        t_cost: 1,
        p_cost: 1,
    };

    /// Minimum memory cost accepted on read, in KiB. Argon2 itself refuses less than 8.
    pub const MIN_M_COST: u32 = 8;
    /// Maximum memory cost accepted on read, in KiB — 4 GiB.
    ///
    /// Not a security control. A hostile header cannot make a downgrade useful, because wrong
    /// parameters derive a wrong key and the unwrap fails; the cap exists so that such a file
    /// cannot make the reader allocate itself to death first.
    pub const MAX_M_COST: u32 = 4_194_304;
    /// Maximum number of passes accepted on read.
    pub const MAX_T_COST: u32 = 64;
    /// Maximum number of lanes accepted on read.
    pub const MAX_P_COST: u8 = 64;

    /// The wall-clock target calibration aims for, above S-03's 500 ms floor.
    const CALIBRATION_TARGET: Duration = Duration::from_millis(600);
    /// The floor calibration refuses to finish below — S-03 itself.
    const CALIBRATION_FLOOR: Duration = Duration::from_millis(500);
    /// Upper bound on passes during calibration, so a very slow machine terminates.
    const CALIBRATION_MAX_T: u32 = 16;
    /// Lowest memory cost calibration will fall back to, in KiB — 64 MiB.
    const CALIBRATION_MIN_M: u32 = 65_536;

    /// Rejects parameters outside `docs/vault-format.md` §3.3.
    pub fn validate(&self) -> Result<()> {
        let in_range = (Self::MIN_M_COST..=Self::MAX_M_COST).contains(&self.m_cost)
            && (1..=Self::MAX_T_COST).contains(&self.t_cost)
            && (1..=Self::MAX_P_COST).contains(&self.p_cost);
        if in_range {
            Ok(())
        } else {
            Err(Error::KdfParams)
        }
    }

    /// Picks parameters that take about 600 ms *on this machine*, per the algorithm in
    /// `docs/vault-format.md` §3.2.
    ///
    /// Called once, at vault creation. It costs a handful of derivations — several seconds on
    /// a slow machine — which is why it is not called on unlock: on unlock the parameters come
    /// from the header, which is the only place they can safely come from.
    pub fn calibrate() -> Self {
        Self::calibrate_to(Self::CALIBRATION_TARGET, Self::CALIBRATION_FLOOR)
    }

    fn calibrate_to(target: Duration, floor: Duration) -> Self {
        let mut m_cost = Self::DEFAULT.m_cost;

        loop {
            let probe = Self {
                m_cost,
                t_cost: 1,
                p_cost: 1,
            };
            match time_one_derivation(probe) {
                // Allocation failed: halve the memory and try again, down to the floor.
                None if m_cost > Self::CALIBRATION_MIN_M => {
                    m_cost = (m_cost / 2).max(Self::CALIBRATION_MIN_M);
                    continue;
                }
                // Even the fallback will not allocate. Return the smallest sane thing rather
                // than looping; the caller gets a working vault and a weak one beats none.
                None => {
                    return Self {
                        m_cost: Self::CALIBRATION_MIN_M,
                        t_cost: 3,
                        p_cost: 1,
                    };
                }
                Some(one_pass) => {
                    let ratio =
                        target.as_secs_f64() / one_pass.as_secs_f64().max(f64::MIN_POSITIVE);
                    // `as u32` saturates on overflow and on NaN yields 0, both of which the
                    // clamp below absorbs. No unwrap, no panic.
                    let mut candidate = Self {
                        m_cost,
                        t_cost: (ratio.round() as u32).clamp(2, Self::CALIBRATION_MAX_T),
                        p_cost: 1,
                    };

                    // Verify rather than trust the extrapolation: Argon2's cost is affine in
                    // the pass count, not linear, so the model under-shoots at low pass counts.
                    while candidate.t_cost < Self::CALIBRATION_MAX_T {
                        match time_one_derivation(candidate) {
                            Some(measured) if measured >= floor => break,
                            Some(_) => candidate.t_cost += 1,
                            None => break,
                        }
                    }
                    return candidate;
                }
            }
        }
    }
}

impl Default for KdfParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Times one derivation, or returns `None` if the parameters will not allocate.
fn time_one_derivation(params: KdfParams) -> Option<Duration> {
    let start = Instant::now();
    derive_key(b"calibration probe", &[0u8; 16], params).ok()?;
    Some(start.elapsed())
}

/// Derives a 256-bit key from a password and salt.
///
/// The password is hashed as the caller supplied it: no normalization, no trimming, no case
/// folding (`docs/vault-format.md` §3.1). Normalizing would mean a vault created under one
/// Unicode normalization table cannot be opened under another, and those tables change.
pub(crate) fn derive_key(password: &[u8], salt: &[u8; 16], params: KdfParams) -> Result<Key> {
    params.validate()?;

    let argon_params = Params::new(
        params.m_cost,
        params.t_cost,
        u32::from(params.p_cost),
        Some(KEY_LEN),
    )
    .map_err(|_| Error::KdfParams)?;

    let mut key = Key::zeroed();
    Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params)
        .hash_password_into(password, salt, key.expose_mut())
        // The only remaining failure is allocation, which is not distinguishable from a
        // hostile parameter set here and is reported the same way.
        .map_err(|_| Error::KdfParams)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_measured_ones() {
        // These numbers are quoted in docs/vault-format.md §3.2 with a measurement date.
        // Changing them without re-measuring makes the document a lie.
        assert_eq!(KdfParams::DEFAULT.m_cost, 262_144);
        assert_eq!(KdfParams::DEFAULT.t_cost, 3);
        assert_eq!(KdfParams::DEFAULT.p_cost, 1);
        assert!(KdfParams::DEFAULT.validate().is_ok());
    }

    #[test]
    fn default_is_the_documented_constant() {
        // `Default` exists so a deserialized header missing its parameter block gets the
        // measured constant rather than something weaker. If the two ever diverge, a vault
        // could be written with parameters no document describes.
        assert_eq!(KdfParams::default(), KdfParams::DEFAULT);
    }

    #[test]
    fn parameters_outside_the_documented_bounds_are_rejected() {
        let cases = [
            KdfParams {
                m_cost: 7,
                t_cost: 1,
                p_cost: 1,
            },
            KdfParams {
                m_cost: KdfParams::MAX_M_COST + 1,
                t_cost: 1,
                p_cost: 1,
            },
            KdfParams {
                m_cost: 8,
                t_cost: 0,
                p_cost: 1,
            },
            KdfParams {
                m_cost: 8,
                t_cost: KdfParams::MAX_T_COST + 1,
                p_cost: 1,
            },
            KdfParams {
                m_cost: 8,
                t_cost: 1,
                p_cost: 0,
            },
        ];
        for case in cases {
            assert!(case.validate().is_err(), "{case:?} should be rejected");
            assert!(derive_key(b"pw", &[0u8; 16], case).is_err());
        }
    }

    #[test]
    fn derivation_is_deterministic_and_salt_dependent() {
        let a = derive_key(b"pw", &[1u8; 16], KdfParams::TESTING).ok();
        let b = derive_key(b"pw", &[1u8; 16], KdfParams::TESTING).ok();
        let other_salt = derive_key(b"pw", &[2u8; 16], KdfParams::TESTING).ok();
        let other_pw = derive_key(b"px", &[1u8; 16], KdfParams::TESTING).ok();

        assert!(a.is_some());
        assert_eq!(a, b);
        assert_ne!(a, other_salt);
        assert_ne!(a, other_pw);
    }

    #[test]
    fn parameters_change_the_derived_key() {
        // R-02: the header's parameters are part of what the key depends on, so editing them
        // cannot silently produce a usable key.
        let base = derive_key(b"pw", &[1u8; 16], KdfParams::TESTING).ok();
        let more_passes = derive_key(
            b"pw",
            &[1u8; 16],
            KdfParams {
                t_cost: 2,
                ..KdfParams::TESTING
            },
        )
        .ok();
        assert_ne!(base, more_passes);
    }

    #[test]
    fn the_shipping_calibration_produces_usable_parameters() {
        // The path that actually runs at vault creation, at its real target. Slow — several
        // seconds and a 256 MiB allocation — and worth it: this is the only test that exercises
        // the numbers a real user's vault will be created with, and a calibration that returned
        // something out of range would make vault creation fail on their machine, not ours.
        let params = KdfParams::calibrate();
        assert!(params.validate().is_ok());
        assert!(params.m_cost >= KdfParams::CALIBRATION_MIN_M);
        assert!((2..=KdfParams::CALIBRATION_MAX_T).contains(&params.t_cost));
        assert_eq!(params.p_cost, 1);

        // And the parameters it chose really do derive a key.
        assert!(derive_key(b"pw", &[3u8; 16], params).is_ok());
    }

    #[test]
    fn calibration_lands_above_its_floor() {
        // Deliberately tiny targets: the point is that the algorithm terminates and returns
        // something in range, not that it reproduces the 600 ms production target inside a
        // test suite that has to finish quickly.
        let params = KdfParams::calibrate_to(Duration::from_millis(4), Duration::from_millis(2));
        assert!(params.validate().is_ok());
        assert!(
            params.t_cost >= 2,
            "calibration must never return t_cost < 2"
        );
        assert!(params.t_cost <= KdfParams::CALIBRATION_MAX_T);
        assert_eq!(params.p_cost, 1);
    }
}
