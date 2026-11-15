//! Bulk byte zeroization and volatile writes of a repeated `Copy` value.

use crate::volatile_write;
use core::mem::{MaybeUninit, size_of};

/// Securely zero a byte slice, preventing the compiler from eliminating the writes.
///
/// # Examples
/// ```
/// let mut secret = [42u8; 32];
/// zeroize::zeroize_bytes(&mut secret);
/// assert_eq!(secret, [0u8; 32]);
/// ```
#[inline]
pub fn zeroize_bytes(bytes: &mut [u8]) {
    fill_slice(bytes, 0);
}

/// Zero bytes which may be uninitialized, including padding.
///
/// # Safety
/// For nonzero `len`, `dst` must be non-NULL and provide exclusive access to `len` writable bytes
/// within a single allocation.
///
/// The size must not exceed `isize::MAX` and the address range must not wrap. If zeroes are not a
/// valid bit pattern for the underlying type, the caller must restore a valid value before any
/// typed access or drop.
#[inline]
pub(crate) unsafe fn zeroize_byte_ptr(dst: *mut u8, len: usize) {
    if len == 0 {
        return;
    }

    // SAFETY: the caller supplies exclusive access to this range. `MaybeUninit<u8>` permits
    // uninitialized bytes and has byte alignment.
    let bytes = unsafe { core::slice::from_raw_parts_mut(dst.cast::<MaybeUninit<u8>>(), len) };
    fill_slice(bytes, MaybeUninit::new(0));
}

/// Fill slice using an `asm!` optimization barrier or portable volatile writes.
#[inline]
fn fill_slice<T: Copy>(slice: &mut [T], value: T) {
    #[cfg(all(
        not(miri),
        any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "arm64ec",
            target_arch = "loongarch64",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "s390x",
            target_arch = "x86",
            target_arch = "x86_64",
        )
    ))]
    {
        slice.fill(value);
        crate::optimization_barrier(slice);
    }

    #[cfg(not(all(
        not(miri),
        any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "arm64ec",
            target_arch = "loongarch64",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "s390x",
            target_arch = "x86",
            target_arch = "x86_64",
        )
    )))]
    {
        fill_volatile(slice, value);
    }
}

/// Fill a slice with a repeated value, preserving its type and alignment.
#[inline(always)]
pub(crate) fn fill_volatile<T: Copy>(dst: &mut [T], src: T) {
    // Target 16-byte chunks
    match size_of::<T>() {
        0 => {}
        1 => fill_chunked::<_, 16>(dst, src),
        2 => fill_chunked::<_, 8>(dst, src),
        4 => fill_chunked::<_, 4>(dst, src),
        8 => fill_chunked::<_, 2>(dst, src),
        _ => write_elements(dst, src),
    }
}

/// Prepare the repeated pattern and handle the tail independently of the bulk backend.
#[inline(always)]
fn fill_chunked<T: Copy, const N: usize>(dst: &mut [T], src: T) {
    let (chunks, tail) = as_chunks_mut::<T, N>(dst);

    if !chunks.is_empty() {
        write_elements(chunks, [src; N]);
    }

    write_elements(tail, src);
}

/// Write individual elements or complete chunks with the same volatile primitive.
#[inline(always)]
fn write_elements<T: Copy>(dst: &mut [T], src: T) {
    for item in dst {
        volatile_write(item, src);
    }
}

/// MSRV-compatible copy of core's `slice::as_chunks_mut` taken from Rust 1.88.0.
/// Source: <https://github.com/rust-lang/rust/blob/1.88.0/library/core/src/slice/mod.rs>
// TODO(tarcieri): use `slice::as_chunks_mut` when MSRV is 1.88 or newer.
#[inline]
#[track_caller]
#[must_use]
const fn as_chunks_mut<T, const N: usize>(slice: &mut [T]) -> (&mut [[T; N]], &mut [T]) {
    assert!(N != 0, "chunk size must be non-zero");
    let len_rounded_down = slice.len() / N * N;
    // SAFETY: The rounded-down value is always the same or smaller than the
    // original length, and thus must be in-bounds of the slice.
    let (multiple_of_n, remainder) = unsafe { slice.split_at_mut_unchecked(len_rounded_down) };
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N. Arrays have the same alignment as T.
    let array_slice = unsafe {
        core::slice::from_raw_parts_mut(multiple_of_n.as_mut_ptr().cast(), multiple_of_n.len() / N)
    };
    (array_slice, remainder)
}
