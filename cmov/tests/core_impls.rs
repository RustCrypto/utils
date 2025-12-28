/// Write the tests for an integer type, given two unequal integers
macro_rules! int_tests {
    ($int:ident, $a:expr, $b:expr) => {
        mod $int {
            use cmov::{Cmov, CmovEq};

            #[test]
            fn cmovz_works() {
                let mut n: $int = $a;

                for cond in 1..0xFF {
                    n.cmovz(&$b, cond);
                    assert_eq!(n, $a);
                }

                n.cmovz(&$b, 0);
                assert_eq!(n, $b);

                n.cmovz(&<$int>::MAX, 0);
                assert_eq!(n, <$int>::MAX);
            }

            #[test]
            fn cmovnz_works() {
                let mut n = $a;
                n.cmovnz(&$b, 0);
                assert_eq!(n, $a);

                for cond in 1..0xFF {
                    let mut n = $a;
                    n.cmovnz(&$b, cond);
                    assert_eq!(n, $b);
                }
            }

            #[test]
            fn cmoveq_works() {
                let mut o = 0u8;

                // compare to zero (a and b should be non-zero)
                $a.cmoveq(&0, 1, &mut o);
                assert_eq!(o, 0);
                0.cmoveq(&$a, 1, &mut o);
                assert_eq!(o, 0);

                for cond in 1..0xFFi64 {
                    cond.cmoveq(&cond, cond as u8, &mut o);
                    assert_eq!(o, cond as u8);
                    cond.cmoveq(&0, 0, &mut o);
                    assert_eq!(o, cond as u8);
                }

                // equal so we move
                $a.cmoveq(&$a, 43u8, &mut o);
                assert_eq!(o, 43u8);

                // non-equal so we don't move
                $a.cmoveq(&$b, 55u8, &mut o);
                assert_eq!(o, 43u8);
                <$int>::MAX.cmoveq(&$a, 55u8, &mut o);
                assert_eq!(o, 43u8);

                // equal so we move
                <$int>::MAX.cmoveq(&<$int>::MAX, 55u8, &mut o);
                assert_eq!(o, 55u8);
            }

            #[test]
            fn cmovne_works() {
                let mut o = 0u8;

                // compare to zero (a and b should be non-zero)
                $a.cmovne(&0, 1, &mut o);
                assert_eq!(o, 1);
                o = 0;
                0.cmovne(&$a, 1, &mut o);
                assert_eq!(o, 1);
                o = 0;

                for cond in 1..0xFFi64 {
                    cond.cmovne(&0, cond as u8, &mut o);
                    assert_eq!(o, cond as u8);
                    cond.cmovne(&cond, 0, &mut o);
                    assert_eq!(o, cond as u8);
                }

                // non-equal so we move
                o = 0;
                $a.cmovne(&$b, 55u8, &mut o);
                assert_eq!(o, 55u8);

                // equal so we don't move
                $a.cmovne(&$a, 66u8, &mut o);
                assert_eq!(o, 55u8);
                <$int>::MAX.cmovne(&<$int>::MAX, 66u8, &mut o);
                assert_eq!(o, 55u8);

                // non-equal so we move
                <$int>::MAX.cmovne(&$a, 66u8, &mut o);
                assert_eq!(o, 66u8);
            }
        }
    };
}

int_tests!(i8, 0x11i8, -0x22i8);
int_tests!(i16, 0x1111i16, -0x2222i16);
int_tests!(i32, 0x1111_1111i32, -0x2222_2222i32);
int_tests!(i64, 0x1111_1111_1111_1111i64, -0x2222_2222_2222_2222i64);
int_tests!(
    i128,
    0x1111_1111_1111_1111_1111_1111_1111_1111i128,
    -0x2222_2222_2222_2222_2222_2222_2222_2222i128
);
int_tests!(u8, 0x11u8, 0x22u8);
int_tests!(u16, 0x1111u16, 0x2222u16);
int_tests!(u32, 0x1111_1111u32, 0x2222_2222u32);
int_tests!(u64, 0x1111_1111_1111_1111u64, 0x2222_2222_2222_2222u64);
int_tests!(
    u128,
    0x1111_1111_1111_1111_2222_2222_2222_2222u128,
    0x2222_2222_2222_2222_3333_3333_3333_3333u128
);

mod slices {
    use cmov::CmovEq;

    #[test]
    fn cmoveq_works() {
        let mut o = 0u8;

        // Same slices.
        [1u8, 2, 3].cmoveq(&[1, 2, 3], 43, &mut o);
        assert_eq!(o, 43);

        // Different lengths.
        [1u8, 2, 3].cmoveq(&[1, 2], 44, &mut o);
        assert_ne!(o, 44);

        // Different contents.
        [1u8, 2, 3].cmoveq(&[1, 2, 4], 45, &mut o);
        assert_ne!(o, 45);
    }

    #[test]
    fn cmovne_works() {
        let mut o = 0u8;

        // Same slices.
        [1u8, 2, 3].cmovne(&[1, 2, 3], 43, &mut o);
        assert_ne!(o, 43);

        // Different lengths.
        [1u8, 2, 3].cmovne(&[1, 2], 44, &mut o);
        assert_eq!(o, 44);

        // Different contents.
        [1u8, 2, 3].cmovne(&[1, 2, 4], 45, &mut o);
        assert_eq!(o, 45);
    }
}
