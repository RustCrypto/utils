//! [`UInt`] addition modulus operations.

use super::UInt;

macro_rules! impl_add_mod {
    ($size:expr, $base_name:ident, $test_name:ident) => {
        impl UInt<$size> {
            /// Computes `a + b mod p` in constant time.
            ///
            /// Assumes `a` and `b` are `< p`.
            pub const fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
                base::$base_name(&self, &rhs, &p)
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
                let base_cases = [(1u64, 0u64, 1u64), (0, 1, 1), (0, 0, 0)];
                for (a, b, c) in &base_cases {
                    let a: UInt<$size> = (*a).into();
                    let b: UInt<$size> = (*b).into();
                    let c: UInt<$size> = (*c).into();

                    assert_eq!(c, a.add_mod(&b, p));
                }

                assert_eq!(p.add_mod(&0u64.into(), p), 0u64.into());
                assert_eq!(p.add_mod(&1u64.into(), p), 1u64.into());

                if $size > 1 {
                    for _i in 0..100 {
                        let a: UInt<$size> = Limb::random(&mut rng).into();
                        let b: UInt<$size> = Limb::random(&mut rng).into();
                        let (a, b) = if a < b { (b, a) } else { (a, b) };

                        let c = a.add_mod(&b, p);
                        assert!(c < *p, "not reduced");
                        assert_eq!(c, a.wrapping_add(&b), "result incorrect");
                    }
                }

                for _i in 0..100 {
                    let a = UInt::<$size>::random_mod(&mut rng, p);
                    let b = UInt::<$size>::random_mod(&mut rng, p);

                    let c = a.add_mod(&b, p);
                    assert!(c < *p, "not reduced: {} >= {} ", c, p);

                    let x = a.wrapping_add(&b);
                    if x < *p {
                        assert_eq!(c, x, "incorrect result");
                    }
                }
            }
        }
    };
}

impl_add_mod!(1, add1, test_add1);
impl_add_mod!(2, add2, test_add2);
impl_add_mod!(3, add3, test_add3);
impl_add_mod!(4, add4, test_add4);
impl_add_mod!(5, add5, test_add5);
impl_add_mod!(6, add6, test_add6);
impl_add_mod!(7, add7, test_add7);
impl_add_mod!(8, add8, test_add8);
impl_add_mod!(9, add9, test_add9);
impl_add_mod!(10, add10, test_add10);
impl_add_mod!(11, add11, test_add11);
impl_add_mod!(12, add12, test_add12);

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
