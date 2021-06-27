//! [`UInt`] multiplication modulus operations.

use super::UInt;
use crate::Limb;

impl UInt<1> {
    /// Computes `a * b mod p` in constant time.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    pub const fn mul_mod(&self, b: &Self, p: &Self, p_inv: Limb) -> Self {
        base::mul1(&self, &b, p, p_inv)
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

    pub const fn mul1(a: &UInt<1>, b: &UInt<1>, p: &UInt<1>, p_inv: Limb) -> UInt<1> {
        mul(a, b, p, p_inv)
    }

    pub const fn mul2(a: &UInt<2>, b: &UInt<2>, p: &UInt<2>, p_inv: Limb) -> UInt<2> {
        mul(a, b, p, p_inv)
    }

    pub const fn mul3(a: &UInt<3>, b: &UInt<3>, p: &UInt<3>, p_inv: Limb) -> UInt<3> {
        mul(a, b, p, p_inv)
    }

    pub const fn mul4(a: &UInt<4>, b: &UInt<4>, p: &UInt<4>, p_inv: Limb) -> UInt<4> {
        mul(a, b, p, p_inv)
    }

    // macro, because can't use &mut in const yet
    macro_rules! set {
        ($lo:expr, $hi:expr, $i:expr, $r:expr) => {
            if $i < LIMBS {
                $lo[$i] = $r;
            } else {
                $hi[$i - LIMBS] = $r;
            }
        };
    }

    #[inline]
    const fn get<const LIMBS: usize>(lo: &[Limb; LIMBS], hi: &[Limb; LIMBS], i: usize) -> Limb {
        if i < LIMBS {
            lo[i]
        } else {
            hi[i - LIMBS]
        }
    }

    pub const fn mul<const LIMBS: usize>(
        a: &UInt<LIMBS>,
        b: &UInt<LIMBS>,
        p: &UInt<LIMBS>,
        p_inv: Limb,
    ) -> UInt<LIMBS> {
        // Schoolbook multiplication

        let mut r_lo = [Limb::ZERO; LIMBS];
        let mut r_hi = [Limb::ZERO; LIMBS];

        let mut i = 0;

        while i < LIMBS {
            let mut carry = Limb::ZERO;
            let mut j = 0;

            while j < LIMBS - 1 {
                let (r_ij, c) = get(&r_lo, &r_hi, i + j).mac(a.limbs[i], b.limbs[j], carry);
                set!(r_lo, r_hi, i + j, r_ij);
                carry = c;

                j += 1;
            }

            let (r_ij, c) = get(&r_lo, &r_hi, i + j).mac(a.limbs[i], b.limbs[j], carry);
            set!(r_lo, r_hi, i + j, r_ij);
            set!(r_lo, r_hi, i + j + 1, c);

            i += 1;
        }

        mont_reduce(r_lo, r_hi, p, p_inv)
    }

    #[inline(always)]
    const fn mont_reduce<const LIMBS: usize>(
        mut r_lo: [Limb; LIMBS],
        mut r_hi: [Limb; LIMBS],
        p: &UInt<LIMBS>,
        p_inv: Limb,
    ) -> UInt<LIMBS> {
        // The Montgomery reduction here is based on Algorithm 14.32 in
        // Handbook of Applied Cryptography
        // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

        let mut carry = Limb::ZERO;
        let mut carry2 = Limb::ZERO;
        let mut i = 0;

        while i < LIMBS {
            let k = get(&r_lo, &r_hi, i).wrapping_mul(p_inv);

            let mut j = 0;
            while j < LIMBS - 1 {
                let (rj, c) = get(&r_lo, &r_hi, i + j).mac(k, p.limbs[j], Limb::ZERO);
                carry = c;
                if j > 0 {
                    set!(r_lo, r_hi, i + j, rj);
                }
                j += 1;
            }
            let (rj, c) = get(&r_lo, &r_hi, i + j).adc(carry2, carry);
            carry2 = c;
            set!(r_lo, r_hi, i + j, rj);

            i += 1;
        }

        // Result may be within p of the correct value
        super::super::sub_mod::base::sub(&UInt::new(r_hi), p, p)
    }
}
