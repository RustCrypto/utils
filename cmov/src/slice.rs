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

/// Assert the lengths of the two slices are equal.
macro_rules! assert_lengths_eq {
    ($a:expr, $b:expr) => {
        assert_eq!(
            $a, $b,
            "source slice length ({}) does not match destination slice length ({})",
            $b, $a
        );
    };
}

/// Optimized implementation for byte slices which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// # Panics
/// - if slices have unequal lengths
impl Cmov for [u8] {
    #[inline]
    #[track_caller]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        assert_lengths_eq!(self.len(), value.len());

        let (dst_chunks, dst_remainder) = slice_as_chunks_mut::<u8, WORD_SIZE>(self);
        let (src_chunks, src_remainder) = slice_as_chunks::<u8, WORD_SIZE>(value);

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
}

/// Optimized implementation for slices of `u16` which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// # Panics
/// - if slices have unequal lengths
#[cfg(not(target_pointer_width = "64"))]
#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for [u16] {
    #[inline]
    #[track_caller]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        assert_lengths_eq!(self.len(), value.len());

        let (dst_chunks, dst_remainder) = slice_as_chunks_mut::<u16, 2>(self);
        let (src_chunks, src_remainder) = slice_as_chunks::<u16, 2>(value);

        for (dst_chunk, src_chunk) in dst_chunks.iter_mut().zip(src_chunks.iter()) {
            let mut a = Word::from(dst_chunk[0]) | (Word::from(dst_chunk[1]) << 16);
            let b = Word::from(src_chunk[0]) | (Word::from(src_chunk[1]) << 16);
            a.cmovnz(&b, condition);
            dst_chunk[0] = (a & 0xFFFF) as u16;
            dst_chunk[1] = (a >> 16) as u16;
        }

        // If slice is odd-length
        if !dst_remainder.is_empty() {
            dst_remainder[0].cmovnz(&src_remainder[0], condition);
        }
    }
}

/// Optimized implementation for slices of `u16` which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// # Panics
/// - if slices have unequal lengths
#[cfg(target_pointer_width = "64")]
#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for [u16] {
    #[inline]
    #[track_caller]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        assert_lengths_eq!(self.len(), value.len());

        #[inline(always)]
        fn u16x4_to_u64(input: &[u16; 4]) -> u64 {
            Word::from(input[0])
                | (Word::from(input[1]) << 16)
                | (Word::from(input[2]) << 32)
                | (Word::from(input[3]) << 48)
        }

        let (dst_chunks, dst_remainder) = slice_as_chunks_mut::<u16, 4>(self);
        let (src_chunks, src_remainder) = slice_as_chunks::<u16, 4>(value);

        for (dst_chunk, src_chunk) in dst_chunks.iter_mut().zip(src_chunks.iter()) {
            let mut a = u16x4_to_u64(dst_chunk);
            let b = u16x4_to_u64(src_chunk);
            a.cmovnz(&b, condition);
            dst_chunk[0] = (a & 0xFFFF) as u16;
            dst_chunk[1] = ((a >> 16) & 0xFFFF) as u16;
            dst_chunk[2] = ((a >> 32) & 0xFFFF) as u16;
            dst_chunk[3] = ((a >> 48) & 0xFFFF) as u16;
        }

        for (a, b) in dst_remainder.iter_mut().zip(src_remainder.iter()) {
            a.cmovnz(b, condition);
        }
    }
}

/// Implementation for slices of `u32` on 32-bit platforms, where we can just loop.
///
/// # Panics
/// - if slices have unequal lengths
#[cfg(not(target_pointer_width = "64"))]
#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for [u32] {
    #[inline]
    #[track_caller]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        assert_lengths_eq!(self.len(), value.len());

        for (a, b) in self.iter_mut().zip(value.iter()) {
            a.cmovnz(b, condition);
        }
    }
}

/// Optimized implementation for slices of `u32` which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// # Panics
/// - if slices have unequal lengths
#[cfg(target_pointer_width = "64")]
#[cfg_attr(docsrs, doc(cfg(true)))]
impl Cmov for [u32] {
    #[inline]
    #[track_caller]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        assert_lengths_eq!(self.len(), value.len());

        let (dst_chunks, dst_remainder) = slice_as_chunks_mut::<u32, 2>(self);
        let (src_chunks, src_remainder) = slice_as_chunks::<u32, 2>(value);

        for (dst_chunk, src_chunk) in dst_chunks.iter_mut().zip(src_chunks.iter()) {
            let mut a = Word::from(dst_chunk[0]) | (Word::from(dst_chunk[1]) << 32);
            let b = Word::from(src_chunk[0]) | (Word::from(src_chunk[1]) << 32);
            a.cmovnz(&b, condition);
            dst_chunk[0] = (a & 0xFFFF_FFFF) as u32;
            dst_chunk[1] = (a >> 32) as u32;
        }

        // If slice is odd-length
        if !dst_remainder.is_empty() {
            dst_remainder[0].cmovnz(&src_remainder[0], condition);
        }
    }
}

/// Implement [`Cmov`] using a simple loop.
macro_rules! impl_cmov_with_loop {
    ($int:ty, $doc:expr) => {
        #[doc = $doc]
        #[doc = "# Panics"]
        #[doc = "- if slices have unequal lengths"]
        impl Cmov for [$int] {
            #[inline]
            #[track_caller]
            fn cmovnz(&mut self, value: &Self, condition: Condition) {
                assert_lengths_eq!(self.len(), value.len());
                for (a, b) in self.iter_mut().zip(value.iter()) {
                    a.cmovnz(b, condition);
                }
            }
        }
    };
}

impl_cmov_with_loop!(
    u64,
    "Implementation for `u64` slices where we can just loop."
);
impl_cmov_with_loop!(
    u128,
    "Implementation for `u128` slices where we can just loop."
);

macro_rules! assert_size_and_alignment_eq {
    ($signed:ty, $unsigned:ty) => {
        const {
            assert!(
                size_of::<$signed>() == size_of::<$unsigned>(),
                "integers are of unequal size"
            );

            assert!(
                align_of::<$signed>() == align_of::<$unsigned>(),
                "integers have unequal alignment"
            );
        }
    };
}

/// Implement [`Cmov`] for a signed type by invoking the corresponding unsigned impl.
macro_rules! impl_cmov_for_signed_with_unsigned {
    ($signed:ty, $unsigned:ty) => {
        impl_cmov_for_signed_with_unsigned!(
            $signed,
            $unsigned,
            "Delegating implementation of `Cmov` for signed type which delegates to unsigned."
        );
    };
    ($signed:ty, $unsigned:ty, $doc:expr) => {
        #[doc = $doc]
        #[doc = "# Panics"]
        #[doc = "- if slices have unequal lengths"]
        impl Cmov for [$signed] {
            #[inline]
            #[track_caller]
            fn cmovnz(&mut self, value: &Self, condition: Condition) {
                assert_size_and_alignment_eq!($signed, $unsigned);

                // SAFETY:
                // - Slices being constructed are of same-sized integers as asserted above.
                // - We source the slice length directly from the other valid slice.
                #[allow(unsafe_code)]
                let (self_unsigned, value_unsigned) = unsafe {
                    (
                        slice::from_raw_parts_mut(self.as_mut_ptr() as *mut $unsigned, self.len()),
                        slice::from_raw_parts(value.as_ptr() as *const $unsigned, value.len()),
                    )
                };

                self_unsigned.cmovnz(value_unsigned, condition);
            }
        }
    };
}

/// Implement [`CmovEq`] for a signed type by invoking the corresponding unsigned impl.
macro_rules! impl_cmoveq_for_signed_with_unsigned {
    ($signed:ty, $unsigned:ty) => {
        impl_cmoveq_for_signed_with_unsigned!(
            $signed,
            $unsigned,
            "Delegating implementation of `CmovEq` for signed type which delegates to unsigned."
        );
    };
    ($signed:ty, $unsigned:ty, $doc:expr) => {
        #[doc = $doc]
        #[doc = "# Panics"]
        #[doc = "- if slices have unequal lengths"]
        impl CmovEq for [$signed] {
            #[inline]
            fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                assert_size_and_alignment_eq!($signed, $unsigned);

                // SAFETY:
                // - Slices being constructed are of same-sized integers as asserted above.
                // - We source the slice length directly from the other valid slice.
                #[allow(unsafe_code)]
                let (self_unsigned, rhs_unsigned) = unsafe {
                    (
                        slice::from_raw_parts(self.as_ptr() as *const $unsigned, self.len()),
                        slice::from_raw_parts(rhs.as_ptr() as *const $unsigned, rhs.len()),
                    )
                };

                self_unsigned.cmovne(rhs_unsigned, input, output);
            }
        }
    };
}

/// Implement [`Cmov`] and [`CmovEq`] for the given signed/unsigned type pair.
macro_rules! impl_cmov_traits_for_signed_with_unsigned {
    ($signed:ty, $unsigned:ty) => {
        impl_cmov_for_signed_with_unsigned!($signed, $unsigned);
        impl_cmoveq_for_signed_with_unsigned!($signed, $unsigned);
    };
}

impl_cmov_traits_for_signed_with_unsigned!(i8, u8);
impl_cmov_traits_for_signed_with_unsigned!(i16, u16);
impl_cmov_traits_for_signed_with_unsigned!(i32, u32);
impl_cmov_traits_for_signed_with_unsigned!(i64, u64);
impl_cmov_traits_for_signed_with_unsigned!(i128, u128);

/// Optimized implementation for byte slices which coalesces them into word-sized chunks first,
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

/// Implement [`CmovEq`] using a simple loop.
macro_rules! impl_cmoveq_with_loop {
    ($int:ty, $doc:expr) => {
        #[doc = $doc]
        impl CmovEq for [$int] {
            #[inline]
            fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                // Short-circuit the comparison if the slices are of different lengths, and set the output
                // condition to the input condition.
                if self.len() != rhs.len() {
                    *output = input;
                    return;
                }

                for (a, b) in self.iter().zip(rhs.iter()) {
                    a.cmovne(b, input, output);
                }
            }
        }
    };
}

// TODO(tarcieri): optimized coalescing implementations of `CmovEq` for `u16` and `u32`
impl_cmoveq_with_loop!(
    u16,
    "Implementation for `u16` slices where we can just loop."
);
impl_cmoveq_with_loop!(
    u32,
    "Implementation for `u32` slices where we can just loop."
);
impl_cmoveq_with_loop!(
    u64,
    "Implementation for `u64` slices where we can just loop."
);
impl_cmoveq_with_loop!(
    u128,
    "Implementation for `u128` slices where we can just loop."
);

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
