//! Integration tests for `zeroize_derive` proc macros

#[cfg(feature = "zeroize_derive")]
mod custom_derive_tests {
    use zeroize::Zeroize;

    #[test]
    fn derive_tuple_struct_test() {
        #[derive(Zeroize)]
        #[zeroize(drop)]
        struct Z([u8; 3]);

        let mut value = Z([1, 2, 3]);
        value.zeroize();
        assert_eq!(&value.0, &[0, 0, 0])
    }

    #[test]
    fn derive_struct_test() {
        #[derive(Zeroize)]
        #[zeroize(drop)]
        struct Z {
            string: String,
            vec: Vec<u8>,
            bytearray: [u8; 3],
            number: usize,
            boolean: bool,
        }

        let mut value = Z {
            string: String::from("Hello, world!"),
            vec: vec![1, 2, 3],
            bytearray: [4, 5, 6],
            number: 42,
            boolean: true,
        };

        value.zeroize();

        assert!(value.string.is_empty());
        assert!(value.vec.is_empty());
        assert_eq!(&value.bytearray, &[0, 0, 0]);
        assert_eq!(value.number, 0);
        assert!(!value.boolean);
    }

    #[test]
    fn derive_enum_test() {
        #[derive(Zeroize)]
        #[zeroize(drop)]
        enum Z {
            #[allow(dead_code)]
            Variant1,
            Variant2(usize),
        }

        let mut value = Z::Variant2(26);

        value.zeroize();

        assert!(matches!(value, Z::Variant2(0)));
    }

    /// Test that the custom macro actually derived `Drop` for `Z`
    #[test]
    fn derive_struct_drop() {
        #[derive(Zeroize)]
        #[zeroize(drop)]
        struct Z([u8; 3]);

        assert!(std::mem::needs_drop::<Z>());
    }

    /// Test that the custom macro actually derived `Drop` for `Z`
    #[test]
    fn derive_enum_drop() {
        #[allow(dead_code)]
        #[derive(Zeroize)]
        #[zeroize(drop)]
        enum Z {
            Variant1,
            Variant2(usize),
        }

        assert!(std::mem::needs_drop::<Z>());
    }

    /// Test that `Drop` is not derived in the following case by defining a
    /// `Drop` impl which should conflict if the custom derive defined one too
    #[allow(dead_code)]
    #[derive(Zeroize)]
    struct ZeroizeNoDropStruct([u8; 3]);

    impl Drop for ZeroizeNoDropStruct {
        fn drop(&mut self) {}
    }

    #[allow(dead_code)]
    #[derive(Zeroize)]
    enum ZeroizeNoDropEnum {
        Variant([u8; 3]),
    }

    impl Drop for ZeroizeNoDropEnum {
        fn drop(&mut self) {}
    }
}
