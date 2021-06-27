//! [`UInt`] addition modulus operations.

use super::UInt;

macro_rules! impl_add_mod {
    ($size:expr, $base_name:ident) => {
        impl UInt<$size> {
            /// Computes `a + b mod p` in constant time.
            pub const fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
                base::$base_name(&self, &rhs, &p)
            }
        }
    };
}

impl_add_mod!(1, add1);
impl_add_mod!(2, add2);
impl_add_mod!(3, add3);
impl_add_mod!(4, add4);
impl_add_mod!(5, add5);
impl_add_mod!(6, add6);
impl_add_mod!(7, add7);
impl_add_mod!(8, add8);
impl_add_mod!(9, add9);
impl_add_mod!(10, add10);
impl_add_mod!(11, add11);
impl_add_mod!(12, add12);

pub(super) mod base {
    use crate::{Limb, UInt};

    macro_rules! impl_base {
        ($size:expr, $name:ident) => {
            pub const fn $name(a: &UInt<$size>, b: &UInt<$size>, p: &UInt<$size>) -> UInt<$size> {
                add(a, b, p)
            }
        };
    }

    impl_base!(1, add1);
    impl_base!(2, add2);
    impl_base!(3, add3);
    impl_base!(4, add4);
    impl_base!(5, add5);
    impl_base!(6, add6);
    impl_base!(7, add7);
    impl_base!(8, add8);
    impl_base!(9, add9);
    impl_base!(10, add10);
    impl_base!(11, add11);
    impl_base!(12, add12);

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
