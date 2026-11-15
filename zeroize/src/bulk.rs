//! Bulk volatile writes of a repeated `Copy` value.

use crate::volatile_write;
use core::mem::size_of;

/// Fill a slice with a repeated value, preserving its type and alignment.
#[inline(always)]
pub(crate) fn fill<T: Copy>(dst: &mut [T], src: T) {
    // Target 16-byte chunks
    match size_of::<T>() {
        1 => fill_chunked::<_, 16>(dst, src),
        2 => fill_chunked::<_, 8>(dst, src),
        4 => fill_chunked::<_, 4>(dst, src),
        8 => fill_chunked::<_, 2>(dst, src),
        _ => {
            for item in dst {
                volatile_write(item, src);
            }
        }
    }
}

/// Prepare the repeated pattern and handle the tail independently of the bulk backend.
#[inline(always)]
fn fill_chunked<T: Copy, const N: usize>(dst: &mut [T], src: T) {
    let (chunks, tail) = as_chunks_mut::<T, N>(dst);
    if !chunks.is_empty() {
        write_chunks(chunks, [src; N]);
    }
    for item in tail {
        volatile_write(item, src);
    }
}

/// Portable backend for writing all complete chunks.
#[inline(always)]
fn write_chunks<T: Copy, const N: usize>(dst: &mut [[T; N]], pattern: [T; N]) {
    for chunk in dst {
        volatile_write(chunk, pattern);
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
