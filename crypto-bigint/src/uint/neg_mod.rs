//! [`UInt`] subtraction modulus operations.

use super::UInt;

impl UInt<1> {
    /// Computes `-a mod p` in constant time.
    pub const fn neg_mod(&self, p: &Self) -> Self {
        let l0 = base::neg1(self.limbs[0], p.limbs[0]);

        UInt::new([l0])
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

    pub const fn neg1(a: Limb, p: Limb) -> Limb {
        // Subtract `a` from `p` to negate. Ignore the final
        // borrow because it cannot underflow; a is guaranteed to
        // be in the field.
        let (l0, _) = p.sbb(a, Limb::ZERO);

        // `tmp` could be `p` if `a` was zero. Create a mask that is
        // zero if `a` was zero, and `Limb::MAX` if self was nonzero.

        // FIXME: constant time comparison
        let v = if a.eq_vartime(&Limb::ZERO) {
            Limb::ONE
        } else {
            Limb::ZERO
        };
        let mask = v.wrapping_sub(Limb::ONE);

        l0.bitand(mask)
    }

    pub const fn neg2(a: &UInt<2>, p: &UInt<2>) -> UInt<2> {
        // Subtract `a` from `p` to negate. Ignore the final
        // borrow because it cannot underflow; a is guaranteed to
        // be in the field.
        let (l0, borrow) = p.limbs[0].sbb(a.limbs[0], Limb::ZERO);
        let (l1, _) = p.limbs[1].sbb(a.limbs[1], borrow);

        // `tmp` could be `p` if `a` was zero. Create a mask that is
        // zero if `a` was zero, and `Limb::MAX` if self was nonzero.

        // FIXME: constant time comparison
        let v = if a.limbs[0].bitor(a.limbs[1]).eq_vartime(&Limb::ZERO) {
            Limb::ONE
        } else {
            Limb::ZERO
        };
        let mask = v.wrapping_sub(Limb::ONE);

        UInt::new([l0.bitand(mask), l1.bitand(mask)])
    }

    pub const fn neg3(a: &UInt<3>, p: &UInt<3>) -> UInt<3> {
        // Subtract `a` from `p` to negate. Ignore the final
        // borrow because it cannot underflow; a is guaranteed to
        // be in the field.
        let (l0, borrow) = p.limbs[0].sbb(a.limbs[0], Limb::ZERO);
        let (l1, borrow) = p.limbs[1].sbb(a.limbs[1], borrow);
        let (l2, _) = p.limbs[2].sbb(a.limbs[2], borrow);

        // `tmp` could be `p` if `a` was zero. Create a mask that is
        // zero if `a` was zero, and `Limb::MAX` if self was nonzero.

        // FIXME: constant time comparison
        let v = if a.limbs[0]
            .bitor(a.limbs[1])
            .bitor(a.limbs[2])
            .eq_vartime(&Limb::ZERO)
        {
            Limb::ONE
        } else {
            Limb::ZERO
        };
        let mask = v.wrapping_sub(Limb::ONE);

        UInt::new([l0.bitand(mask), l1.bitand(mask), l2.bitand(mask)])
    }

    pub const fn neg4(a: &UInt<4>, p: &UInt<4>) -> UInt<4> {
        // Subtract `a` from `p` to negate. Ignore the final
        // borrow because it cannot underflow; a is guaranteed to
        // be in the field.
        let (l0, borrow) = p.limbs[0].sbb(a.limbs[0], Limb::ZERO);
        let (l1, borrow) = p.limbs[1].sbb(a.limbs[1], borrow);
        let (l2, borrow) = p.limbs[2].sbb(a.limbs[2], borrow);
        let (l3, _) = p.limbs[3].sbb(a.limbs[3], borrow);

        // `tmp` could be `p` if `a` was zero. Create a mask that is
        // zero if `a` was zero, and `Limb::MAX` if self was nonzero.

        // FIXME: constant time comparison
        let v = if a.limbs[0]
            .bitor(a.limbs[1])
            .bitor(a.limbs[2])
            .bitor(a.limbs[3])
            .eq_vartime(&Limb::ZERO)
        {
            Limb::ONE
        } else {
            Limb::ZERO
        };
        let mask = v.wrapping_sub(Limb::ONE);

        UInt::new([
            l0.bitand(mask),
            l1.bitand(mask),
            l2.bitand(mask),
            l3.bitand(mask),
        ])
    }
}
