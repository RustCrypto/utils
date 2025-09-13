//! Stack sanitization integration tests

#[cfg(all(feature = "stack_sanitization", feature = "alloc"))]
mod stack_sanitization_tests {
    use zeroize::{create_aligned_vec, secure_crypto_call_heap};

    fn dummy_fn() -> (*const u8, u64) {
        let temporary_data = 42;
        let ptr = temporary_data as *const u8;
        (ptr, 12345)
    }

    #[test]
    fn stack_sanitization_v2() {
        let mut stack = create_aligned_vec(4, 16);
        let result = unsafe { secure_crypto_call_heap(|| {dummy_fn()}, &mut stack)};
        assert_eq!(result.1, 12345);
        // results in segmentation fault
        // assert_eq!(unsafe {*result.0}, 42);
    }
}