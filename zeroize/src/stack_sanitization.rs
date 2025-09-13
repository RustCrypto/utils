//! Module for sanitizing the stack, sometimes referred to as "Stack Bleaching."

use core::arch::asm;
use core::ptr;

use crate::Zeroize;

#[cfg(feature = "alloc")]
use alloc::{
    vec,
    vec::{Vec}
};

/// Gets the current stack pointer
#[inline(never)]
fn get_stack_pointer() -> *mut u8 {
    let sp: *mut u8;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("mov {}, rsp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!("mov {}, esp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}

/// Clears stack memory between two stack pointers using volatile writes
#[inline(never)]
unsafe fn clear_stack_range(start_sp: *mut u8, end_sp: *mut u8) {
    let start = start_sp.min(end_sp) as usize;
    let end = start_sp.max(end_sp) as usize;
    let size = end - start;
    
    if size == 0 || size > 1024 * 1024 {  // Sanity check
        return;
    }
    
    // Clear using volatile writes to prevent optimization
    let mut ptr = start as *mut u64;
    // Align to 8-byte boundary to clear 8 bytes at a time
    let end_ptr = (end & !7) as *mut u64;
    
    while ptr < end_ptr {
        unsafe {
            ptr::write_volatile(ptr, 0);
            ptr = ptr.add(1);
        }
    }
    
    // Clear remaining bytes
    let mut byte_ptr = ptr as *mut u8;
    let byte_end = end as *mut u8;
    while byte_ptr < byte_end {
        unsafe {
            ptr::write_volatile(byte_ptr, 0);
            byte_ptr = byte_ptr.add(1);
        }
    }
}

/// Wrapper function that captures stack state and clears after crypto operation
/// 
/// # Safety
/// 
/// * `crypto_fn` should be marked as `#[inline(never)]`, preventing register 
/// reuse or stack layout changes
#[inline(never)]
pub unsafe fn secure_crypto_call<F, R>(crypto_fn: F) -> R
where
    F: FnOnce() -> R,
{
    // Get initial stack pointer
    let initial_sp = get_stack_pointer();
    assert!(!initial_sp.is_null());
    
    // Call the crypto function (this will use more stack)
    let result = crypto_fn();
    
    // Get stack pointer after crypto operation
    let final_sp = psm::stack_pointer();
    assert!(!final_sp.is_null());
    debug_assert_ne!(initial_sp, final_sp);
    
    // Clear the stack range used by the crypto function
    unsafe {
        clear_stack_range(initial_sp, final_sp);
    }
    
    result
}

/// Wrapper function that captures stack state and clears after crypto operation
/// by using an allocation on the heap as the stack.
/// 
/// If you wish to clear the registers, it is recommended to clear them from 
/// within `crypto_fn()`. This function does not clear them for you.
/// 
/// # Safety
/// 
/// * `crypto_fn` should be marked as `#[inline(never)]`, preventing register 
/// reuse and stack layout changes.
/// * The stack start address needs to be aligned for the target architecture, which is 
/// typically 16 bytes for x86_64.
/// * The stack size needs to be a multiple of stack alignment required by 
/// the target.
/// * The stack size must not overflow `isize`.
/// * The stack needs to be large enough for `crypto_fn()` to execute without 
/// overflow.
/// * `crypto_fn()` must not unwind or return control flow by any other means 
/// than by directly returning.
pub unsafe fn secure_crypto_call_heap<F, R>(crypto_fn: F, stack: &mut [u8] ) -> R
where 
    F: FnOnce() -> R,
{
    let res = unsafe {
        psm::on_stack(stack.as_mut_ptr(), stack.len(), || {
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
#[cfg(feature = "alloc")]
pub fn create_aligned_vec(size_kb: usize, alignment: usize) -> Vec<u8> {
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