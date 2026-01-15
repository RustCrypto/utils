#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::checked_conversions,
    clippy::implicit_saturating_sub,
    clippy::integer_division_remainder_used,
    clippy::mod_module_files,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::unwrap_used,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused_lifetimes,
    unused_qualifications
)]

#[cfg(not(miri))]
#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")),
    miri
))]
mod portable;
#[cfg(not(miri))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Condition
pub type Condition = u8;

/// Conditional move
pub trait Cmov {
    /// Move if non-zero.
    ///
    /// Uses a `test` instruction to check if the given `condition` value is
    /// equal to zero, conditionally moves `value` to `self` when `condition` is
    /// not equal to zero.
    fn cmovnz(&mut self, value: &Self, condition: Condition);

    /// Move if zero.
    ///
    /// Uses a `cmp` instruction to check if the given `condition` value is
    /// equal to zero, and if so, conditionally moves `value` to `self`
    /// when `condition` is equal to zero.
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        self.cmovnz(value, !condition)
    }
}

/// Conditional move with equality comparison
pub trait CmovEq {
    /// Move if both inputs are equal.
    ///
    /// Uses a `xor` instruction to compare the two values, and
    /// conditionally moves `input` to `output` when they are equal.
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition);

    /// Move if both inputs are not equal.
    ///
    /// Uses a `xor` instruction to compare the two values, and
    /// conditionally moves `input` to `output` when they are not equal.
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let mut tmp = 1u8;
        self.cmoveq(rhs, 0u8, &mut tmp);
        tmp.cmoveq(&1u8, input, output);
    }
}

impl Cmov for u8 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = u16::from(*self);
        tmp.cmovnz(&u16::from(*value), condition);
        debug_assert!(u8::try_from(tmp).is_ok());
        *self = (tmp & 0xFF) as u8;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = u16::from(*self);
        tmp.cmovz(&u16::from(*value), condition);
        debug_assert!(u8::try_from(tmp).is_ok());
        *self = (tmp & 0xFF) as u8;
    }
}

impl Cmov for u128 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovnz(&((*value & 0xFFFF_FFFF_FFFF_FFFF) as u64), condition);
        hi.cmovnz(&((*value >> 64) as u64), condition);

        *self = u128::from(lo) | (u128::from(hi) << 64);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovz(&((*value & 0xFFFF_FFFF_FFFF_FFFF) as u64), condition);
        hi.cmovz(&((*value >> 64) as u64), condition);

        *self = u128::from(lo) | (u128::from(hi) << 64);
    }
}

impl CmovEq for u8 {
    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        u16::from(*self).cmoveq(&u16::from(*rhs), input, output);
    }

    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        u16::from(*self).cmovne(&u16::from(*rhs), input, output);
    }
}

impl CmovEq for u128 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let hi = (*self >> 64) as u64;

        let mut tmp = 1u8;
        lo.cmovne(&((*rhs & 0xFFFF_FFFF_FFFF_FFFF) as u64), 0, &mut tmp);
        hi.cmovne(&((*rhs >> 64) as u64), 0, &mut tmp);
        tmp.cmoveq(&0, input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let hi = (*self >> 64) as u64;

        let mut tmp = 1u8;
        lo.cmovne(&((*rhs & 0xFFFF_FFFF_FFFF_FFFF) as u64), 0, &mut tmp);
        hi.cmovne(&((*rhs >> 64) as u64), 0, &mut tmp);
        tmp.cmoveq(&1, input, output);
    }
}

// Impl `Cmov*` by first casting to unsigned then using the unsigned `Cmov` impls
// TODO(tarcieri): use `cast_unsigned`/`cast_signed` to get rid of the `=> u*`
macro_rules! impl_cmov_traits_for_signed_ints {
    ( $($int:ty => $uint:ty),+ ) => {
        $(
            impl Cmov for $int {
                #[inline]
                fn cmovnz(&mut self, value: &Self, condition: Condition) {
                    let mut tmp = *self as $uint;
                    tmp.cmovnz(&(*value as $uint), condition);
                    *self = tmp as $int;
                }

                #[inline]
                fn cmovz(&mut self, value: &Self, condition: Condition) {
                    let mut tmp = *self as $uint;
                    tmp.cmovz(&(*value as $uint), condition);
                    *self = tmp as $int;
                }
            }

            impl CmovEq for $int {
                #[inline]
                fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                    (*self as $uint).cmoveq(&(*rhs as $uint), input, output);
                }

                #[inline]
                fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                    (*self as $uint).cmovne(&(*rhs as $uint), input, output);
                }
            }
        )+
    };
}

impl_cmov_traits_for_signed_ints!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128);

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// With compile-time knowledge of `N`, the compiler should also be able to unroll the loops in
/// cases where efficiency would benefit, reducing the implementation to a sequence of word-sized
/// [`Cmov`] ops (and if `N` isn't word-aligned, followed by a series of 1-byte ops).
impl<const N: usize> Cmov for [u8; N] {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        // Uses 64-bit words on 64-bit targets, 32-bit everywhere else
        #[cfg(not(target_pointer_width = "64"))]
        type Chunk = u32;
        #[cfg(target_pointer_width = "64")]
        type Chunk = u64;
        const CHUNK_SIZE: usize = size_of::<Chunk>();

        // Load a chunk from a byte slice
        // TODO(tarcieri): use `array_chunks` when stable (rust-lang/rust##100450)
        #[inline]
        fn load_chunk(slice: &[u8]) -> Chunk {
            Chunk::from_ne_bytes(slice.try_into().expect("should be the right size"))
        }

        let mut self_chunks = self.chunks_exact_mut(CHUNK_SIZE);
        let mut value_chunks = value.chunks_exact(CHUNK_SIZE);

        // Process as much input as we can a `Chunk`-at-a-time.
        for (self_chunk, value_chunk) in self_chunks.by_ref().zip(value_chunks.by_ref()) {
            let mut a = load_chunk(self_chunk);
            let b = load_chunk(value_chunk);
            a.cmovnz(&b, condition);
            self_chunk.copy_from_slice(&a.to_ne_bytes());
        }

        // Process the remainder a byte-at-a-time.
        for (a, b) in self_chunks
            .into_remainder()
            .iter_mut()
            .zip(value_chunks.remainder().iter())
        {
            a.cmovnz(b, condition);
        }
    }
}

impl<T: CmovEq> CmovEq for [T] {
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let mut tmp = 1u8;
        self.cmovne(rhs, 0u8, &mut tmp);
        tmp.cmoveq(&1, input, output);
    }

    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        // Short-circuit the comparison if the slices are of different lengths, and set the output
        // condition to the input condition.
        if self.len() != rhs.len() {
            *output = input;
            return;
        }

        // Compare each byte.
        for (a, b) in self.iter().zip(rhs.iter()) {
            a.cmovne(b, input, output);
        }
    }
}
