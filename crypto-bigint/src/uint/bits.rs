use crate::limb::{Inner, BIT_SIZE};
use crate::{Limb, UInt};

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Calculate the number of bits needed to represent this number
    pub const fn bits(self) -> Inner {
        let mut i = LIMBS - 1;
        while i > 0 && self.limbs[i].0 == 0 {
            i -= 1;
        }
        let mut bits = BIT_SIZE * i;
        let mut limb = self.limbs[i].0;
        while limb != 0 {
            limb >>= 1;
            bits += 1;
        }

        Limb::ct_select(
            Limb(bits as Inner),
            Limb::ZERO,
            !self.limbs[0].is_nonzero() & !Limb(i as Inner).is_nonzero(),
        )
        .0
    }
}
