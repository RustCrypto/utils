//! [`UInt`] subtraction modulus operations.

use super::UInt;

impl UInt<1> {
    /// Computes `a - b mod p` in constant time.
    pub const fn sub_mod(&self, rhs: &Self, p: &Self) -> Self {
        let l0 = base::sub1(self.limbs[0], rhs.limbs[0], p.limbs[0]);

        UInt::new([l0])
    }
}

impl UInt<2> {
    /// Computes `a - b mod p` in constant time.
    pub const fn sub_mod(&self, rhs: &Self, p: &Self) -> Self {
        base::sub2(self, rhs, p)
    }
}

impl UInt<3> {
    /// Computes `a - b mod p` in constant time.
    pub const fn sub_mod(&self, rhs: &Self, p: &Self) -> Self {
        base::sub3(self, rhs, p)
    }
}

impl UInt<4> {
    /// Computes `a - b mod p` in constant time.
    pub const fn sub_mod(&self, rhs: &Self, p: &Self) -> Self {
        base::sub4(self, rhs, p)
    }
}

pub(super) mod base {
    use crate::{Limb, UInt};

    pub const fn sub1(a: Limb, b: Limb, p: Limb) -> Limb {
        let (l0, borrow) = a.sbb(b, Limb::ZERO);

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
        let (l0, _) = l0.adc(p.bitand(borrow), Limb::ZERO);

        l0
    }

    pub const fn sub2(a: &UInt<2>, b: &UInt<2>, p: &UInt<2>) -> UInt<2> {
        sub(a, b, p)
    }

    pub const fn sub3(a: &UInt<3>, b: &UInt<3>, p: &UInt<3>) -> UInt<3> {
        sub(a, b, p)
    }

    pub const fn sub4(a: &UInt<4>, b: &UInt<4>, p: &UInt<4>) -> UInt<4> {
        sub(a, b, p)
    }

    pub const fn sub<const LIMBS: usize>(
        a: &UInt<LIMBS>,
        b: &UInt<LIMBS>,
        p: &UInt<LIMBS>,
    ) -> UInt<LIMBS> {
        let mut out = [Limb::ZERO; LIMBS];
        let mut borrow = Limb::ZERO;
        let mut i = 0;
        while i < LIMBS {
            let (l, b) = a.limbs[i].sbb(b.limbs[i], borrow);
            out[i] = l;
            borrow = b;
            i += 1;
        }

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.

        let mut carry = Limb::ZERO;
        let mut i = 0;
        while i < LIMBS {
            let (l, c) = out[i].adc(p.limbs[i].bitand(borrow), carry);
            out[i] = l;
            carry = c;
            i += 1;
        }

        UInt::new(out)
    }
}
