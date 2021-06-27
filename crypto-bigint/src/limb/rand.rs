//! Random number generator support

use super::Limb;
use rand_core::{CryptoRng, RngCore};

impl Limb {
    /// Generate a random limb
    #[cfg(target_pointer_width = "32")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn random(mut rng: impl CryptoRng + RngCore) -> Self {
        Self(rng.next_u32())
    }

    /// Generate a random limb
    #[cfg(target_pointer_width = "64")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn random(mut rng: impl CryptoRng + RngCore) -> Self {
        Self(rng.next_u64())
    }
}
