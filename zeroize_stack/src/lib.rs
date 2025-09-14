//! # zeroize_stack
//!
//! A crate for sanitizing stack memory after sensitive operations—sometimes referred to as _Stack Bleaching_.
//!
//! Modern compilers and CPUs routinely copy, spill, and rearrange data during execution. Even if sensitive values are scoped to a function, they may:
//! - Be duplicated across multiple stack frames
//! - Be spilled from registers to the stack during register pressure
//! - Persist in memory long after the function returns
//!
//! This crate provides tools to explicitly zeroize stack regions used during
//! cryptographic or sensitive computations, helping mitigate:
//! - Leakage through stack inspection or memory dumps
//! - Residual data from compiler-inserted spills
//! - ABI-visible register reuse across function boundaries
//!
//! ## Why Stack Sanitization Matters
//!
//! Unlike heap memory, stack allocations are ephemeral and compiler-controlled.
//! Sensitive data may be:
//! - Copied implicitly by the optimizer
//! - Stored temporarily during register allocation
//! - Left behind in stack frames even after function return
//!
//! This crate offers abstractions for:
//! - Executing functions on isolated, aligned stack buffers
//! - Zeroizing stack memory after execution
//!
//! ## Safety
//!
//! These operations involve low-level stack manipulation and unsafe code. The
//! caller must ensure:
//! - The stack size provided is large enough for the closure to run with.
//! - The closure does not unwind or return control flow by any means other than
//! directly returning.
//!
//! ## Use Cases
//!
//! - Cryptographic routines
//! - Secure enclave transitions
//! - Sanitizing temporary buffers in high-assurance systems

use psm::on_stack;

use zeroize::Zeroize;

extern crate alloc;

use alloc::{vec, vec::Vec};

/// Executes a function/closure and clears the function's stack by using
/// preallocated space on the heap as the function's stack, and then zeroing
/// that allocated space once the code has ran.
///
/// This function does not clear the CPU registers.
///
/// # Arguments
///
/// * `stack_size_kb` - how large the stack will be. `psm` recommends at least
/// `4 KB` of stack size, but the total size cannot overflow an `isize`. Also,
/// some architectures might consume more memory in the stack, such as SPARC.
/// * `crypto_fn` - the code to run while on the separate stack.
///
/// # Safety
///
/// * `crypto_fn` should be marked as `#[inline(never)]`, preventing register
/// reuse and stack layout changes.
/// * The stack needs to be large enough for `crypto_fn()` to execute without
/// overflow.
/// * `crypto_fn()` must not unwind or return control flow by any other means
/// than by directly returning.
pub unsafe fn exec_on_sanitized_stack<F, R>(stack_size_kb: isize, crypto_fn: F) -> R
where
    F: FnOnce() -> R,
{
    assert!(
        stack_size_kb * 1024 > 0,
        "Stack size must be greater than 0 kb and `* 1024` must not overflow `isize`"
    );
    let mut stack = create_aligned_vec(stack_size_kb as usize, core::mem::align_of::<u128>());
    let res = unsafe {
        on_stack(stack.as_mut_ptr(), stack.len(), || {
            let res = crypto_fn();
            res
        })
    };
    stack.zeroize();
    res
}

/// Round up to the nearest multiple of alignment
const fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

/// Creates an aligned Vec<u8> with the specified size in KB and alignment.
///
/// This helps ensure that the safety requirements are met when using
/// `fn secure_crypto_call_heap()`.
///
/// Both the data pointer and length will be aligned to the specified boundary.
fn create_aligned_vec(size_kb: usize, alignment: usize) -> Vec<u8> {
    let size_bytes = size_kb * 1024;
    // checking one of the safety conditions of `psm::on_stack()`
    assert!(size_bytes <= isize::MAX as usize);

    let aligned_size = align_up(size_bytes, alignment);

    // Allocate extra space to ensure we can find an aligned region
    let mut vec = vec![0u8; aligned_size + alignment];

    // Find the aligned position within the vec
    let ptr_addr = vec.as_ptr() as usize;
    let aligned_addr = align_up(ptr_addr, alignment);
    let offset = aligned_addr - ptr_addr;

    // Remove elements from the beginning to align the start
    vec.drain(0..offset);

    // Truncate to the exact aligned size we want
    vec.truncate(aligned_size);

    // Verify alignment (these will be optimized out in release builds)
    debug_assert_eq!(vec.as_ptr() as usize % alignment, 0);
    debug_assert_eq!(vec.len() % alignment, 0);
    debug_assert_eq!(vec.len(), aligned_size);

    vec
}
