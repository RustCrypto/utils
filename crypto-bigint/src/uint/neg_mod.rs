//! [`UInt`] subtraction modulus operations.

use super::UInt;

impl UInt<1> {
    /// Computes `-a mod p` in constant time.
    pub const fn neg_mod(&self, p: &Self) -> Self {
        base::neg1(self, p)
    }
}

impl UInt<2> {
    /// Computes `-a mod p` in constant time.
    pub const fn neg_mod(&self, p: &Self) -> Self {
        base::neg2(self, p)
    }
}

impl UInt<3> {
    /// Computes `-a mod p` in constant time.
    pub const fn neg_mod(&self, p: &Self) -> Self {
        base::neg3(self, p)
    }
}

impl UInt<4> {
    /// Computes `-a mod p` in constant time.
    pub const fn neg_mod(&self, p: &Self) -> Self {
        base::neg4(self, p)
    }
}

pub(super) mod base {
    use crate::{Limb, UInt};

    pub const fn neg1(a: &UInt<1>, p: &UInt<1>) -> UInt<1> {
        neg(a, p)
    }

    pub const fn neg2(a: &UInt<2>, p: &UInt<2>) -> UInt<2> {
        neg(a, p)
    }

    pub const fn neg3(a: &UInt<3>, p: &UInt<3>) -> UInt<3> {
        neg(a, p)
    }

    pub const fn neg4(a: &UInt<4>, p: &UInt<4>) -> UInt<4> {
        neg(a, p)
    }

    pub const fn neg<const LIMBS: usize>(a: &UInt<LIMBS>, p: &UInt<LIMBS>) -> UInt<LIMBS> {
        let mut tmp = [Limb::ZERO; LIMBS];

        // Subtract `a` from `p` to negate. Ignore the final
        // borrow because it cannot underflow; a is guaranteed to
        // be in the field.

        let mut borrow = Limb::ZERO;
        let mut i = 0;
        while i < LIMBS {
            let (l, b) = p.limbs[i].sbb(a.limbs[i], borrow);
            tmp[i] = l;
            borrow = b;

            i += 1;
        }

        // `tmp` could be `p` if `a` was zero. Create a mask that is
        // zero if `a` was zero, and `Limb::MAX` if self was nonzero.

        // FIXME: constant time comparison
        let mut a_or = a.limbs[0];
        let mut i = 1;
        while i < LIMBS {
            a_or = a_or.bitor(a.limbs[i]);

            i += 1;
        }

        let v = if a_or.eq_vartime(&Limb::ZERO) {
            Limb::ONE
        } else {
            Limb::ZERO
        };
        let mask = v.wrapping_sub(Limb::ONE);

        let mut i = 0;
        while i < LIMBS {
            tmp[i] = tmp[i].bitand(mask);

            i += 1;
        }

        UInt::new(tmp)
    }
}
