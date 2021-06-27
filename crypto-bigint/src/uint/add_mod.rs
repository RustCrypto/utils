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
        add(a, b, p)
    }

    pub const fn add3(a: &UInt<3>, b: &UInt<3>, p: &UInt<3>) -> UInt<3> {
        add(a, b, p)
    }

    pub const fn add4(a: &UInt<4>, b: &UInt<4>, p: &UInt<4>) -> UInt<4> {
        add(a, b, p)
    }

    pub const fn add<const LIMBS: usize>(
        a: &UInt<LIMBS>,
        b: &UInt<LIMBS>,
        p: &UInt<LIMBS>,
    ) -> UInt<LIMBS> {
        let mut out = [Limb::ZERO; LIMBS];
        let mut carry = Limb::ZERO;
        let mut i = 0;
        while i < LIMBS {
            let (l, c) = a.limbs[i].adc(b.limbs[i], carry);
            out[i] = l;
            carry = c;
            i += 1;
        }

        // Subtract the modulus, to ensure the result is smaller.
        super::super::sub_mod::base::sub(&UInt::new(out), p, p)
    }
}
