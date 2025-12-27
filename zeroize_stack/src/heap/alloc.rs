//! This file contains code derived from the Rust project,
//! originally written by Alex Crichton and licensed under
//! the Apache License, Version 2.0 or the MIT license, at
//! your option.
//!
//! Copyright (c) 2014 Alex Crichton
//!
//! Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
//! http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
//! <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

use core::{ptr, sync::atomic};

extern crate std;

/// A zeroizing heap-based stack. Feed one of these into the `switch_stacks` 
/// function.
pub struct ZeroizingHeapStack {
    new_stack: *mut u8,
    stack_bytes: usize,
}

const ALIGNMENT: usize = 32;

impl ZeroizingHeapStack {
    /// Initializes a new "Zeroizing Heap Stack". To be fed into the `switch_stacks` 
    /// function, and it can be reused, but it must not be reused while it is in use.
    /// The borrow-checker should enforce this.
    pub fn new(stack_kb: usize) -> ZeroizingHeapStack {
        let stack_bytes = stack_kb * 1024;
        assert!(
            stack_bytes as isize > 0,
            "stack_kb must be positive and must not overflow isize when expanded to number of bytes instead of KB"
        );
        // On these platforms we do not use stack guards. this is very unfortunate,
        // but there is not much we can do about it without OS support.
        // We simply allocate the requested size from the global allocator with a suitable
        // alignment.
        let stack_bytes = stack_bytes
            .checked_add(ALIGNMENT - 1)
            .expect("unreasonably large stack requested")
            / ALIGNMENT
            * ALIGNMENT;
        let layout = std::alloc::Layout::from_size_align(stack_bytes, ALIGNMENT).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        assert!(!ptr.is_null(), "unable to allocate stack");
        ZeroizingHeapStack {
            new_stack: ptr,
            stack_bytes,
        }
    }
    /// Returns (`start ptr of usable stack`, `size of usable stack`).
    pub fn stack_area(&self) -> (*mut u8, usize) {
        (self.new_stack, self.stack_bytes)
    }
}

impl Drop for ZeroizingHeapStack {
    fn drop(&mut self) {
        let mut ptr = self.new_stack as *mut u128;
        for _ in 0..self.stack_bytes / size_of::<u128>() {
            unsafe {
                ptr::write_volatile(ptr, 0);
                ptr = ptr.add(1);
            }
        }
        atomic::compiler_fence(atomic::Ordering::SeqCst);
        unsafe {
            std::alloc::dealloc(
                self.new_stack,
                std::alloc::Layout::from_size_align_unchecked(self.stack_bytes, ALIGNMENT),
            );
        }
    }
}