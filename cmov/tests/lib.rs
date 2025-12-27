/// Write the tests for an integer type, given two unequal integers
macro_rules! int_tests {
    ($a:expr, $b:expr) => {
        use cmov::{Cmov as _, CmovEq as _};

        #[test]
        fn cmovz_works() {
            let mut n = $a;

            for cond in 1..0xFF {
                n.cmovz(&$b, cond);
                assert_eq!(n, $a);
            }

            n.cmovz(&$b, 0);
            assert_eq!(n, $b);
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
        }

        #[test]
        fn cmovne_works() {
            let mut o = 0u8;

            for cond in 1..0xFFi64 {
                cond.cmovne(&0, cond as u8, &mut o);
                assert_eq!(o, cond as u8);
                cond.cmovne(&cond, 0, &mut o);
                assert_eq!(o, cond as u8);
            }

            // non-equal so we move
            $a.cmovne(&$b, 55u8, &mut o);
            assert_eq!(o, 55u8);

            // equal so we don't move
            $a.cmovne(&$a, 12u8, &mut o);
            assert_eq!(o, 55u8);
        }
    };
}

mod i8 {
    pub const I8_A: i64 = 0x11;
    pub const I8_B: i64 = -0x22;
    int_tests!(I8_A, I8_B);
}

mod i16 {
    pub const I16_A: i64 = 0x1111;
    pub const I16_B: i64 = -0x2222;
    int_tests!(I16_A, I16_B);
}

mod i32 {
    pub const I32_A: i32 = 0x1111_1111;
    pub const I32_B: i32 = -0x2222_2222;
    int_tests!(I32_A, I32_B);
}

mod i64 {
    pub const I64_A: i64 = 0x1111_1111_1111_1111;
    pub const I64_B: i64 = -0x2222_2222_2222_2222;
    int_tests!(I64_A, I64_B);
}

mod i128 {
    pub const I128_A: i128 = 0x1111_1111_1111_1111_1111_1111_1111_1111;
    pub const I128_B: i128 = -0x2222_2222_2222_2222_2222_2222_2222_2222;
    int_tests!(I128_A, I128_B);
}

mod u8 {
    pub const U8_A: u8 = 0x11;
    pub const U8_B: u8 = 0x22;
    int_tests!(U8_A, U8_B);
}

mod u16 {
    pub const U16_A: u16 = 0x1111;
    pub const U16_B: u16 = 0x2222;
    int_tests!(U16_A, U16_B);
}

mod u32 {
    pub const U32_A: u32 = 0x1111_1111;
    pub const U32_B: u32 = 0x2222_2222;
    int_tests!(U32_A, U32_B);
}

mod u64 {
    pub const U64_A: u64 = 0x1111_1111_1111_1111;
    pub const U64_B: u64 = 0x2222_2222_2222_2222;
    int_tests!(U64_A, U64_B);
}

mod u128 {
    pub const U128_A: u128 = 0x1111_1111_1111_1111_2222_2222_2222_2222;
    pub const U128_B: u128 = 0x2222_2222_2222_2222_3333_3333_3333_3333;
    int_tests!(U128_A, U128_B);
}

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
