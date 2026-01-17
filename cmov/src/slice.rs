//! Trait impls for core slices.

use crate::{Cmov, CmovEq, Condition};
use core::{
    ops::{BitOrAssign, Shl},
    slice,
};

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

// Optimized implementation for byte slices which coalesces them into word-sized chunks first,
// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
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

        cmovnz_remainder(dst_remainder, src_remainder, condition);
    }
}

// Optimized implementation for slices of `u16` which coalesces them into word-sized chunks first,
// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
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

        cmovnz_remainder(dst_remainder, src_remainder, condition);
    }
}

// Optimized implementation for slices of `u16` which coalesces them into word-sized chunks first,
// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
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

        cmovnz_remainder(dst_remainder, src_remainder, condition);
    }
}

/// Implement [`Cmov`] using a simple loop.
macro_rules! impl_cmov_with_loop {
    ( $($int:ty),+ ) => {
        $(
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
        )+
    };
}

impl_cmov_with_loop!(u32, u64, u128);

macro_rules! assert_size_and_alignment_eq {
    ($int:ty, $uint:ty) => {
        const {
            assert!(
                size_of::<$int>() == size_of::<$uint>(),
                "integers are of unequal size"
            );

            assert!(
                align_of::<$int>() == align_of::<$uint>(),
                "integers have unequal alignment"
            );
        }
    };
}

/// Implement [`Cmov`] for a signed type by invoking the corresponding unsigned impl.
macro_rules! impl_cmov_for_signed_with_unsigned {
    ($int:ty, $uint:ty) => {
        impl Cmov for [$int] {
            #[inline]
            #[track_caller]
            #[allow(unsafe_code)]
            fn cmovnz(&mut self, value: &Self, condition: Condition) {
                assert_size_and_alignment_eq!($int, $uint);

                // SAFETY:
                // - Slices being constructed are of same-sized integers as asserted above.
                // - We source the slice length directly from the other valid slice.

                let self_unsigned = unsafe { cast_slice_mut::<$int, $uint>(self) };
                let value_unsigned = unsafe { cast_slice::<$int, $uint>(value) };
                self_unsigned.cmovnz(value_unsigned, condition);
            }
        }
    };
}

/// Implement [`CmovEq`] for a signed type by invoking the corresponding unsigned impl.
macro_rules! impl_cmoveq_for_signed_with_unsigned {
    ($int:ty, $uint:ty) => {
        impl CmovEq for [$int] {
            #[inline]
            #[allow(unsafe_code)]
            fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                // SAFETY:
                // - Slices being constructed are of same-sized integers as asserted above.
                // - We source the slice length directly from the other valid slice.
                let self_unsigned = unsafe { cast_slice::<$int, $uint>(self) };
                let rhs_unsigned = unsafe { cast_slice::<$int, $uint>(rhs) };
                self_unsigned.cmovne(rhs_unsigned, input, output);
            }
        }
    };
}

/// Implement [`Cmov`] and [`CmovEq`] for the given signed/unsigned type pair.
// TODO(tarcieri): use `cast_unsigned`/`cast_signed` to get rid of the `=> u*`
macro_rules! impl_cmov_traits_for_signed_with_unsigned {
    ( $($int:ty => $uint:ty),+ ) => {
        $(
            impl_cmov_for_signed_with_unsigned!($int, $uint);
            impl_cmoveq_for_signed_with_unsigned!($int, $uint);
        )+
    };
}

impl_cmov_traits_for_signed_with_unsigned!(
    i8 => u8,
    i16 => u16,
    i32 => u32,
    i64 => u64,
    i128 => u128
);

// Optimized implementation for byte slices which coalesces them into word-sized chunks first,
// then performs [`CmovEq`] at the word-level to cut down on the total number of instructions.
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

        cmovne_remainder(self_remainder, rhs_remainder, input, output);
    }
}

/// Implement [`CmovEq`] using a simple loop.
macro_rules! impl_cmoveq_with_loop {
    ( $($int:ty),+ ) => {
        $(
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
        )+
    };
}

// TODO(tarcieri): investigate word-coalescing impls
impl_cmoveq_with_loop!(u16, u32, u64, u128);

/// Performs an unsafe pointer cast from one slice type to the other.
///
/// # Panics
/// - If `T` and `U` differ in size
/// - If `T` and `U` differ in alignment
#[allow(unsafe_code)]
unsafe fn cast_slice<T, U>(slice: &[T]) -> &[U] {
    const {
        assert!(size_of::<T>() == size_of::<U>(), "T/U size differs");
        assert!(align_of::<T>() == align_of::<U>(), "T/U alignment differs");
    }

    // SAFETY:
    // - Slices being constructed are of same-sized/aligned types as asserted above.
    // - We source the slice length directly from the other valid slice.
    // - It's up to the caller to ensure the pointer cast itself is valid.
    unsafe { slice::from_raw_parts(slice.as_ptr() as *const U, slice.len()) }
}

/// Performs an unsafe pointer cast from one mutable slice type to the other.
///
/// # Panics
/// - If `T` and `U` differ in size
/// - If `T` and `U` differ in alignment
#[allow(unsafe_code)]
unsafe fn cast_slice_mut<T, U>(slice: &mut [T]) -> &mut [U] {
    const {
        assert!(size_of::<T>() == size_of::<U>(), "T/U size differs");
        assert!(align_of::<T>() == align_of::<U>(), "T/U alignment differs");
    }

    // SAFETY:
    // - Slices being constructed are of same-sized/aligned types as asserted above.
    // - We source the slice length directly from the other valid slice.
    // - It's up to the caller to ensure the pointer cast itself is valid.
    unsafe { slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut U, slice.len()) }
}

/// Compare the two remainder slices by loading a `Word` then performing `cmovne`.
#[inline]
fn cmovne_remainder<T>(
    a_remainder: &[T],
    b_remainder: &[T],
    input: Condition,
    output: &mut Condition,
) where
    T: Copy,
    Word: From<T>,
{
    let a = slice_to_word(a_remainder);
    let b = slice_to_word(b_remainder);
    a.cmovne(&b, input, output);
}

/// Load the remainder from chunking the slice into a single `Word`, perform `cmovnz`, then write
/// the result back out to `dst_remainder`.
#[inline]
fn cmovnz_remainder<T>(dst_remainder: &mut [T], src_remainder: &[T], condition: Condition)
where
    T: BitOrAssign + Copy + From<u8> + Shl<usize, Output = T>,
    Word: From<T>,
{
    let mut remainder = slice_to_word(dst_remainder);
    remainder.cmovnz(&slice_to_word(src_remainder), condition);
    word_to_slice(remainder, dst_remainder);
}

/// Create a [`Word`] from the given input slice.
#[inline]
fn slice_to_word<T>(slice: &[T]) -> Word
where
    T: Copy,
    Word: From<T>,
{
    debug_assert!(size_of_val(slice) <= WORD_SIZE, "slice too large");
    slice
        .iter()
        .rev()
        .copied()
        .fold(0, |acc, n| (acc << (size_of::<T>() * 8)) | Word::from(n))
}

/// Serialize [`Word`] as bytes using the same byte ordering as `slice_to_word`.
#[inline]
fn word_to_slice<T>(word: Word, out: &mut [T])
where
    T: BitOrAssign + Copy + From<u8> + Shl<usize, Output = T>,
{
    debug_assert!(size_of::<T>() > 0, "can't be used with ZSTs");
    debug_assert!(out.len() <= WORD_SIZE, "slice too large");

    let bytes = word.to_le_bytes();
    for (o, chunk) in out.iter_mut().zip(bytes.chunks(size_of::<T>())) {
        *o = T::from(0u8);
        for (i, &byte) in chunk.iter().enumerate() {
            *o |= T::from(byte) << (i * 8);
        }
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

#[cfg(test)]
mod tests {
    #[test]
    fn cmovnz_remainder() {
        // - Test endianness handling on non-64-bit platforms
        // - Test handling of odd length slices on 64-bit platforms
        #[cfg(not(target_pointer_width = "64"))]
        const A_U16: [u16; 2] = [0xAAAA, 0xBBBB];
        #[cfg(target_pointer_width = "64")]
        const A_U16: [u16; 3] = [0xAAAA, 0xBBBB, 0xCCCC];

        #[cfg(not(target_pointer_width = "64"))]
        const B_U16: [u16; 2] = [0x10, 0xFFFF];
        #[cfg(target_pointer_width = "64")]
        const B_U16: [u16; 3] = [0x10, 0x10, 0xFFFF];

        let mut out = A_U16;

        super::cmovnz_remainder(&mut out, &B_U16, 0);
        assert_eq!(A_U16, out);

        super::cmovnz_remainder(&mut out, &B_U16, 1);
        assert_eq!(B_U16, out);
    }

    #[test]
    fn slice_to_word() {
        assert_eq!(0xAABBCC, super::slice_to_word(&[0xCCu8, 0xBB, 0xAA]));
        assert_eq!(0xAAAABBBB, super::slice_to_word(&[0xBBBBu16, 0xAAAA]));

        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            0xAAAABBBBCCCC,
            super::slice_to_word(&[0xCCCCu16, 0xBBBB, 0xAAAA])
        );
    }

    #[test]
    fn word_to_slice() {
        let mut out = [0u8; 3];
        super::word_to_slice(0xAABBCC, &mut out);
        assert_eq!(&[0xCC, 0xBB, 0xAA], &out);

        let mut out = [0u16; 2];
        super::word_to_slice(0xAAAABBBB, &mut out);
        assert_eq!(&[0xBBBB, 0xAAAA], &out);

        #[cfg(target_pointer_width = "64")]
        {
            let mut out = [0u16; 3];
            super::word_to_slice(0xAAAABBBBCCCC, &mut out);
            assert_eq!(&[0xCCCC, 0xBBBB, 0xAAAA], &out);
        }
    }
}
