//! [`UInt`] multiplication modulus operations.

use crate::{Limb, MulMod, UInt};

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

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Computes `self * rhs mod p` in constant time.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    pub const fn mul_mod(&self, rhs: &UInt<LIMBS>, p: &UInt<LIMBS>, p_inv: Limb) -> UInt<LIMBS> {
        // Schoolbook multiplication
        let mut r_lo = [Limb::ZERO; LIMBS];
        let mut r_hi = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let mut carry = Limb::ZERO;
            let mut j = 0;

            while j < LIMBS - 1 {
                let (r_ij, c) = get(&r_lo, &r_hi, i + j).mac(self.limbs[i], rhs.limbs[j], carry);
                set!(r_lo, r_hi, i + j, r_ij);
                carry = c;

                j += 1;
            }

            let (r_ij, c) = get(&r_lo, &r_hi, i + j).mac(self.limbs[i], rhs.limbs[j], carry);
            set!(r_lo, r_hi, i + j, r_ij);
            set!(r_lo, r_hi, i + j + 1, c);

            i += 1;
        }

        mont_reduce(r_lo, r_hi, p, p_inv)
    }
}

#[inline]
const fn get<const LIMBS: usize>(lo: &[Limb; LIMBS], hi: &[Limb; LIMBS], i: usize) -> Limb {
    if i < LIMBS {
        lo[i]
    } else {
        hi[i - LIMBS]
    }
}

/// The Montgomery reduction here is based on Algorithm 14.32 in
/// Handbook of Applied Cryptography
/// <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.
#[inline(always)]
const fn mont_reduce<const LIMBS: usize>(
    mut r_lo: [Limb; LIMBS],
    mut r_hi: [Limb; LIMBS],
    p: &UInt<LIMBS>,
    p_inv: Limb,
) -> UInt<LIMBS> {
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
    UInt::new(r_hi).sub_mod(p, p)
}

macro_rules! impl_mul_mod {
    ($($size:expr),+) => {
        $(
            impl MulMod for UInt<$size> {
                type Output = Self;

                fn mul_mod(&self, rhs: &Self, p: &Self, p_inv: Limb) -> Self {
                    debug_assert!(self < p);
                    debug_assert!(rhs < p);
                    self.mul_mod(rhs, p, p_inv)
                }
            }
        )+
    };
}

impl_mul_mod!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
