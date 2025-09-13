//! Stack sanitization integration tests

mod stack_sanitization_tests {
    use stack_sanitizer::exec_on_sanitized_stack;

    fn dummy_fn() -> (*const u8, u64) {
        let temporary_data = 42;
        let ptr = temporary_data as *const u8;
        (ptr, 12345)
    }

    #[test]
    fn stack_sanitization_v2() {
        let result = unsafe { exec_on_sanitized_stack(4, || dummy_fn())};
        assert_eq!(result.1, 12345);
        // results in segmentation fault
        // assert_eq!(unsafe {*result.0}, 42);
    }
}