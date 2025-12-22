#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![warn(missing_docs, unused_qualifications)]

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
//!   directly returning. `std` users do not need to worry about this due to
//!   the existence of `catch_unwind`.
//!
//! ## `nostd` Support
//!
//! This crate is compatible with `nostd` environments, but it is less safe
//! in the event that your stack-switched stack panics. Panicking on a separate
//! stack can cause undefined behavior (UB), but if it can be caught with
//! `std::panic::catch_unwind`, that aspect of the safety should be more safe.
//!
//! ## Use Cases
//!
//! - Cryptographic routines
//! - Secure enclave transitions
//! - Sanitizing temporary buffers in high-assurance systems

#[cfg(feature = "heap")]
mod heap;

use zeroize::{Zeroize, ZeroizeOnDrop};

extern crate alloc;

use alloc::{vec, vec::Vec};

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use core::any::Any;
#[cfg(feature = "std")]
use std::{boxed::Box, panic::catch_unwind};
#[cfg(feature = "std")]
type StackSwitchResult<T> = Result<T, Box<dyn Any + Send>>;
#[cfg(not(feature = "std"))]
type StackSwitchResult<T> = T;

use core::panic::{AssertUnwindSafe, UnwindSafe};

/// An aligned HeapStack. Aligned to the alignment of a `u128`, and aligned using
/// safe code instead of manual alloc calls. This implements `ZeroizeOnDrop` and
/// contains a lock flag to prevent the stack from being reused while it is being
/// used.
pub struct AlignedHeapStack {
    locked: bool,
    stack: Vec<u8>,
}

impl AlignedHeapStack {
    /// Creates a new `AlignedHeapStack`. `psm` recommends using at least `4 KB`
    /// of stack space.
    ///
    /// # Panics
    ///
    /// This function panics when `size_kb * 1024` overflows `isize`.
    pub fn new(size_kb: usize) -> Self {
        assert!(
            size_kb as isize * 1024 > 0,
            "size_kb must be positive and must not overflow isize when expanded to number of bytes instead of kb"
        );
        let result = Self {
            locked: false,
            stack: create_aligned_vec(size_kb, align_of::<u128>()),
        };
        // these may be redundant but I just want to be sure that the alignment doesn't
        // change somehow
        debug_assert_eq!(result.stack.as_ptr() as usize % align_of::<u128>(), 0);
        debug_assert_eq!(result.stack.len() % align_of::<u128>(), 0);
        result
    }

    fn is_locked(&self) -> bool {
        self.locked
    }

    fn set_lock(&mut self, locked: bool) {
        self.locked = locked;
    }
}

impl Drop for AlignedHeapStack {
    fn drop(&mut self) {
        self.stack.zeroize();
    }
}

impl ZeroizeOnDrop for AlignedHeapStack {}

psm::psm_stack_manipulation! {
    yes {
        /// Executes a function/closure and clears the function's stack by using
        /// preallocated space on the heap as the function's stack, and then zeroing
        /// that allocated space once the code has ran.
        ///
        /// This function does not clear the CPU registers.
        ///
        /// # Arguments
        ///
        /// * `aligned_heap_stack` - the heap-based aligned region of memory to
        ///   be used as the stack. `psm` recommends at least `4 KB` of stack
        ///   space, but the total size cannot overflow an `isize`. Also,
        ///   some architectures might consume more memory in the stack, such as
        ///   SPARC.
        /// * `crypto_fn` - the code to run while on the separate stack.
        ///
        /// ## Panicking
        ///
        /// This function panics when `psm` detects that `on_stack` is unavailable.
        ///
        /// ## Errors
        ///
        /// With the `std` feature enabled, this function will result in an error when
        /// the closure panics. You may want to log these errors securely, privately,
        /// as cryptography panics could be a little revealing if displayed to
        /// the end user.
        ///
        /// ## Debugging
        ///
        /// Using `#[inline(never)]` on the closure's function definition(s) could
        /// make it easier to debug as the function(s) should show up in backtraces.
        ///
        /// # Returns
        ///
        /// * If `std` is enabled, this returns a `Result<R, Box<dyn Any + Send>>`
        /// * Otherwise, this returns `R` directly; no panics are caught.
        ///
        /// # Safety
        ///
        /// * The stack needs to be large enough for `crypto_fn()` to execute without
        ///   overflow.
        /// * `nostd` only: `crypto_fn()` must not unwind or return control flow by any other means
        ///   than by directly returning.
        pub unsafe fn exec_on_sanitized_stack<F, R>(aligned_heap_stack: &mut AlignedHeapStack, crypto_fn: F) -> StackSwitchResult<R>
        where
            F: FnOnce() -> R + UnwindSafe,
        {
            assert!(!aligned_heap_stack.is_locked(), "AlignedHeapStack was locked. You must not use it while it is being used already!");
            aligned_heap_stack.set_lock(true);
            let res = unsafe {
                psm::on_stack(aligned_heap_stack.stack.as_mut_ptr(), aligned_heap_stack.stack.len(), || {
                    #[cfg(not(feature = "std"))]
                    {
                        crypto_fn()
                    }
                    #[cfg(feature = "std")]
                    {
                        catch_unwind(AssertUnwindSafe(crypto_fn))
                    }
                })
            };
            aligned_heap_stack.set_lock(false);
            res
        }
    }
    no {
        pub unsafe fn exec_on_sanitized_stack<F, R>(_stack_size_kb: isize, _crypto_fn: F) -> StackSwitchResult
        where
            F: FnOnce() -> R,
        {
            panic!("Stack manipulation not possible on this platform")
        }
    }
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
