//! Tests for uses of ZeroizeAllocator.

#![cfg(feature = "allocator")]

use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
    slice,
    sync::atomic::{AtomicBool, Ordering},
};
use std::{alloc::System, sync::Mutex};

use zeroize::ZeroizingAllocator;

/// An Allocator that delays deallocation until it is dropped.
#[derive(Debug)]
struct DelayedDeallocator<A: Allocator> {
    allocator: A,
    // Use the System allocator for the Vec to avoid recursions.
    dealloc_queue: Mutex<Vec<(NonNull<u8>, Layout), System>>,
}
impl<A: Allocator> DelayedDeallocator<A> {
    pub fn new(allocator: A) -> Self {
        Self {
            allocator,
            dealloc_queue: Mutex::new(Vec::new_in(System)),
        }
    }
}
impl<A: Allocator> Drop for DelayedDeallocator<A> {
    fn drop(&mut self) {
        /*
         * Clear the poison so that the unwrap always succeeds, and free what we've collected.
         * Panics elsewhere would not have corrupted any existing items, so at most we leak some
         * memory, which is allowed.
         */
        self.dealloc_queue.clear_poison();
        let queue_inner = self.dealloc_queue.get_mut().unwrap();
        for (ptr, layout) in queue_inner.iter() {
            /*
             * SAFETY: values on the vec were pushed on by our deallocate, and we expect those
             * callers to have satisfied the deallocate preconditions.
             */
            unsafe {
                self.allocator.deallocate(*ptr, *layout);
            }
        }
    }
}
unsafe impl<A: Allocator> Allocator for DelayedDeallocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocator.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // CLear the poison so that the unwrap always succeeds
        self.dealloc_queue.clear_poison();
        // This would only panic upon getting close to isize::MAX, which we don't hit in our tests
        // TODO: actually prevent unwinding by catching unwinds and aborting if necessary
        self.dealloc_queue.lock().unwrap().push((ptr, layout));
    }
}

/// An allocator that tracks whether a non-zeroed byte is present at deallocation time.
#[derive(Debug)]
struct VerifyZeroizedAllocator<A: Allocator> {
    inner: ZeroizingAllocator<DelayedDeallocator<A>>,
    has_dealloced: AtomicBool,
    non_zero_in_dealloc: AtomicBool,
}

impl<A: Allocator> VerifyZeroizedAllocator<A> {
    pub fn new(allocator: A) -> Self {
        Self {
            inner: ZeroizingAllocator::new(DelayedDeallocator::new(allocator)),
            has_dealloced: AtomicBool::new(false),
            non_zero_in_dealloc: AtomicBool::new(false),
        }
    }
    pub fn has_dealloced(&self) -> bool {
        // Load from flag synchronizes-with the deallocate method
        self.has_dealloced.load(Ordering::Acquire)
    }
    pub fn non_zero_dealloced(&self) -> bool {
        // Load from flag synchronizes-with the deallocate method
        self.non_zero_in_dealloc.load(Ordering::Acquire)
    }
}

unsafe impl<A: Allocator> Allocator for VerifyZeroizedAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.inner.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: we assume the nested Allocator's deallocate is sound
        unsafe { self.inner.deallocate(ptr, layout) };
        /*
         * Inspecting the memory after self.inner.deallocate is only safe because DelayedDeallocator
         * means that the memory hasn't actually been freed yet.
         */
        {
            /*
             * SAFETY:
             * - The memory is being deallocated -> we are not bound to avoid touching it
             * - The memory is being deallocated -> no references to it still exist
             * - ptr is valid and not null.
             * - The layout gives us the size of the memory allocation, so the zeroize only
             *   writes to memory belonging to this allocation.
             * Thus, we can create a &mut [u8] and zeroize it without causing UB.
             */
            let allocated_slice: &[u8] =
                unsafe { slice::from_raw_parts(ptr.as_ptr(), layout.size()) };
            for byte in allocated_slice {
                if *byte != 0 {
                    // Store to flag synchronizes-with the non_zero_dealloced method
                    self.non_zero_in_dealloc.store(true, Ordering::Release);
                }
            }
        }
        // Store to flag synchronizes-with the has_dealloced method
        self.has_dealloced.store(true, Ordering::Release);
    }
}

#[test]
fn zeroized_alloc_box() {
    let alloc_helper = VerifyZeroizedAllocator::new(System);
    let mut box_example = Box::new_in(
        *b"But you'll look sweet upon the seat\nOf a bicycle built for two!",
        &alloc_helper,
    );
    box_example = core::hint::black_box(box_example);
    drop(box_example);
    // Assert that a deallocation actually occurred...
    assert!(alloc_helper.has_dealloced());
    // ...and that the deallocation did not leave non-zero bytes hanging.
    assert!(!alloc_helper.non_zero_dealloced());
}

#[test]
fn zeroized_alloc_box_pads() {
    // (u16, u8) has a padding byte-verify that we handle it correctly
    let alloc_helper = VerifyZeroizedAllocator::new(System);
    let mut box_example = Box::new_in([(16u16, 8u8), (1u16, 0u8)], &alloc_helper);
    box_example = core::hint::black_box(box_example);
    drop(box_example);
    // Assert that a deallocation actually occurred...
    assert!(alloc_helper.has_dealloced());
    // ...and that the deallocation did not leave non-zero bytes hanging.
    assert!(!alloc_helper.non_zero_dealloced());
}

#[test]
fn zeroized_alloc_vec() {
    let alloc_helper = VerifyZeroizedAllocator::new(System);
    let mut vec_example = Vec::new_in(&alloc_helper);
    vec_example.extend([2, 3, 5, 7, 11, 13, 17, 19]);
    vec_example = core::hint::black_box(vec_example);
    drop(vec_example);
    // Assert that a deallocation actually occurred...
    assert!(alloc_helper.has_dealloced());
    // ...and that the deallocation did not leave non-zero bytes hanging.
    assert!(!alloc_helper.non_zero_dealloced());
}
