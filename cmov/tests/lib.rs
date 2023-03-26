mod u8 {
    use cmov::{Cmov, CmovEq};

    pub const U8_A: u8 = 0x11;
    pub const U8_B: u8 = 0x22;

    #[test]
    fn cmovz_works() {
        let mut n = U8_A;

        for cond in 1..0xFF {
            n.cmovz(&U8_B, cond);
            assert_eq!(n, U8_A);
        }

        n.cmovz(&U8_B, 0);
        assert_eq!(n, U8_B);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = U8_A;
        n.cmovnz(&U8_B, 0);
        assert_eq!(n, U8_A);

        for cond in 1..0xFF {
            let mut n = U8_A;
            n.cmovnz(&U8_B, cond);
            assert_eq!(n, U8_B);
        }
    }

    #[test]
    fn cmoveq_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu8 {
            cond.cmoveq(&cond, cond, &mut o);
            assert_eq!(o, cond);
            cond.cmoveq(&0, 0, &mut o);
            assert_eq!(o, cond);
        }

        U8_A.cmoveq(&U8_A, 43u8, &mut o);
        assert_eq!(o, 43u8);
        U8_A.cmoveq(&U8_B, 55u8, &mut o);
        assert_eq!(o, 43u8);
    }

    #[test]
    fn cmovne_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu8 {
            cond.cmovne(&0, cond, &mut o);
            assert_eq!(o, cond);
            cond.cmovne(&cond, 0, &mut o);
            assert_eq!(o, cond);
        }

        U8_A.cmovne(&U8_B, 55u8, &mut o);
        assert_eq!(o, 55u8);
        U8_A.cmovne(&U8_A, 12u8, &mut o);
        assert_eq!(o, 55u8);
    }
}

mod u16 {
    use cmov::{Cmov, CmovEq};

    pub const U16_A: u16 = 0x1111;
    pub const U16_B: u16 = 0x2222;

    #[test]
    fn cmovz_works() {
        let mut n = U16_A;

        for cond in 1..0xFF {
            n.cmovz(&U16_B, cond);
            assert_eq!(n, U16_A);
        }

        n.cmovz(&U16_B, 0);
        assert_eq!(n, U16_B);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = U16_A;
        n.cmovnz(&U16_B, 0);
        assert_eq!(n, U16_A);

        for cond in 1..0xFF {
            let mut n = U16_A;
            n.cmovnz(&U16_B, cond);
            assert_eq!(n, U16_B);
        }
    }

    #[test]
    fn cmoveq_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu16 {
            cond.cmoveq(&cond, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmoveq(&0, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U16_A.cmoveq(&U16_A, 43u8, &mut o);
        assert_eq!(o, 43u8);
        U16_A.cmoveq(&U16_B, 55u8, &mut o);
        assert_eq!(o, 43u8);
    }

    #[test]
    fn cmovne_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu16 {
            cond.cmovne(&0, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmovne(&cond, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U16_A.cmovne(&U16_B, 55u8, &mut o);
        assert_eq!(o, 55u8);
        U16_A.cmovne(&U16_A, 12u8, &mut o);
        assert_eq!(o, 55u8);
    }
}

mod u32 {
    use cmov::{Cmov, CmovEq};

    pub const U32_A: u32 = 0x1111_1111;
    pub const U32_B: u32 = 0x2222_2222;

    #[test]
    fn cmovz_works() {
        let mut n = U32_A;

        for cond in 1..0xFF {
            n.cmovz(&U32_B, cond);
            assert_eq!(n, U32_A);
        }

        n.cmovz(&U32_B, 0);
        assert_eq!(n, U32_B);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = U32_A;
        n.cmovnz(&U32_B, 0);
        assert_eq!(n, U32_A);

        for cond in 1..0xFF {
            let mut n = U32_A;
            n.cmovnz(&U32_B, cond);
            assert_eq!(n, U32_B);
        }
    }

    #[test]
    fn cmoveq_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu32 {
            cond.cmoveq(&cond, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmoveq(&0, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U32_A.cmoveq(&U32_A, 43u8, &mut o);
        assert_eq!(o, 43u8);
        U32_A.cmoveq(&U32_B, 55u8, &mut o);
        assert_eq!(o, 43u8);
    }

    #[test]
    fn cmovne_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu32 {
            cond.cmovne(&0, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmovne(&cond, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U32_A.cmovne(&U32_B, 55u8, &mut o);
        assert_eq!(o, 55u8);
        U32_A.cmovne(&U32_A, 12u8, &mut o);
        assert_eq!(o, 55u8);
    }
}

mod u64 {
    use cmov::{Cmov, CmovEq};

    pub const U64_A: u64 = 0x1111_1111_1111_1111;
    pub const U64_B: u64 = 0x2222_2222_2222_2222;

    #[test]
    fn cmovz_works() {
        let mut n = U64_A;

        for cond in 1..0xFF {
            n.cmovz(&U64_B, cond);
            assert_eq!(n, U64_A);
        }

        n.cmovz(&U64_B, 0);
        assert_eq!(n, U64_B);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = U64_A;
        n.cmovnz(&U64_B, 0);
        assert_eq!(n, U64_A);

        for cond in 1..0xFF {
            let mut n = U64_A;
            n.cmovnz(&U64_B, cond);
            assert_eq!(n, U64_B);
        }
    }

    #[test]
    fn cmoveq_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu64 {
            cond.cmoveq(&cond, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmoveq(&0, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U64_A.cmoveq(&U64_A, 43u8, &mut o);
        assert_eq!(o, 43u8);
        U64_A.cmoveq(&U64_B, 55u8, &mut o);
        assert_eq!(o, 43u8);
    }

    #[test]
    fn cmovne_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu64 {
            cond.cmovne(&0, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmovne(&cond, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U64_A.cmovne(&U64_B, 55u8, &mut o);
        assert_eq!(o, 55u8);
        U64_A.cmovne(&U64_A, 12u8, &mut o);
        assert_eq!(o, 55u8);
    }
}

mod u128 {
    use cmov::{Cmov, CmovEq};

    pub const U128_A: u128 = 0x1111_1111_1111_1111_2222_2222_2222_2222;
    pub const U128_B: u128 = 0x2222_2222_2222_2222_3333_3333_3333_3333;

    #[test]
    fn cmovz_works() {
        let mut n = U128_A;

        for cond in 1..0xFF {
            n.cmovz(&U128_B, cond);
            assert_eq!(n, U128_A);
        }

        n.cmovz(&U128_B, 0);
        assert_eq!(n, U128_B);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = U128_A;
        n.cmovnz(&U128_B, 0);
        assert_eq!(n, U128_A);

        for cond in 1..0xFF {
            let mut n = U128_A;
            n.cmovnz(&U128_B, cond);
            assert_eq!(n, U128_B);
        }
    }

    #[test]
    fn cmoveq_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu128 {
            cond.cmoveq(&cond, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmoveq(&0, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U128_A.cmoveq(&U128_A, 43u8, &mut o);
        assert_eq!(o, 43u8);
        U128_A.cmoveq(&U128_B, 55u8, &mut o);
        assert_eq!(o, 43u8);
    }

    #[test]
    fn cmovne_works() {
        let mut o = 0u8;

        for cond in 1..0xFFu128 {
            cond.cmovne(&0, cond as u8, &mut o);
            assert_eq!(o, cond as u8);
            cond.cmovne(&cond, 0, &mut o);
            assert_eq!(o, cond as u8);
        }

        U128_A.cmovne(&U128_B, 55u8, &mut o);
        assert_eq!(o, 55u8);
        U128_A.cmovne(&U128_A, 12u8, &mut o);
        assert_eq!(o, 55u8);
    }
}
