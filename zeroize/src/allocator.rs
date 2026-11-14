use crate::Zeroize;

use core::{
    alloc::{AllocError, Allocator, Layout},
    mem::MaybeUninit,
    ptr::NonNull,
    slice,
};

/// An allocator that zeroizes memory before freeing it.
#[repr(transparent)]
#[derive(Debug)]
pub struct ZeroizingAllocator<A: Allocator> {
    inner: A,
}
impl<A: Allocator> ZeroizingAllocator<A> {
    /// Constructs a [ZeroizingAllocator] around a pre-existing allocator.
    pub fn new(allocator: A) -> Self {
        Self { inner: allocator }
    }
}

/*
 * SAFETY: Most of the safety invariants are expected to be upheld by the underlying allocator.
 * For the rest, see the safety comments inside this impl.
 */
unsafe impl<A: Allocator> Allocator for ZeroizingAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.inner.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // Use scoping to ensure that the slice is dead when we call the underlying allocator.
        {
            /*
             * SAFETY:
             * - The memory is being deallocated -> we are not bound to avoid touching it
             * - The memory is being deallocated -> no references to it still exist
             * - ptr is valid and not null.
             * - MaybeUninit<u8> has the same size, alignment, and ABI as u8.
             * - The layout gives us the size of the memory allocation, so the zeroize only
             *   writes to memory belonging to this allocation.
             * Thus, we can create a &mut [MaybeUninit<u8>] and zeroize it without causing UB.
             *
             * We need to use MaybeUninit<u8> instead of u8 to handle uninitialized padding bytes.
             */
            let allocated_slice: &mut [MaybeUninit<u8>] =
                unsafe { slice::from_raw_parts_mut(ptr.as_ptr().cast(), layout.size()) };
            /*
             * SAFETY: the only panic inside [Z]::zeroize() is an assert that the slice length is
             * smaller than isize::MAX, which is a precondition of constructing a Layout value.
             * (The checked_mul(size_of::<u8>) is a multiplication by 1, so it cannot fail.)
             * Therefore, this zeroize call cannot panic.
             */
            allocated_slice.zeroize();
        }
        // Now do the actual deallocation
        // SAFETY: we assume inner's deallocate is sound
        unsafe { self.inner.deallocate(ptr, layout) }
    }
}
