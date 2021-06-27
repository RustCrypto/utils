//! [`UInt`] subtraction modulus operations.

use super::UInt;

macro_rules! impl_sub_mod {
    ($size:expr, $base_name:ident) => {
        impl UInt<$size> {
            /// Computes `a - b mod p` in constant time.
            pub const fn sub_mod(&self, rhs: &Self, p: &Self) -> Self {
                base::$base_name(self, rhs, p)
            }
        }
    };
}

impl_sub_mod!(1, sub1);
impl_sub_mod!(2, sub2);
impl_sub_mod!(3, sub3);
impl_sub_mod!(4, sub4);
impl_sub_mod!(5, sub5);
impl_sub_mod!(6, sub6);
impl_sub_mod!(7, sub7);
impl_sub_mod!(8, sub8);
impl_sub_mod!(9, sub9);
impl_sub_mod!(10, sub10);
impl_sub_mod!(11, sub11);
impl_sub_mod!(12, sub12);

pub(super) mod base {
    use crate::{Limb, UInt};

    macro_rules! impl_base {
        ($size:expr, $name:ident) => {
            pub const fn $name(a: &UInt<$size>, b: &UInt<$size>, p: &UInt<$size>) -> UInt<$size> {
                sub(a, b, p)
            }
        };
    }

    impl_base!(1, sub1);
    impl_base!(2, sub2);
    impl_base!(3, sub3);
    impl_base!(4, sub4);
    impl_base!(5, sub5);
    impl_base!(6, sub6);
    impl_base!(7, sub7);
    impl_base!(8, sub8);
    impl_base!(9, sub9);
    impl_base!(10, sub10);
    impl_base!(11, sub11);
    impl_base!(12, sub12);

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
