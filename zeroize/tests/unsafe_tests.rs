#![cfg(not(miri))]
use core::ptr;
use std::alloc::{GlobalAlloc, Layout, System};

use zeroize::Zeroize;

// Allocator that leaks all memory it allocates, thus leaving the memory open for inspection.
struct UnfreeAllocator;
unsafe impl GlobalAlloc for UnfreeAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Do nothing, leak memory
        eprintln!("Leaking memory: {:?}", ptr);
        let _ = (ptr, layout);
    }
}

#[global_allocator]
static UNFREE_ALLOCATOR: UnfreeAllocator = UnfreeAllocator;

#[test]
#[allow(unsafe_code, unused_assignments)]
fn clears_memory_when_scope_ends() {
    struct SecretBox<S: Zeroize + ?Sized>(Box<S>);
    impl<S: Zeroize + ?Sized> Drop for SecretBox<S> {
        fn drop(&mut self) {
            self.0.as_mut().zeroize()
        }
    }

    let mut ptr: *const u128 = ptr::null();

    unsafe {
        {
            let secret = SecretBox(Box::new(0xdeadbeef_u128));
            let boxptr = &secret as *const SecretBox<u128>;
            let boxptr = boxptr as *const *const u128;
            ptr = *boxptr;
            assert!(!ptr.is_null(), "ptr is null before drop, not ok");
            let bytes: &[u8] = core::slice::from_raw_parts(ptr as *const u8, size_of::<u128>());
            assert!(
                !bytes.iter().all(|&b| b == 0),
                "Expected non-zero data, instead found 0s: {:X?}",
                bytes
            );
        }
        // Check that the memory is cleared after the scope ends
        for _ in 0..size_of::<u128>() {
            // This is UB but proooobably fine given the leaking allocator.
            let byte = *(ptr as *const u8).add(1);
            assert_eq!(byte, 0);
        }
    }
}
