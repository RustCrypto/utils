#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]

mod errors;
mod iter;
mod tagged_len;

pub use errors::IndexOutOfBounds;
pub use iter::Iter;

use core::{
    fmt::{self, Debug},
    iter::IntoIterator,
    ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeTo},
    panic::RefUnwindSafe,
    ptr, slice,
};
use tagged_len::TaggedLen;

/// Slice of bits: similar to `&[u8]`, but with bit-level granularity.
#[repr(transparent)]
pub struct BitSlice {
    /// Fat pointer type which carries the original pointer to a `&[u8]` and its length, but also
    /// carries additional bits for encoding the sub-byte positions of the beginning and ending of
    /// the bit slice.
    inner: [Inner],
}

/// Inaccessible placeholder ZST which is sound to construct slices of in any length (since ZST
/// slices occupy no memory regardless of their length).
type Inner = ();

impl BitSlice {
    /// An empty bit slice, the equivalent of `&[]`.
    pub const EMPTY: &Self = Self::new(&[]);

    /// Create a new immutable bit slice from an immutable byte slice.
    #[must_use]
    pub const fn new(bytes: &[u8]) -> &Self {
        Self::new_with_offsets(bytes, 0, 0)
    }

    /// Create a new immutable bit slice backed by the given byte slice, with the given bit-level
    /// offsets in to the first and last byte, which may be the same if `bytes` is 1-byte long.
    #[inline]
    const fn new_with_offsets(bytes: &[u8], head_offset: usize, tail_offset: usize) -> &Self {
        let len = TaggedLen::new(bytes.len(), head_offset, tail_offset).encode();

        // SAFETY: we are constructing a slice whose elements are `()` a.k.a. `Inner`, which is a
        // zero-sized type (ZST).
        //
        // We can't actually read or write memory via this slice itself since it's a slice of ZSTs,
        // which occupies no memory regardless of element count.
        //
        // Note that under Stacked Borrows, this loses the pointer's provenance, which doesn't
        // become an issue until we try to reconstruct the original slice (see SAFETY comment on
        // `as_raw_bytes` below). However, the provenance is preserved under Tree Borrows.
        let slice = unsafe { slice::from_raw_parts::<Inner>(bytes.as_ptr().cast(), len) };

        // SAFETY: `Self` is a `repr(transparent)` newtype for `[()]` a.k.a. `[Inner]`, so the fat
        // pointer metadata is preserved and the cast is valid.
        unsafe { &*(ptr::from_ref(slice) as *const Self) }
    }

    /// Create a new mutable bit slice from a mutable byte slice.
    #[must_use]
    pub const fn new_mut(bytes: &mut [u8]) -> &mut Self {
        Self::new_mut_with_offsets(bytes, 0, 0)
    }

    /// Create a new mutable bit slice from a mutable byte slice.
    #[must_use]
    const fn new_mut_with_offsets(
        bytes: &mut [u8],
        head_offset: usize,
        tail_offset: usize,
    ) -> &mut Self {
        let len = TaggedLen::new(bytes.len(), head_offset, tail_offset).encode();

        // SAFETY: we are using the same approach as outlined in `new`, except constructing a
        // mutable slice of a ZST which occupies no memory regardless of element count.
        //
        // The same caveats about soundness under Stacked Borrows vs Tree Borrows also hold.
        let slice = unsafe { slice::from_raw_parts_mut::<Inner>(bytes.as_mut_ptr().cast(), len) };

        // SAFETY: `Self` is a `repr(transparent)` newtype for `[()]` a.k.a. `[Inner]`, so the fat
        // pointer metadata is preserved and the cast is valid.
        unsafe { &mut *(ptr::from_mut(slice) as *mut Self) }
    }

    /// Raw access to the backing memory for this bit slice.
    #[must_use]
    const fn as_raw_bytes(&self) -> &[u8] {
        let ptr = self.inner.as_ptr();
        let len = self.tagged_len().byte_len();

        // SAFETY: `len` is the original length of the valid slice this bit slice was constructed
        // from, and the lifetime of `ptr` is tied to the lifetime of `&self` which is in turn tied
        // to the returned slice's lifetime.
        //
        // However, this particular conversion is not yet fully specified by the Rust memory model.
        // See: rust-lang/unsafe-code-guidelines#134
        //
        // Notably, Stacked Borrows loses provenance of the original pointer when it's cast to
        // `*const Inner`, so Miri considers this UB, e.g.:
        //
        // > error: Undefined Behavior: trying to retag from <177556> for SharedReadOnly permission
        // > at alloc64685[0x0], but that tag does not exist in the borrow stack for this location
        //
        // However, Tree Borrows retains the provenance and accepts this code under Miri, i.e. with
        // MIRIFLAGS="-Zmiri-tree-borrows".
        //
        // While reconstructing the original slice using its original length and a pointer cast is
        // sound on all existing versions of the Rust compiler, this doesn't necessarily hold for
        // future versions of the compiler and is still awaiting a resolution of the discrepancy
        // between Stacked Borrows and Tree Borrows. The possibility remains that this may be UB
        // in future versions of the Rust compiler.
        unsafe { slice::from_raw_parts(ptr.cast(), len) }
    }

    /// Raw mutable access to the backing memory for this bit slice.
    #[must_use]
    const fn as_mut_raw_bytes(&mut self) -> &mut [u8] {
        let ptr = self.inner.as_mut_ptr();
        let len = self.tagged_len().byte_len();

        // SAFETY: we are using the same approach as outlined in `as_raw_bytes`, with the same
        // caveats. `len` is the original length of the valid slice this bit slice was constructed
        // from, and the lifetime of `ptr` is tied to the lifetime of `&mut self` which is in turn
        // tied to the returned slice's lifetime.
        unsafe { slice::from_raw_parts_mut(ptr.cast(), len) }
    }

    /// Decode the [`TaggedLen`] for this bit slice.
    const fn tagged_len(&self) -> TaggedLen {
        TaggedLen::decode(self.inner.len())
    }

    /// Get the length of this bit slice in bits.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.tagged_len().bit_len()
    }

    /// Is this bit slice empty?
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the bits in this slice.
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    /// Return the first bit in the bit slice, or `None` if it's empty.
    #[must_use]
    pub const fn first(&self) -> Option<bool> {
        match self.get_bit(0) {
            Ok(bit) => Some(bit),
            Err(_) => None,
        }
    }

    /// Return the last bit in the bit slice, or `None` if it's empty.
    #[must_use]
    pub const fn last(&self) -> Option<bool> {
        if let Some(index) = self.len().checked_sub(1) {
            if let Ok(bit) = self.get_bit(index) {
                return Some(bit);
            }
        }

        None
    }

    /// Get the bit at the given position within the bit slice.
    ///
    /// # Errors
    /// Returns [`IndexOutOfBounds`] if `index` is past the number of bits in the slice.
    pub const fn get_bit(&self, index: usize) -> Result<bool, IndexOutOfBounds> {
        match self.tagged_len().offset_and_mask(index) {
            Ok((offset, mask)) => Ok(self.as_raw_bytes()[offset] & mask != 0),
            Err(e) => Err(e),
        }
    }

    /// Get a subslice of this bit slice.
    ///
    /// # Errors
    /// Returns [`IndexOutOfBounds`] if the given range is out-of-bounds.
    pub const fn get_slice(&self, bits: Range<usize>) -> Result<&Self, IndexOutOfBounds> {
        match self.tagged_len().slice(bits) {
            Ok((len, offset)) => {
                // Abusing `split_at` as a workaround for `const fn` slicing with dynamic positions
                let tail = self.as_raw_bytes().split_at(offset).1;
                let bytes = tail.split_at(len.byte_len()).0;
                Ok(Self::new_with_offsets(
                    bytes,
                    len.head_offset(),
                    len.tail_offset(),
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Get a mutable subslice of this bit slice.
    ///
    /// # Errors
    /// Returns [`IndexOutOfBounds`] if the given range is out-of-bounds.
    pub const fn get_mut_slice(
        &mut self,
        bits: Range<usize>,
    ) -> Result<&mut Self, IndexOutOfBounds> {
        match self.tagged_len().slice(bits) {
            Ok((len, offset)) => {
                // Abusing `split_at` as a workaround for `const fn` slicing with dynamic positions
                let tail = self.as_mut_raw_bytes().split_at_mut(offset).1;
                let bytes = tail.split_at_mut(len.byte_len()).0;
                Ok(Self::new_mut_with_offsets(
                    bytes,
                    len.head_offset(),
                    len.tail_offset(),
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Set the bit at the given position within the bit slice to the given value.
    ///
    /// # Errors
    /// Returns [`IndexOutOfBounds`] if `index` is past the number of bits in the slice.
    pub const fn set_bit(&mut self, index: usize, value: bool) -> Result<(), IndexOutOfBounds> {
        if let Err(e) = self.replace_bit(index, value) {
            return Err(e);
        }

        Ok(())
    }

    /// Set the bit at the given position within the bit slice to the given value, returning the
    /// original value.
    ///
    /// # Errors
    /// Returns [`IndexOutOfBounds`] if `index` is past the number of bits in the slice.
    pub const fn replace_bit(
        &mut self,
        index: usize,
        value: bool,
    ) -> Result<bool, IndexOutOfBounds> {
        match self.tagged_len().offset_and_mask(index) {
            Ok((offset, mask)) => {
                let orig = self.as_raw_bytes()[offset] & mask != 0;

                if value {
                    self.as_mut_raw_bytes()[offset] |= mask;
                } else {
                    self.as_mut_raw_bytes()[offset] &= !mask;
                }

                Ok(orig)
            }
            Err(e) => Err(e),
        }
    }

    /// Return the first bit and the rest of the elements of the bit slice, or `None` if it's empty.
    #[must_use]
    pub const fn split_first(&self) -> Option<(bool, &Self)> {
        match (self.first(), self.get_slice(1..self.len())) {
            (Some(bit), Ok(rest)) => Some((bit, rest)),
            _ => None,
        }
    }

    /// Return the last bit and the rest of the elements of the bit slice, or `None` if it's empty.
    #[must_use]
    pub const fn split_last(&self) -> Option<(bool, &Self)> {
        if let Some(index) = self.len().checked_sub(1) {
            match (self.get_bit(index), self.get_slice(0..index)) {
                (Ok(bit), Ok(rest)) => Some((bit, rest)),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Debug for BitSlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitSlice([")?;
        for bit in self {
            write!(f, "{}", u8::from(bit))?;
        }
        write!(f, "])")
    }
}

impl<'a> From<&'a [u8]> for &'a BitSlice {
    fn from(bytes: &'a [u8]) -> Self {
        BitSlice::new(bytes)
    }
}

impl<'a> From<&'a mut [u8]> for &'a mut BitSlice {
    fn from(bytes: &'a mut [u8]) -> Self {
        BitSlice::new_mut(bytes)
    }
}

// NOTE: can't impl `IndexMut<usize>` since we can't borrow a single bit from a byte mutably
impl Index<usize> for BitSlice {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        if self.get_bit(index).expect("index out of bounds") {
            &true
        } else {
            &false
        }
    }
}

impl Index<Range<usize>> for BitSlice {
    type Output = Self;

    fn index(&self, range: Range<usize>) -> &Self {
        self.get_slice(range).expect("index out of bounds")
    }
}

impl Index<RangeFull> for BitSlice {
    type Output = Self;

    fn index(&self, _range: RangeFull) -> &Self {
        self
    }
}

impl Index<RangeFrom<usize>> for BitSlice {
    type Output = Self;

    fn index(&self, range: RangeFrom<usize>) -> &Self {
        self.get_slice(range.start..self.len())
            .expect("index out of bounds")
    }
}

impl Index<RangeTo<usize>> for BitSlice {
    type Output = Self;

    fn index(&self, range: RangeTo<usize>) -> &Self {
        self.get_slice(0..range.end).expect("index out of bounds")
    }
}

impl IndexMut<Range<usize>> for BitSlice {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self {
        self.get_mut_slice(range).expect("index out of bounds")
    }
}

impl IndexMut<RangeFull> for BitSlice {
    fn index_mut(&mut self, _range: RangeFull) -> &mut Self {
        self
    }
}

impl IndexMut<RangeFrom<usize>> for BitSlice {
    fn index_mut(&mut self, range: RangeFrom<usize>) -> &mut Self {
        self.get_mut_slice(range.start..self.len())
            .expect("index out of bounds")
    }
}

impl IndexMut<RangeTo<usize>> for BitSlice {
    fn index_mut(&mut self, range: RangeTo<usize>) -> &mut Self {
        self.get_mut_slice(0..range.end)
            .expect("index out of bounds")
    }
}

impl<'a> IntoIterator for &'a BitSlice {
    type Item = bool;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl Eq for BitSlice {}
impl PartialEq for BitSlice {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (a, b) in self.iter().zip(other.iter()) {
            if a != b {
                return false;
            }
        }

        true
    }
}

impl RefUnwindSafe for BitSlice {}

/// Tests for private APIs.
#[cfg(test)]
mod tests {
    use crate::BitSlice;
    const BYTES: [u8; 2] = [0xa0, 0x0a];

    #[test]
    fn as_raw_bytes() {
        assert_eq!(BitSlice::new(&BYTES).as_raw_bytes(), BYTES);
    }

    #[test]
    fn as_mut_raw_bytes() {
        let mut bytes = BYTES;
        let bits = BitSlice::new_mut(&mut bytes);
        assert_eq!(bits.as_mut_raw_bytes(), BYTES);
    }
}
