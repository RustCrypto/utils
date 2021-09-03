//! Random number generator support
// TODO(tarcieri): use `Random` and `RandomMod` impls exclusively in next breaking release

use super::UInt;
use crate::{Limb, Random, RandomMod};
use rand_core::{CryptoRng, RngCore};
use subtle::ConstantTimeLess;

// TODO(tarcieri): replace this `impl` block with `impl Random`/`impl RandomMod`
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
impl<const LIMBS: usize> UInt<LIMBS> {
    /// Generate a cryptographically secure random [`UInt`].
    pub fn random(mut rng: impl CryptoRng + RngCore) -> Self {
        let mut limbs = [Limb::default(); LIMBS];

        for limb in &mut limbs {
            *limb = Limb::random(&mut rng)
        }

        limbs.into()
    }

    /// Generate a cryptographically secure random [`UInt`] which is less than
    /// a given `modulus`.
    ///
    /// This function uses rejection sampling, a method which produces an
    /// unbiased distribution of in-range values provided the underlying
    /// [`CryptoRng`] is unbiased, but runs in variable-time.
    ///
    /// The variable-time nature of the algorithm should not pose a security
    /// issue so long as the underlying random number generator is truly a
    /// [`CryptoRng`], where previous outputs are unrelated to subsequent
    /// outputs and do not reveal information about the RNG's internal state.
    pub fn random_mod(mut rng: impl CryptoRng + RngCore, modulus: &Self) -> Self {
        loop {
            let n = Self::random(&mut rng);

            if n.ct_lt(modulus).into() {
                return n;
            }
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
impl<const LIMBS: usize> Random for UInt<LIMBS> {
    fn random(rng: impl CryptoRng + RngCore) -> Self {
        Self::random(rng)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
impl<const LIMBS: usize> RandomMod for UInt<LIMBS> {
    fn random_mod(rng: impl CryptoRng + RngCore, modulus: &Self) -> Self {
        Self::random_mod(rng, modulus)
    }
}
