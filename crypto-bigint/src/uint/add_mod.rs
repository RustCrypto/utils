//! [`UInt`] addition modulus operations.

use crate::{AddMod, Limb, UInt};

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Computes `self + rhs mod p` in constant time.
    ///
    /// Assumes `self` and `rhs` are `< p`.
    pub const fn add_mod(&self, rhs: &UInt<LIMBS>, p: &UInt<LIMBS>) -> UInt<LIMBS> {
        let (out, _carry) = self.adc(rhs, Limb::ZERO);

        // Subtract the modulus, to ensure the result is smaller.
        out.sub_mod(p, p)
    }
}

macro_rules! impl_add_mod {
    ($($size:expr),+) => {
        $(
            impl AddMod for UInt<$size> {
                type Output = Self;

                fn add_mod(&self, rhs: &Self, p: &Self) -> Self {
                    debug_assert!(self < p);
                    debug_assert!(rhs < p);
                    self.add_mod(rhs, p)
                }
            }
        )+
    };
}

impl_add_mod!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);

#[cfg(all(test, feature = "rand"))]
mod tests {
    use crate::{UInt, U256};

    macro_rules! test_add_mod {
        ($size:expr, $test_name:ident) => {
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

    // Test requires 1-limb is capable of representing a 64-bit integer
    #[cfg(target_pointer_width = "64")]
    test_add_mod!(1, test_add1);

    test_add_mod!(2, test_add2);
    test_add_mod!(3, test_add3);
    test_add_mod!(4, test_add4);
    test_add_mod!(5, test_add5);
    test_add_mod!(6, test_add6);
    test_add_mod!(7, test_add7);
    test_add_mod!(8, test_add8);
    test_add_mod!(9, test_add9);
    test_add_mod!(10, test_add10);
    test_add_mod!(11, test_add11);
    test_add_mod!(12, test_add12);

    #[test]
    fn add_mod_nist_p256() {
        let a =
            U256::from_be_hex("44acf6b7e36c1342c2c5897204fe09504e1e2efb1a900377dbc4e7a6a133ec56");
        let b =
            U256::from_be_hex("d5777c45019673125ad240f83094d4252d829516fac8601ed01979ec1ec1a251");
        let n =
            U256::from_be_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551");

        let actual = a.add_mod(&b, &n);
        let expected =
            U256::from_be_hex("1a2472fde50286541d97ca6a3592dd75beb9c9646e40c511b82496cfc3926956");

        assert_eq!(expected, actual);
    }
}
