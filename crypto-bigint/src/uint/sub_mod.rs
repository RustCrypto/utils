//! [`UInt`] subtraction modulus operations.

use super::UInt;

macro_rules! impl_sub_mod {
    ($size:expr, $base_name:ident, $test_name:ident) => {
        impl UInt<$size> {
            /// Computes `a - b mod p` in constant time.
            ///
            /// Assumes `a` and `b` are `< p`.
            pub const fn sub_mod(&self, rhs: &Self, p: &Self) -> Self {
                base::$base_name(self, rhs, p)
            }
        }

        #[cfg(all(test, feature = "rand"))]
        #[test]
        fn $test_name() {
            use crate::Limb;
            use rand_core::SeedableRng;

            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(1);

            let moduli = [UInt::<$size>::random(&mut rng), UInt::random(&mut rng)];

            for p in &moduli {
                let base_cases = [
                    (1u64, 0u64, 1u64.into()),
                    (0, 1, p.wrapping_sub(&1u64.into())),
                    (0, 0, 0u64.into()),
                ];
                for (a, b, c) in &base_cases {
                    let a: UInt<$size> = (*a).into();
                    let b: UInt<$size> = (*b).into();

                    let x = a.sub_mod(&b, p);
                    assert_eq!(*c, x, "{} - {} mod {} = {} != {}", a, b, p, x, c);
                }

                if $size > 1 {
                    for _i in 0..100 {
                        let a: UInt<$size> = Limb::random(&mut rng).into();
                        let b: UInt<$size> = Limb::random(&mut rng).into();
                        let (a, b) = if a < b { (b, a) } else { (a, b) };

                        let c = a.sub_mod(&b, p);
                        assert!(c < *p, "not reduced");
                        assert_eq!(c, a.wrapping_sub(&b), "result incorrect");
                    }
                }

                for _i in 0..100 {
                    let a = UInt::<$size>::random_mod(&mut rng, p);
                    let b = UInt::<$size>::random_mod(&mut rng, p);

                    let c = a.sub_mod(&b, p);
                    assert!(c < *p, "not reduced: {} >= {} ", c, p);

                    let x = a.wrapping_sub(&b);
                    if a >= b && x < *p {
                        assert_eq!(c, x, "incorrect result");
                    }
                }
            }
        }
    };
}

impl_sub_mod!(1, sub1, test_sub1);
impl_sub_mod!(2, sub2, test_sub2);
impl_sub_mod!(3, sub3, test_sub3);
impl_sub_mod!(4, sub4, test_sub4);
impl_sub_mod!(5, sub5, test_sub5);
impl_sub_mod!(6, sub6, test_sub6);
impl_sub_mod!(7, sub7, test_sub7);
impl_sub_mod!(8, sub8, test_sub8);
impl_sub_mod!(9, sub9, test_sub9);
impl_sub_mod!(10, sub10, test_sub10);
impl_sub_mod!(11, sub11, test_sub11);
impl_sub_mod!(12, sub12, test_sub12);

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
