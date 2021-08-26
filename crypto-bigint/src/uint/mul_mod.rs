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
    /// Assumes `self` and `rhs` are both less than `p`.
    /// Assumes `self` and `rhs` are in montgomery space, the result will be again.
    pub fn mul_mod(&self, rhs: &UInt<LIMBS>, p: &UInt<LIMBS>, p_inv: Limb) -> UInt<LIMBS> {
        std::println!("start");
        // Schoolbook multiplication
        let mut r_lo = [Limb::ZERO; LIMBS];
        let mut r_hi = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            std::dbg!(i);
            let mut carry = Limb::ZERO;
            let mut j = 0;

            while j < LIMBS - 1 {
                std::dbg!((i, j));
                let (r_ij, c) = get(&r_lo, &r_hi, i + j).mac(self.limbs[i], rhs.limbs[j], carry);
                set!(r_lo, r_hi, i + j, r_ij);
                carry = c;

                j += 1;
            }

            std::dbg!(i, j, i + j);
            let a = self.limbs[i];
            let b = rhs.limbs[j];
            let (r_ij, c) = get(&r_lo, &r_hi, i + j).mac(a, b, carry);
            set!(r_lo, r_hi, i + j, r_ij);
            set!(r_lo, r_hi, i + j + 1, c);

            i += 1;
        }

        std::dbg!(&r_lo, &r_hi);
        std::println!("end");
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
fn mont_reduce<const LIMBS: usize>(
    mut r_lo: [Limb; LIMBS],
    mut r_hi: [Limb; LIMBS],
    p: &UInt<LIMBS>,
    p_inv: Limb,
) -> UInt<LIMBS> {
    let mut carry2 = Limb::ZERO;
    let mut i = 0;

    while i < LIMBS {
        std::dbg!(i);
        let k = get(&r_lo, &r_hi, i).wrapping_mul(p_inv);

        let mut carry = Limb::ZERO;
        let mut j = 0;

        while j < LIMBS {
            std::dbg!((i, j));
            let (rj, c) = get(&r_lo, &r_hi, i + j).mac(k, p.limbs[j], carry);
            if j > 0 {
                set!(r_lo, r_hi, i + j, rj);
            }
            carry = c;

            j += 1;
        }

        std::dbg!(i + j);
        let (rj, c) = get(&r_lo, &r_hi, i + j).adc(carry2, carry);
        set!(r_lo, r_hi, i + j, rj);
        carry2 = c;

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

#[cfg(test)]
mod tests {
    use crate::{Encoding, Limb, MulMod, U256};

    const P_BLS: U256 =
        U256::from_be_hex("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");
    const P_BLS_INV: Limb = Limb(0xffff_fffe_ffff_ffff);

    const LARGEST: U256 =
        U256::from_be_hex("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000");

    /// R^2 = 2^512 mod q
    const R2_BLS: U256 =
        U256::from_be_hex("0748d9d99f59ff1105d314967254398f2b6cedcb87925c23c999e990f3f29c6d");

    #[test]
    fn test_mul_mod_bls12_381_scalar_simple() {
        let two: U256 = 2u32.into();
        let three: U256 = 3u32.into();
        let six: U256 = 6u32.into();

        assert_eq!(
            six.wrapping_mul(&R2_BLS),
            two.wrapping_mul(&R2_BLS)
                .mul_mod(&three.wrapping_mul(&R2_BLS), &P_BLS, P_BLS_INV)
        );
    }

    #[test]
    fn test_mul_mod_bls12_381_scalar_complex() {
        let mut cur = LARGEST;

        for _ in 0..100 {
            let mut tmp = cur;
            tmp = tmp.mul_mod(&cur, &P_BLS, P_BLS_INV);

            let mut tmp2 = U256::ZERO;
            for b in cur
                .to_le_bytes()
                .iter()
                .rev()
                .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1u8) == 1u8))
            {
                tmp2 = tmp2.add_mod(&tmp2, &P_BLS);

                if b {
                    tmp2 = tmp2.add_mod(&cur, &P_BLS);
                }
            }

            assert_eq!(tmp, tmp2);

            cur = cur.add_mod(&LARGEST, &P_BLS);
        }
    }
}
