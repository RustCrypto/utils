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
        let (l0, borrow) = a.limbs[0].sbb(b.limbs[0], Limb::ZERO);
        let (l1, borrow) = a.limbs[1].sbb(b.limbs[1], borrow);

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
        let (l0, carry) = l0.adc(p.limbs[0].bitand(borrow), Limb::ZERO);
        let (l1, _) = l1.adc(p.limbs[1].bitand(borrow), carry);

        UInt::new([l0, l1])
    }

    pub const fn sub3(a: &UInt<3>, b: &UInt<3>, p: &UInt<3>) -> UInt<3> {
        let (l0, borrow) = a.limbs[0].sbb(b.limbs[0], Limb::ZERO);
        let (l1, borrow) = a.limbs[1].sbb(b.limbs[1], borrow);
        let (l2, borrow) = a.limbs[2].sbb(b.limbs[2], borrow);

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
        let (l0, carry) = l0.adc(p.limbs[0].bitand(borrow), Limb::ZERO);
        let (l1, carry) = l1.adc(p.limbs[1].bitand(borrow), carry);
        let (l2, _) = l2.adc(p.limbs[2].bitand(borrow), carry);

        UInt::new([l0, l1, l2])
    }

    pub const fn sub4(a: &UInt<4>, b: &UInt<4>, p: &UInt<4>) -> UInt<4> {
        let (l0, borrow) = a.limbs[0].sbb(b.limbs[0], Limb::ZERO);
        let (l1, borrow) = a.limbs[1].sbb(b.limbs[1], borrow);
        let (l2, borrow) = a.limbs[2].sbb(b.limbs[2], borrow);
        let (l3, borrow) = a.limbs[3].sbb(b.limbs[3], borrow);

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
        let (l0, carry) = l0.adc(p.limbs[0].bitand(borrow), Limb::ZERO);
        let (l1, carry) = l1.adc(p.limbs[1].bitand(borrow), carry);
        let (l2, carry) = l2.adc(p.limbs[2].bitand(borrow), carry);
        let (l3, _) = l3.adc(p.limbs[3].bitand(borrow), carry);

        UInt::new([l0, l1, l2, l3])
    }
}
