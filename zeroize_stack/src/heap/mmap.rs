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

use core::ptr;

use zeroize::ZeroizeOnDrop;

extern crate std;

/// A zeroizing heap-based stack. Feed one of these into the `switch_stacks` 
/// function.
pub struct ZeroizingHeapStack {
    mapping: *mut u8,
    size_with_guard: usize,
    page_size: usize,
}

impl ZeroizingHeapStack {
    /// Initializes a new "Zeroizing Heap Stack". To be fed into the `switch_stacks` 
    /// function, and it can be reused, but it must not be reused while it is in use.
    /// The borrow-checker should enforce this.
    pub fn new(stack_kb: usize) -> ZeroizingHeapStack {
        // For maximum portability we want to produce a stack that is aligned to a page and has
        // a size that’s a multiple of page size. It is natural to use mmap to allocate
        // these pages. Furthermore, we want to allocate two extras pages for the stack guard.
        // To achieve that we do our calculations in number of pages and convert to bytes last.
        let page_size = page_size();
        let requested_pages = stack_kb
            .checked_mul(1024)
            .expect("unreasonably large stack requested")
            .checked_add(page_size - 1)
            .expect("unreasonably large stack requested")
            / page_size;
        let page_count_with_guard = std::cmp::max(1, requested_pages) + 2;
        let size_with_guard = page_count_with_guard
            .checked_mul(page_size)
            .expect("unreasonably large stack requested");

        unsafe {
            let new_stack = libc::mmap(
                ptr::null_mut(),
                size_with_guard,
                libc::PROT_NONE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1, // Some implementations assert fd = -1 if MAP_ANON is specified
                0,
            );
            assert_ne!(
                new_stack,
                libc::MAP_FAILED,
                "mmap failed to allocate stack: {}",
                std::io::Error::last_os_error()
            );
            let guard = ZeroizingHeapStack {
                mapping: new_stack as *mut u8,
                page_size,
                size_with_guard,
            };
            // We leave two guard pages without read/write access in our allocation.
            // There is one guard page below the stack and another above it.
            let above_guard_page = new_stack.add(page_size);
            #[cfg(not(target_os = "openbsd"))]
            let result = libc::mprotect(
                above_guard_page,
                size_with_guard - 2 * page_size,
                libc::PROT_READ | libc::PROT_WRITE,
            );
            #[cfg(target_os = "openbsd")]
            let result = if libc::mmap(
                above_guard_page,
                size_with_guard - 2 * page_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_FIXED | libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_STACK,
                -1,
                0,
            ) == above_guard_page
            {
                0
            } else {
                -1
            };
            assert_ne!(
                result,
                -1,
                "mprotect/mmap failed: {}",
                std::io::Error::last_os_error()
            );
            guard
        }
    }

    // TODO this should return a *mut [u8], but pointer slices only got proper support with Rust 1.79.
    /// Returns (`start ptr of usable stack`, `size of usable stack`).
    pub fn stack_area(&self) -> (*mut u8, usize) {
        unsafe {
            (
                self.mapping.add(self.page_size),
                self.size_with_guard - 2 * self.page_size,
            )
        }
    }
}

impl Drop for ZeroizingHeapStack {
    fn drop(&mut self) {
        let (mut ptr, size) = self.stack_area();
        for _ in 0..size / size_of::<u128>() {
            unsafe {
                ptr::write_volatile(ptr, 0);
                ptr = ptr.add(1);
            }
        }
        unsafe {
            // FIXME: check the error code and decide what to do with it.
            // Perhaps a debug_assertion?
            libc::munmap(self.mapping as *mut std::ffi::c_void, self.size_with_guard);
        }
    }
}

impl ZeroizeOnDrop for ZeroizingHeapStack {}

fn page_size() -> usize {
    // FIXME: consider caching the page size.
    unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_size() {
        for kb in 1..64 {
            let stack = ZeroizingHeapStack::new(kb);
            assert_eq!(
                stack.stack_area().1, 
                (kb * 1024).div_ceil(page_size()) * page_size()
            );
        }
    }
}