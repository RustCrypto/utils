//! Stack sanitization integration tests

mod stack_sanitization_tests {
    use std::panic::AssertUnwindSafe;

    use zeroize_stack::{AlignedHeapStack, exec_on_sanitized_stack};

    #[inline(never)]
    fn dummy_fn() -> (*const u8, u64) {
        let temporary_data = 42;
        let ptr = temporary_data as *const u8;
        (ptr, 12345)
    }

    #[test]
    fn stack_sanitization_v2() {
        let mut heap_stack = AlignedHeapStack::new(4);
        let result = unsafe { exec_on_sanitized_stack(&mut heap_stack, || dummy_fn()) };
        assert_eq!(result.unwrap().1, 12345);
        // results in segmentation fault, which is somewhat normal... just wanted
        // to try it
        // assert_eq!(unsafe {*result.0}, 42);
    }

    #[test]
    fn allow_stack_reuse_between_calls() {
        let mut heap_stack = AlignedHeapStack::new(4);
        let result_1 = unsafe { exec_on_sanitized_stack(&mut heap_stack, || dummy_fn()) };
        assert!(result_1.is_ok());
        assert_eq!(result_1.unwrap().1, 12345);
        let result_2 = unsafe { exec_on_sanitized_stack(&mut heap_stack, || dummy_fn()) };
        assert!(result_2.is_ok());
        assert_eq!(result_2.unwrap().1, 12345);
    }

    fn non_returning_function(v: &mut u32) {
        *v += 5;
    }
    #[test]
    fn non_returning_function_test() {
        let mut heap_stack = AlignedHeapStack::new(4);
        let mut v = 0;
        unsafe {
            exec_on_sanitized_stack(
                &mut heap_stack,
                AssertUnwindSafe(|| non_returning_function(&mut v)),
            )
        }
        .unwrap();
        assert_eq!(v, 5);
    }
}
