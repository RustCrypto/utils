// TODO(tarcieri): known to be broken on PPC32. See RustCrypto/utils#1298
#![cfg(not(target_arch = "powerpc"))]

/// Write the proptests for an integer type.
macro_rules! int_proptests {
    ( $($int:ident),+ ) => {
        $(
            mod $int {
                use cmov::{Cmov, CmovEq};
                use proptest::prelude::*;

                proptest! {
                    #[test]
                    fn cmovz_works(mut a in any::<$int>(), b in any::<$int>(), cond in any::<u8>()) {
                        a.cmovz(&b, cond);

                        let expected = if cond == 0 {
                            b
                        } else {
                            a
                        };

                        prop_assert_eq!(expected, a);
                    }

                    #[test]
                    fn cmovnz_works(mut a in any::<$int>(), b in any::<$int>(), cond in any::<u8>()) {
                        a.cmovnz(&b, cond);

                        let expected = if cond != 0 {
                            b
                        } else {
                            a
                        };

                        prop_assert_eq!(expected, a);
                    }

                    #[test]
                     fn cmoveq_works(a in any::<$int>(), b in any::<$int>(), cond in any::<u8>()) {
                        let mut actual = 0;
                        a.cmoveq(&b, cond, &mut actual);

                        let expected = if a == b {
                            cond
                        } else {
                            0
                        };

                        prop_assert_eq!(expected, actual);
                     }

                    #[test]
                     fn cmovne_works(a in any::<$int>(), b in any::<$int>(), cond in any::<u8>()) {
                        let mut actual = 0;
                        a.cmovne(&b, cond, &mut actual);

                        let expected = if a != b {
                            cond
                        } else {
                            0
                        };

                        prop_assert_eq!(expected, actual);
                     }
                }
            }
        )+
    };
}

int_proptests!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
