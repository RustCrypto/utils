//! [`UInt`] multiplication modulus operations.

use super::UInt;
use crate::Limb;

macro_rules! impl_mul_mod {
    ($size:expr, $base_name:ident) => {
        impl UInt<$size> {
            /// Computes `a * b mod p` in constant time.
            ///
            /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
            pub const fn mul_mod(&self, b: &Self, p: &Self, p_inv: Limb) -> Self {
                base::$base_name(&self, &b, p, p_inv)
            }
        }
    };
}

impl_mul_mod!(1, mul1);
impl_mul_mod!(2, mul2);
impl_mul_mod!(3, mul3);
impl_mul_mod!(4, mul4);
impl_mul_mod!(5, mul5);
impl_mul_mod!(6, mul6);
impl_mul_mod!(7, mul7);
impl_mul_mod!(8, mul8);
impl_mul_mod!(9, mul9);
impl_mul_mod!(10, mul10);
impl_mul_mod!(11, mul11);
impl_mul_mod!(12, mul12);

pub(super) mod base {
    use crate::{Limb, UInt};

    macro_rules! impl_base {
        ($size:expr, $name:ident) => {
            pub const fn $name(
                a: &UInt<$size>,
                b: &UInt<$size>,
                p: &UInt<$size>,
                p_inv: Limb,
            ) -> UInt<$size> {
                mul(a, b, p, p_inv)
            }
        };
    }

    impl_base!(1, mul1);
    impl_base!(2, mul2);
    impl_base!(3, mul3);
    impl_base!(4, mul4);
    impl_base!(5, mul5);
    impl_base!(6, mul6);
    impl_base!(7, mul7);
    impl_base!(8, mul8);
    impl_base!(9, mul9);
    impl_base!(10, mul10);
    impl_base!(11, mul11);
    impl_base!(12, mul12);

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
