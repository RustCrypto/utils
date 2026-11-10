//! Tests for `Zeroize` impls on heap-allocated data structures

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering::Relaxed},
};
use zeroize::Zeroize;

use std::{
    alloc::System,
    sync::{Mutex, MutexGuard, PoisonError},
};

static REG_PTR: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut());

/// Set by the allocator if the registered allocation was freed while not fully zeroized.
///
/// The allocator can't assert itself, since `GlobalAlloc` methods must not unwind.
static NOT_ZEROIZED: AtomicBool = AtomicBool::new(false);

/// `REG_PTR` is global, so tests which use it must not run concurrently.
static REG_LOCK: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    REG_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Assert the registered allocation has been freed, and that it was zeroized when it was.
fn assert_checked() {
    // Reset the global state before asserting, so a failure here can't leave `REG_PTR`
    // pointing to memory which may be freed and reused by an unrelated allocation.
    let freed = REG_PTR.swap(ptr::null_mut(), Relaxed).is_null();
    let not_zeroized = NOT_ZEROIZED.swap(false, Relaxed);

    assert!(freed, "registered allocation was never freed");
    assert!(!not_zeroized, "registered allocation was not zeroized");
}

// Allocator that ensures that allocation registered in `REG_PTR` is zeroized.
struct ProxyAllocator;

#[allow(clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for ProxyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr == REG_PTR.load(Relaxed) {
            let bytes = unsafe { core::slice::from_raw_parts(ptr, layout.size()) };
            if bytes.iter().any(|&b| b != 0) {
                NOT_ZEROIZED.store(true, Relaxed);
            }
            REG_PTR.store(ptr::null_mut(), Relaxed);
        }

        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static PROXY_ALLOCATOR: ProxyAllocator = ProxyAllocator;

struct SecretBox<S: Zeroize>(Box<S>);

impl<S: Zeroize> SecretBox<S> {
    fn new(val: S) -> Self {
        let mut b = Box::new(val);
        // TODO(MSRV-1.98): use `Box::as_mut_ptr`
        let p: *mut S = &raw mut *b;
        REG_PTR.store(p.cast(), Relaxed);
        Self(b)
    }
}

impl<S: Zeroize> Drop for SecretBox<S> {
    fn drop(&mut self) {
        self.0.as_mut().zeroize();
    }
}

struct ObserveSecretBox<S: Default>(Box<S>);

impl<S: Default> ObserveSecretBox<S> {
    fn new(val: S) -> Self {
        let mut b = Box::new(val);
        // TODO(MSRV-1.98): use `Box::as_mut_ptr`
        let p: *mut S = &raw mut *b;
        REG_PTR.store(p.cast(), Relaxed);
        Self(b)
    }
}

impl<S: Default> Drop for ObserveSecretBox<S> {
    fn drop(&mut self) {
        *self.0 = Default::default();
        zeroize::optimization_barrier(&self);
    }
}

#[test]
fn proxy_alloc_test() {
    let _guard = lock();

    let b1 = SecretBox::new([u128::MAX; 10]);
    core::hint::black_box(&b1);
    drop(b1);
    assert_checked();

    let b2 = SecretBox::new([u8::MAX; 160]);
    core::hint::black_box(&b2);
    drop(b2);
    assert_checked();

    let b3 = ObserveSecretBox::new([u128::MAX; 10]);
    core::hint::black_box(&b3);
    drop(b3);
    assert_checked();

    let b4 = SecretBox::new([u8::MAX; 160]);
    core::hint::black_box(&b4);
    drop(b4);
    assert_checked();
}

#[cfg(feature = "std")]
#[test]
fn cstring_zeroize_test() {
    use std::ffi::CString;

    let _guard = lock();

    let mut cstring = CString::new("Hello, world!").expect("CString::new failed");
    // Register the original heap buffer. `zeroize` shrinks it to fit the lone nul byte; since
    // `ProxyAllocator` uses the default `GlobalAlloc::realloc` (alloc, copy, dealloc), that
    // frees the original buffer, which must contain only zeroes by then.
    REG_PTR.store(cstring.as_ptr().cast_mut().cast(), Relaxed);

    cstring.zeroize();
    assert_checked();
    assert_eq!(cstring.as_bytes_with_nul(), &[0]);
}
