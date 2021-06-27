//! [`UInt`] multiplication modulus operations.

use super::UInt;
use crate::Limb;

impl UInt<1> {
    /// Computes `a * b mod p` in constant time.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    pub const fn mul_mod(&self, b: &Self, p: &Self, p_inv: Limb) -> Self {
        base::mul1(self.limbs[0], b.limbs[0], p, p_inv)
    }
}

impl UInt<2> {
    /// Computes `a * b mod p` in constant time.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    pub const fn mul_mod(&self, b: &Self, p: &Self, p_inv: Limb) -> Self {
        base::mul2(self, b, p, p_inv)
    }
}

impl UInt<3> {
    /// Computes `a * b mod p` in constant time.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    pub const fn mul_mod(&self, b: &Self, p: &Self, p_inv: Limb) -> Self {
        base::mul3(self, b, p, p_inv)
    }
}

impl UInt<4> {
    /// Computes `a * b mod p` in constant time.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    pub const fn mul_mod(&self, b: &Self, p: &Self, p_inv: Limb) -> Self {
        base::mul4(self, b, p, p_inv)
    }
}

pub(super) mod base {
    use crate::{Limb, UInt};

    pub const fn mul1(a: Limb, b: Limb, p: &UInt<1>, p_inv: Limb) -> UInt<1> {
        // Schoolbook multiplication

        let (r0, r1) = Limb::ZERO.mac(a, b, Limb::ZERO);

        mont_reduce1(r0, r1, p, p_inv)
    }

    pub const fn mul2(a: &UInt<2>, b: &UInt<2>, p: &UInt<2>, p_inv: Limb) -> UInt<2> {
        // Schoolbook multiplication

        let (r0, carry) = Limb::ZERO.mac(a.limbs[0], b.limbs[0], Limb::ZERO);
        let (r1, r2) = Limb::ZERO.mac(a.limbs[0], b.limbs[1], carry);

        let (r1, carry) = r1.mac(a.limbs[1], b.limbs[0], Limb::ZERO);
        let (r2, r3) = r2.mac(a.limbs[1], b.limbs[1], carry);

        mont_reduce2(r0, r1, r2, r3, p, p_inv)
    }

    pub const fn mul3(a: &UInt<3>, b: &UInt<3>, p: &UInt<3>, p_inv: Limb) -> UInt<3> {
        // Schoolbook multiplication

        let (r0, carry) = Limb::ZERO.mac(a.limbs[0], b.limbs[0], Limb::ZERO);
        let (r1, carry) = Limb::ZERO.mac(a.limbs[0], b.limbs[1], carry);
        let (r2, r3) = Limb::ZERO.mac(a.limbs[0], b.limbs[2], carry);

        let (r1, carry) = r1.mac(a.limbs[1], b.limbs[0], Limb::ZERO);
        let (r2, carry) = r2.mac(a.limbs[1], b.limbs[1], carry);
        let (r3, r4) = r3.mac(a.limbs[1], b.limbs[2], carry);

        let (r2, carry) = r2.mac(a.limbs[2], b.limbs[0], Limb::ZERO);
        let (r3, carry) = r3.mac(a.limbs[2], b.limbs[1], carry);
        let (r4, r5) = r4.mac(a.limbs[2], b.limbs[2], carry);

        mont_reduce3(r0, r1, r2, r3, r4, r5, p, p_inv)
    }

    pub const fn mul4(a: &UInt<4>, b: &UInt<4>, p: &UInt<4>, p_inv: Limb) -> UInt<4> {
        // Schoolbook multiplication

        let (r0, carry) = Limb::ZERO.mac(a.limbs[0], b.limbs[0], Limb::ZERO);
        let (r1, carry) = Limb::ZERO.mac(a.limbs[0], b.limbs[1], carry);
        let (r2, carry) = Limb::ZERO.mac(a.limbs[0], b.limbs[2], carry);
        let (r3, r4) = Limb::ZERO.mac(a.limbs[0], b.limbs[3], carry);

        let (r1, carry) = r1.mac(a.limbs[1], b.limbs[0], Limb::ZERO);
        let (r2, carry) = r2.mac(a.limbs[1], b.limbs[1], carry);
        let (r3, carry) = r3.mac(a.limbs[1], b.limbs[2], carry);
        let (r4, r5) = r4.mac(a.limbs[1], b.limbs[3], carry);

        let (r2, carry) = r2.mac(a.limbs[2], b.limbs[0], Limb::ZERO);
        let (r3, carry) = r3.mac(a.limbs[2], b.limbs[1], carry);
        let (r4, carry) = r4.mac(a.limbs[2], b.limbs[2], carry);
        let (r5, r6) = r5.mac(a.limbs[2], b.limbs[3], carry);

        let (r3, carry) = r3.mac(a.limbs[3], b.limbs[0], Limb::ZERO);
        let (r4, carry) = r4.mac(a.limbs[3], b.limbs[1], carry);
        let (r5, carry) = r5.mac(a.limbs[3], b.limbs[2], carry);
        let (r6, r7) = r6.mac(a.limbs[3], b.limbs[3], carry);

        mont_reduce4(r0, r1, r2, r3, r4, r5, r6, r7, p, p_inv)
    }

    #[inline(always)]
    const fn mont_reduce1(r0: Limb, r1: Limb, p: &UInt<1>, p_inv: Limb) -> UInt<1> {
        // The Montgomery reduction here is based on Algorithm 14.32 in
        // Handbook of Applied Cryptography
        // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

        let k = r0.wrapping_mul(p_inv);
        let (_, carry) = r0.mac(k, p.limbs[0], Limb::ZERO);
        let (r1, _) = r1.adc(Limb::ZERO, carry);

        // Result may be within p of the correct value
        UInt::new([r1]).sub_mod(p, p)
    }

    #[inline(always)]
    const fn mont_reduce2(
        r0: Limb,
        r1: Limb,
        r2: Limb,
        r3: Limb,
        p: &UInt<2>,
        p_inv: Limb,
    ) -> UInt<2> {
        // The Montgomery reduction here is based on Algorithm 14.32 in
        // Handbook of Applied Cryptography
        // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

        let k = r0.wrapping_mul(p_inv);
        let (_, carry) = r0.mac(k, p.limbs[0], Limb::ZERO);
        let (r1, carry) = r1.mac(k, p.limbs[1], carry);
        let (r2, carry2) = r2.adc(Limb::ZERO, carry);

        let k = r1.wrapping_mul(p_inv);
        let (_, carry) = r1.mac(k, p.limbs[0], Limb::ZERO);
        let (r2, carry) = r2.mac(k, p.limbs[1], carry);
        let (r3, _) = r3.adc(carry2, carry);

        // Result may be within p of the correct value
        UInt::new([r2, r3]).sub_mod(p, p)
    }

    #[inline(always)]
    const fn mont_reduce3(
        r0: Limb,
        r1: Limb,
        r2: Limb,
        r3: Limb,
        r4: Limb,
        r5: Limb,
        p: &UInt<3>,
        p_inv: Limb,
    ) -> UInt<3> {
        // The Montgomery reduction here is based on Algorithm 14.32 in
        // Handbook of Applied Cryptography
        // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

        let k = r0.wrapping_mul(p_inv);
        let (_, carry) = r0.mac(k, p.limbs[0], Limb::ZERO);
        let (r1, carry) = r1.mac(k, p.limbs[1], carry);
        let (r2, carry) = r2.mac(k, p.limbs[2], carry);
        let (r3, carry2) = r3.adc(Limb::ZERO, carry);

        let k = r1.wrapping_mul(p_inv);
        let (_, carry) = r1.mac(k, p.limbs[0], Limb::ZERO);
        let (r2, carry) = r2.mac(k, p.limbs[1], carry);
        let (r3, carry) = r3.mac(k, p.limbs[2], carry);
        let (r4, carry2) = r4.adc(carry2, carry);

        let k = r2.wrapping_mul(p_inv);
        let (_, carry) = r2.mac(k, p.limbs[0], Limb::ZERO);
        let (r3, carry) = r3.mac(k, p.limbs[1], carry);
        let (r4, carry) = r4.mac(k, p.limbs[2], carry);
        let (r5, _) = r5.adc(carry2, carry);

        // Result may be within p of the correct value
        UInt::new([r3, r4, r5]).sub_mod(p, p)
    }

    #[inline(always)]
    const fn mont_reduce4(
        r0: Limb,
        r1: Limb,
        r2: Limb,
        r3: Limb,
        r4: Limb,
        r5: Limb,
        r6: Limb,
        r7: Limb,
        p: &UInt<4>,
        p_inv: Limb,
    ) -> UInt<4> {
        // The Montgomery reduction here is based on Algorithm 14.32 in
        // Handbook of Applied Cryptography
        // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

        let k = r0.wrapping_mul(p_inv);
        let (_, carry) = r0.mac(k, p.limbs[0], Limb::ZERO);
        let (r1, carry) = r1.mac(k, p.limbs[1], carry);
        let (r2, carry) = r2.mac(k, p.limbs[2], carry);
        let (r3, carry) = r3.mac(k, p.limbs[3], carry);
        let (r4, carry2) = r4.adc(Limb::ZERO, carry);

        let k = r1.wrapping_mul(p_inv);
        let (_, carry) = r1.mac(k, p.limbs[0], Limb::ZERO);
        let (r2, carry) = r2.mac(k, p.limbs[1], carry);
        let (r3, carry) = r3.mac(k, p.limbs[2], carry);
        let (r4, carry) = r4.mac(k, p.limbs[3], carry);
        let (r5, carry2) = r5.adc(carry2, carry);

        let k = r2.wrapping_mul(p_inv);
        let (_, carry) = r2.mac(k, p.limbs[0], Limb::ZERO);
        let (r3, carry) = r3.mac(k, p.limbs[1], carry);
        let (r4, carry) = r4.mac(k, p.limbs[2], carry);
        let (r5, carry) = r5.mac(k, p.limbs[3], carry);
        let (r6, carry2) = r6.adc(carry2, carry);

        let k = r3.wrapping_mul(p_inv);
        let (_, carry) = r3.mac(k, p.limbs[0], Limb::ZERO);
        let (r4, carry) = r4.mac(k, p.limbs[1], carry);
        let (r5, carry) = r5.mac(k, p.limbs[2], carry);
        let (r6, carry) = r6.mac(k, p.limbs[3], carry);
        let (r7, _) = r7.adc(carry2, carry);

        // Result may be within p of the correct value
        UInt::new([r4, r5, r6, r7]).sub_mod(p, p)
    }
}
