//! Stack sanitization integration tests

mod stack_sanitization_tests {
    use zeroize_stack::exec_on_sanitized_stack;

    #[inline(never)]
    fn dummy_fn() -> (*const u8, u64) {
        let temporary_data = 42;
        let ptr = temporary_data as *const u8;
        (ptr, 12345)
    }

    #[test]
    fn stack_sanitization_v2() {
        let result = unsafe { exec_on_sanitized_stack(4, || dummy_fn()) };
        assert_eq!(result.unwrap().1, 12345);
        // results in segmentation fault, which is somewhat normal... just wanted
        // to try it
        // assert_eq!(unsafe {*result.0}, 42);
    }
}
