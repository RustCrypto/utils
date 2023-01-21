mod u8 {
    use cmov::Cmov;

    #[test]
    fn cmovz_works() {
        let mut n = 0x11u8;

        for cond in 1..0xFF {
            n.cmovz(0x22, cond);
            assert_eq!(n, 0x11);
        }

        n.cmovz(0x22, 0);
        assert_eq!(n, 0x22);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 0x11u8;
        n.cmovnz(0x22, 0);
        assert_eq!(n, 0x11);

        for cond in 1..0xFF {
            let mut n = 0x11u8;
            n.cmovnz(0x22, cond);
            assert_eq!(n, 0x22);
        }
    }
}

mod u16 {
    use cmov::Cmov;

    #[test]
    fn cmovz_works() {
        let mut n = 0x1111u16;

        for cond in 1..0xFF {
            n.cmovz(0x2222, cond);
            assert_eq!(n, 0x1111);
        }

        n.cmovz(0x2222, 0);
        assert_eq!(n, 0x2222);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 0x1111u16;
        n.cmovnz(0x2222, 0);
        assert_eq!(n, 0x1111);

        for cond in 1..0xFF {
            let mut n = 0x1111u16;
            n.cmovnz(0x2222, cond);
            assert_eq!(n, 0x2222);
        }
    }
}

mod u32 {
    use cmov::Cmov;

    #[test]
    fn cmovz_works() {
        let mut n = 0x11111111u32;

        for cond in 1..0xFF {
            n.cmovz(0x22222222, cond);
            assert_eq!(n, 0x11111111);
        }

        n.cmovz(0x22222222, 0);
        assert_eq!(n, 0x22222222);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 0x11111111u32;
        n.cmovnz(0x22222222, 0);
        assert_eq!(n, 0x11111111);

        for cond in 1..0xFF {
            let mut n = 0x11111111u32;
            n.cmovnz(0x22222222, cond);
            assert_eq!(n, 0x22222222);
        }
    }
}

mod u64 {
    use cmov::Cmov;

    #[test]
    fn cmovz_works() {
        let mut n = 0x11111111_11111111u64;

        for cond in 1..0xFF {
            n.cmovz(0x22222222_22222222, cond);
            assert_eq!(n, 0x11111111_11111111);
        }

        n.cmovz(0x22222222_22222222, 0);
        assert_eq!(n, 0x22222222_22222222);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 0x11111111_11111111u64;
        n.cmovnz(0x22222222_22222222, 0);
        assert_eq!(n, 0x11111111_11111111);

        for cond in 1..0xFF {
            let mut n = 0x11111111_11111111u64;
            n.cmovnz(0x22222222_22222222, cond);
            assert_eq!(n, 0x22222222_22222222);
        }
    }
}

mod u128 {
    use cmov::Cmov;

    #[test]
    fn cmovz_works() {
        let mut n = 0x11111111_11111111_22222222_22222222u128;

        for cond in 1..0xFF {
            n.cmovz(0x22222222_22222222_33333333_33333333, cond);
            assert_eq!(n, 0x11111111_11111111_22222222_22222222);
        }

        n.cmovz(0x22222222_22222222_33333333_33333333, 0);
        assert_eq!(n, 0x22222222_22222222_33333333_33333333);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 0x11111111_11111111_22222222_22222222u128;
        n.cmovnz(0x22222222_22222222_33333333_33333333, 0);
        assert_eq!(n, 0x11111111_11111111_22222222_22222222);

        for cond in 1..0xFF {
            let mut n = 0x11111111_11111111u128;
            n.cmovnz(0x22222222_22222222_33333333_33333333, cond);
            assert_eq!(n, 0x22222222_22222222_33333333_33333333);
        }
    }
}
