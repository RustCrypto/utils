//! Trait impls for core slices.

use crate::{Cmov, CmovEq, Condition};
use core::slice;

// Uses 64-bit words on 64-bit targets, 32-bit everywhere else
#[cfg(not(target_pointer_width = "64"))]
type Word = u32;
#[cfg(target_pointer_width = "64")]
type Word = u64;
const WORD_SIZE: usize = size_of::<Word>();
const _: () = assert!(size_of::<usize>() <= WORD_SIZE, "unexpected word size");

/// Optimized implementation for byte slices which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// # Panics
/// - if slices have unequal lengths
impl Cmov for [u8] {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        assert_eq!(
            self.len(),
            value.len(),
            "source slice length ({}) does not match destination slice length ({})",
            value.len(),
            self.len()
        );

        cmovnz_slice_unchecked(self, value, condition);
    }
}

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`CmovEq`] at the word-level to cut down on the total number of instructions.
///
/// This is only constant-time for equal-length slices, and will short-circuit and set `output`
/// in the event the slices are of unequal length.
impl CmovEq for [u8] {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        // Short-circuit the comparison if the slices are of different lengths, and set the output
        // condition to the input condition.
        if self.len() != rhs.len() {
            *output = input;
            return;
        }

        let (self_chunks, self_remainder) = slice_as_chunks::<u8, WORD_SIZE>(self);
        let (rhs_chunks, rhs_remainder) = slice_as_chunks::<u8, WORD_SIZE>(rhs);

        for (self_chunk, rhs_chunk) in self_chunks.iter().zip(rhs_chunks.iter()) {
            let a = Word::from_ne_bytes(*self_chunk);
            let b = Word::from_ne_bytes(*rhs_chunk);
            a.cmovne(&b, input, output);
        }

        // Process the remainder a byte-at-a-time.
        for (a, b) in self_remainder.iter().zip(rhs_remainder.iter()) {
            a.cmovne(b, input, output);
        }
    }
}

/// Conditionally move `src` to `dst` in constant-time if `condition` is non-zero.
///
/// This function does not check the slices are equal-length and expects the caller to do so first.
#[inline(always)]
pub(crate) fn cmovnz_slice_unchecked(dst: &mut [u8], src: &[u8], condition: Condition) {
    let (dst_chunks, dst_remainder) = slice_as_chunks_mut::<u8, WORD_SIZE>(dst);
    let (src_chunks, src_remainder) = slice_as_chunks::<u8, WORD_SIZE>(src);

    for (dst_chunk, src_chunk) in dst_chunks.iter_mut().zip(src_chunks.iter()) {
        let mut a = Word::from_ne_bytes(*dst_chunk);
        let b = Word::from_ne_bytes(*src_chunk);
        a.cmovnz(&b, condition);
        dst_chunk.copy_from_slice(&a.to_ne_bytes());
    }

    // Process the remainder a byte-at-a-time.
    for (a, b) in dst_remainder.iter_mut().zip(src_remainder.iter()) {
        a.cmovnz(b, condition);
    }
}

/// Rust core `[T]::as_chunks` vendored because of its 1.88 MSRV.
/// TODO(tarcieri): use upstream function when we bump MSRV
#[inline]
#[track_caller]
#[must_use]
#[allow(clippy::integer_division_remainder_used)]
fn slice_as_chunks<T, const N: usize>(slice: &[T]) -> (&[[T; N]], &[T]) {
    assert!(N != 0, "chunk size must be non-zero");
    let len_rounded_down = slice.len() / N * N;
    // SAFETY: The rounded-down value is always the same or smaller than the
    // original length, and thus must be in-bounds of the slice.
    let (multiple_of_n, remainder) = unsafe { slice.split_at_unchecked(len_rounded_down) };
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N.
    let array_slice = unsafe { slice_as_chunks_unchecked(multiple_of_n) };
    (array_slice, remainder)
}

/// Rust core `[T]::as_chunks_mut` vendored because of its 1.88 MSRV.
/// TODO(tarcieri): use upstream function when we bump MSRV
#[inline]
#[track_caller]
#[must_use]
#[allow(clippy::integer_division_remainder_used)]
fn slice_as_chunks_mut<T, const N: usize>(slice: &mut [T]) -> (&mut [[T; N]], &mut [T]) {
    assert!(N != 0, "chunk size must be non-zero");
    let len_rounded_down = slice.len() / N * N;
    // SAFETY: The rounded-down value is always the same or smaller than the
    // original length, and thus must be in-bounds of the slice.
    let (multiple_of_n, remainder) = unsafe { slice.split_at_mut_unchecked(len_rounded_down) };
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N.
    let array_slice = unsafe { slice_as_chunks_unchecked_mut(multiple_of_n) };
    (array_slice, remainder)
}

/// Rust core `[T]::as_chunks_unchecked` vendored because of its 1.88 MSRV.
/// TODO(tarcieri): use upstream function when we bump MSRV
#[inline]
#[must_use]
#[track_caller]
#[allow(clippy::integer_division_remainder_used)]
unsafe fn slice_as_chunks_unchecked<T, const N: usize>(slice: &[T]) -> &[[T; N]] {
    // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
    const { debug_assert!(N != 0) };
    debug_assert_eq!(slice.len() % N, 0);
    let new_len = slice.len() / N;

    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    unsafe { slice::from_raw_parts(slice.as_ptr().cast(), new_len) }
}

/// Rust core `[T]::as_chunks_unchecked_mut` vendored because of its 1.88 MSRV.
/// TODO(tarcieri): use upstream function when we bump MSRV
#[inline]
#[must_use]
#[track_caller]
#[allow(clippy::integer_division_remainder_used)]
unsafe fn slice_as_chunks_unchecked_mut<T, const N: usize>(slice: &mut [T]) -> &mut [[T; N]] {
    // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
    const { debug_assert!(N != 0) };
    debug_assert_eq!(slice.len() % N, 0);
    let new_len = slice.len() / N;

    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    unsafe { slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), new_len) }
}
