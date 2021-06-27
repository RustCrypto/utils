//! [`UInt`] addition modulus operations.

use super::UInt;

impl UInt<1> {
    /// Computes `a + b mod p` in constant time.
    pub const fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
        let l0 = base::add1(self.limbs[0], rhs.limbs[0], p.limbs[0]);
        UInt::new([l0])
    }
}

impl UInt<2> {
    /// Computes `a + b mod p` in constant time.
    pub const fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
        base::add2(&self, &rhs, &p)
    }
}

impl UInt<3> {
    /// Computes `a + b mod p` in constant time.
    pub const fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
        base::add3(&self, &rhs, &p)
    }
}

impl UInt<4> {
    /// Computes `a + b mod p` in constant time.
    pub const fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
        base::add4(&self, &rhs, &p)
    }
}

pub(super) mod base {
    use crate::{Limb, UInt};

    pub const fn add1(a: Limb, b: Limb, p: Limb) -> Limb {
        let (l0, _) = a.adc(b, Limb::ZERO);

        // Subtract the modulus, to ensure the result is smaller.
        super::super::sub_mod::base::sub1(l0, p, p)
    }

    pub const fn add2(a: &UInt<2>, b: &UInt<2>, p: &UInt<2>) -> UInt<2> {
        let (l0, carry) = a.limbs[0].adc(b.limbs[0], Limb::ZERO);
        let (l1, _) = a.limbs[1].adc(b.limbs[1], carry);

        // Subtract the modulus, to ensure the result is smaller.
        UInt::new([l0, l1]).sub_mod(p, p)
    }

    pub const fn add3(a: &UInt<3>, b: &UInt<3>, p: &UInt<3>) -> UInt<3> {
        let (l0, carry) = a.limbs[0].adc(b.limbs[0], Limb::ZERO);
        let (l1, carry) = a.limbs[1].adc(b.limbs[1], carry);
        let (l2, _) = a.limbs[2].adc(b.limbs[2], carry);

        // Subtract the modulus, to ensure the result is smaller.
        UInt::new([l0, l1, l2]).sub_mod(p, p)
    }

    pub const fn add4(a: &UInt<4>, b: &UInt<4>, p: &UInt<4>) -> UInt<4> {
        let (l0, carry) = a.limbs[0].adc(b.limbs[0], Limb::ZERO);
        let (l1, carry) = a.limbs[1].adc(b.limbs[1], carry);
        let (l2, carry) = a.limbs[2].adc(b.limbs[2], carry);
        let (l3, _) = a.limbs[3].adc(b.limbs[3], carry);

        // Subtract the modulus, to ensure the result is smaller.
        UInt::new([l0, l1, l2, l3]).sub_mod(p, p)
    }
}
