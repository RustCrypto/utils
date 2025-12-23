//! Heap-based stack zeroization module. This module uses Rust-Lang's `psm` 
//! crate to switch stacks to a stack that is allocated on the heap 
//! (`ZeroizingHeapStack`) and then executes a callback function on that 
//! stack. You can reuse this stack as many times as you want, and when it is 
//! dropped, it will be zeroized.

use core::panic::UnwindSafe;

use psm::psm_stack_manipulation;

#[cfg(feature = "std")]
extern crate std;

psm_stack_manipulation! {
    yes {
        #[cfg(any(target_family = "wasm", target_os = "hermit"))]
        #[path = "alloc.rs"]
        mod heap_struct;

        #[cfg(not(any(target_family = "wasm", target_os = "hermit")))]
        #[path = "mmap.rs"]
        mod heap_struct;
        
        pub use heap_struct::ZeroizingHeapStack;

        /// Executes a closure on a provided zeroizing heap-based stack.
        /// 
        /// This function does not clear CPU registers.
        /// 
        /// # Arguments
        /// 
        /// * `zeroizing_heap_stack` - the heap-based stack you plan on using 
        /// for running the closure. `psm` recommends at least `4 KiB` of stack space, 
        /// but the total size cannot overflow an `isize`. Also, some architectures 
        /// might consume more memory in the stack, such as SPARC.
        /// 
        /// * `crypto_fn` - the code to run while on the switched stack.
        /// 
        /// ## Panicking
        /// 
        /// This function does not panic, but it can segfault.
        /// 
        /// ## Segfaults
        /// 
        /// This code will cause a segmentation fault if your closure consumes 
        /// more stack space than what you have allocated.
        /// 
        /// ## Debugging
        /// 
        /// Using `#[inline(never)]` on the closure's function definition(s) could 
        /// make it easier to debug as the function(s) should then show up in 
        /// backtraces.
        /// 
        /// # Returns
        /// 
        /// This function returns the returned value from the closure.
        /// 
        /// # Safety
        /// 
        /// * The stack needs to be large enough for `crypto_fn()` to execute 
        /// without overflowing.
        /// 
        /// * For `nostd`, you should use `panic = 'abort'` to avoid unwinding 
        /// on the switched stack. Unwinding across stack boundaries could cause 
        /// undefined behavior. `nostd` code must not unwind or return control 
        /// flow by any other means.
        pub unsafe fn switch_stacks<F, R>(zeroizing_heap_stack: &mut ZeroizingHeapStack, crypto_fn: F) -> R
        where 
            F: FnOnce() -> R + UnwindSafe,
        {
            let (stack_ptr, size) = zeroizing_heap_stack.stack_area();
            unsafe {
                let panic = psm::on_stack(stack_ptr, size, move || {
                    #[cfg(feature = "std")]
                    {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(crypto_fn))
                    }
                    #[cfg(not(feature = "std"))]
                    return crypto_fn()
                });
                match panic {
                    Err(p) => std::panic::resume_unwind(p),
                    Ok(v) => v
                }
            }
        }
    }

    no {
        pub struct ZeroizingHeapStack;
        compiler_warning(f)
        impl ZeroizingHeapStack {
            pub fn new(stack_kb: usize) -> Self {
                let _ = stack_kb;
                ZeroizingHeapStack
            }
        }
        /// PSM is unavailable on this arch/target.
        #[deprecated(note = "PSM is unavailable on this arch/target. Crypto closures will not run on a zeroizing stack.")]
        pub unsafe fn switch_stacks<F, R>(zeroizing_heap_stack: &mut ZeroizingHeapStack, crypto_fn: F) -> R
        where 
            F: FnOnce() -> R + UnwindSafe,
        {
            let _ = zeroizing_heap_stack;
            crypto_fn()
        }
    }
}