//! [`UInt`] subtraction modulus operations.

use super::UInt;

macro_rules! impl_neg_mod {
    ($size:expr, $base_name:ident) => {
        impl UInt<$size> {
            /// Computes `-a mod p` in constant time.
            pub const fn neg_mod(&self, p: &Self) -> Self {
                base::$base_name(self, p)
            }
        }
    };
}

impl_neg_mod!(1, neg1);
impl_neg_mod!(2, neg2);
impl_neg_mod!(3, neg3);
impl_neg_mod!(4, neg4);
impl_neg_mod!(5, neg5);
impl_neg_mod!(6, neg6);
impl_neg_mod!(7, neg7);
impl_neg_mod!(8, neg8);
impl_neg_mod!(9, neg9);
impl_neg_mod!(10, neg10);
impl_neg_mod!(11, neg11);
impl_neg_mod!(12, neg12);

pub(super) mod base {
    use crate::{Limb, UInt};

    macro_rules! impl_base {
        ($size:expr, $name:ident) => {
            pub const fn $name(a: &UInt<$size>, p: &UInt<$size>) -> UInt<$size> {
                neg(a, p)
            }
        };
    }

    impl_base!(1, neg1);
    impl_base!(2, neg2);
    impl_base!(3, neg3);
    impl_base!(4, neg4);
    impl_base!(5, neg5);
    impl_base!(6, neg6);
    impl_base!(7, neg7);
    impl_base!(8, neg8);
    impl_base!(9, neg9);
    impl_base!(10, neg10);
    impl_base!(11, neg11);
    impl_base!(12, neg12);

    pub const fn neg<const LIMBS: usize>(a: &UInt<LIMBS>, p: &UInt<LIMBS>) -> UInt<LIMBS> {
        let mut tmp = [Limb::ZERO; LIMBS];

        // Subtract `a` from `p` to negate. Ignore the final
        // borrow because it cannot underflow; a is guaranteed to
        // be in the field.

        let mut borrow = Limb::ZERO;
        let mut i = 0;
        while i < LIMBS {
            let (l, b) = p.limbs[i].sbb(a.limbs[i], borrow);
            tmp[i] = l;
            borrow = b;

            i += 1;
        }

        // `tmp` could be `p` if `a` was zero. Create a mask that is
        // zero if `a` was zero, and `Limb::MAX` if self was nonzero.

        // FIXME: constant time comparison
        let mut a_or = a.limbs[0];
        let mut i = 1;
        while i < LIMBS {
            a_or = a_or.bitor(a.limbs[i]);

            i += 1;
        }

        let v = if a_or.eq_vartime(&Limb::ZERO) {
            Limb::ONE
        } else {
            Limb::ZERO
        };
        let mask = v.wrapping_sub(Limb::ONE);

        let mut i = 0;
        while i < LIMBS {
            tmp[i] = tmp[i].bitand(mask);

            i += 1;
        }

        UInt::new(tmp)
    }
}
