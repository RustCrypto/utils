//! `TaggedLen` encodes additional position data into the length field of a fat pointer, ala a
//! normal tagged pointer.

use crate::IndexOutOfBounds;
use core::{
    fmt::{self, Debug},
    ops::Range,
};

/// Panic message in the event of overflow.
const OVERFLOW_MSG: &str = "overflow";

/// Tagged length which encodes the original length of the byte slice.
///
/// This includes the length of the original `&[u8]` verbatim, but left-shifted by 6-bits to make
/// room for two 3-bit sub-byte position cursors within the head and tail bytes (which may refer to
/// the same byte in the event the `&[u8]` is only 1-byte in length).
///
/// They are encoded as follows in little endian:
///
/// ```text
/// | tail offset (3-bits) | head offset (3-bits) | byte len (remaining bits in `usize`) |
/// ```
///
/// The tail and head offsets are stored as a special 3-bit `u3` type, and they encode offsets
/// within a particular byte .
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TaggedLen {
    /// Length of the underlying slice in bytes.
    byte_len: usize,

    /// Bit offset within the head byte.
    head_offset: u3,

    /// Bit offset within the tail byte.
    tail_offset: u3,
}

impl TaggedLen {
    /// Create a new tagged length from the original length in bytes, with the given bit-level
    /// offsets into the first and last bytes, which may be the same byte if `byte_len` is 1.
    ///
    /// # Panics
    /// In debug builds, panics if either of the following are true:
    /// - `head_offset` or `tail_offset` overflows `u3::MAX`.
    /// - `byte_len` is zero and either `head_offset` or `tail_offset` are nonzero.
    #[inline]
    pub(crate) const fn new(byte_len: usize, head_offset: usize, tail_offset: usize) -> Self {
        debug_assert!(head_offset <= u3::MAX && tail_offset <= u3::MAX);
        debug_assert!(byte_len > 0 || (head_offset == 0 && tail_offset == 0));

        Self {
            byte_len,
            head_offset: u3::new(head_offset),
            tail_offset: u3::new(tail_offset),
        }
    }

    /// Slice the bit range represented by this `TaggedLen`, returning a new one along with a
    /// byte-level offset into the backing slice.
    pub(crate) const fn slice(self, bits: Range<usize>) -> Result<(Self, usize), IndexOutOfBounds> {
        if bits.start > bits.end || bits.end > self.bit_len() {
            return Err(IndexOutOfBounds);
        }

        let (head_byte, head_offset) = self.offsets_unchecked(bits.start);
        let (tail_byte, tail_offset) = self.offsets_unchecked(bits.end);

        if let Some(mut byte_len) = tail_byte.checked_sub(head_byte) {
            // The computed `byte_len` won't include this extra byte if `tail_offset` is non-zero.
            if !tail_offset.is_zero() {
                byte_len += 1;
            }

            let sliced_len = Self {
                byte_len,
                head_offset,
                tail_offset,
            };

            return Ok((sliced_len, head_byte));
        }

        Err(IndexOutOfBounds)
    }

    /// Get the length of this slice in bytes.
    pub(crate) const fn byte_len(self) -> usize {
        self.byte_len
    }

    /// Get the length of this slice in bits.
    pub(crate) const fn bit_len(self) -> usize {
        let unsliced_bits = self.byte_len.checked_mul(8).expect(OVERFLOW_MSG);
        let sliced_bits = self.head_offset.value()
            + if self.tail_offset.is_zero() {
                0
            } else {
                8 - self.tail_offset.value()
            };

        unsliced_bits.checked_sub(sliced_bits).expect(OVERFLOW_MSG)
    }

    /// Decode an encoded tagged length back into this type.
    pub(crate) const fn decode(encoded: usize) -> Self {
        let byte_len = encoded >> (u3::BITS * 2);
        let head_offset = (encoded >> u3::BITS) & u3::MAX;
        let tail_offset = encoded & u3::MAX;
        Self::new(byte_len, head_offset, tail_offset)
    }

    /// Encode a tagged length for storage in a fat pointer.
    pub(crate) const fn encode(self) -> usize {
        self.byte_len.checked_shl(u3::BITS * 2).expect(OVERFLOW_MSG)
            | (self.head_offset.value() << u3::BITS)
            | self.tail_offset.value()
    }

    /// Get the head offset.
    pub(crate) const fn head_offset(self) -> usize {
        self.head_offset.value()
    }

    /// Get the tail offset.
    pub(crate) const fn tail_offset(self) -> usize {
        self.tail_offset.value()
    }

    /// Compute the byte-level offset and bitmask for a bit at the given `index`.
    pub(crate) const fn offset_and_mask(
        self,
        index: usize,
    ) -> Result<(usize, u8), IndexOutOfBounds> {
        if index >= self.bit_len() {
            return Err(IndexOutOfBounds);
        }

        let (byte_offset, bit_offset) = self.offsets_unchecked(index);
        Ok((byte_offset, bit_offset.bitmask()))
    }

    /// Compute the byte and bit offsets for a given index.
    ///
    /// Does not ensure that `index` lies within this bit slice.
    const fn offsets_unchecked(self, index: usize) -> (usize, u3) {
        let index = index
            .checked_add(self.head_offset.value())
            .expect(OVERFLOW_MSG);

        let byte_offset = index >> u3::BITS;
        let bit_offset = u3::new(index);
        (byte_offset, bit_offset)
    }
}

/// A 3-bit integer, used to represent a specific bit within a byte.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
enum u3 {
    #[default]
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
}

impl u3 {
    /// Size in bits.
    const BITS: u32 = 3;

    /// Maximum value for a `u3` (7).
    const MAX: usize = 0b111;

    /// Create a new `u3` from the lowest three bits of a `usize`, masking off the rest.
    const fn new(n: usize) -> Self {
        match n & Self::MAX {
            0 => Self::V0,
            1 => Self::V1,
            2 => Self::V2,
            3 => Self::V3,
            4 => Self::V4,
            5 => Self::V5,
            6 => Self::V6,
            7 => Self::V7,
            _ => unreachable!(),
        }
    }

    /// Convert this `u3` to a `usize`
    const fn value(self) -> usize {
        self as usize
    }

    /// Is this `u3` equal to zero?
    const fn is_zero(self) -> bool {
        self.value() == 0
    }

    /// Compute a 1-bit byte-width mask to select the bit identified by this `u3`.
    const fn bitmask(self) -> u8 {
        1u8 << (Self::MAX - self.value())
    }
}

impl Debug for u3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::{TaggedLen, u3};

    #[cfg(all(any(unix, windows), not(miri)))]
    use {core::ops::Range, proptest::prelude::*};

    #[test]
    fn bit_len() {
        assert_eq!(TaggedLen::new(0, 0, 0).bit_len(), 0);
        assert_eq!(TaggedLen::new(1, 0, 1).bit_len(), 1);
        assert_eq!(TaggedLen::new(1, 0, 0).slice(1..7).unwrap().0.bit_len(), 6);
        assert_eq!(TaggedLen::new(1, 0, 0).bit_len(), 8);
        assert_eq!(TaggedLen::new(2, 0, 0).bit_len(), 16);
    }

    #[test]
    fn decode() {
        assert_eq!(TaggedLen::decode(0), TaggedLen::new(0, 0, 0));
        assert_eq!(TaggedLen::decode(0b1000001), TaggedLen::new(1, 0, 1));
        assert_eq!(TaggedLen::decode(0b1000000), TaggedLen::new(1, 0, 0));
        assert_eq!(TaggedLen::decode(0b1001000), TaggedLen::new(1, 1, 0));
        assert_eq!(TaggedLen::decode(0b10001011111), TaggedLen::new(17, 3, 7));
    }

    #[test]
    fn encode() {
        assert_eq!(TaggedLen::new(0, 0, 0).encode(), 0);
        assert_eq!(TaggedLen::new(1, 0, 0).encode(), 0b1000000);
        assert_eq!(TaggedLen::new(1, 0, 1).encode(), 0b1000001);
        assert_eq!(TaggedLen::new(1, 1, 0).encode(), 0b1001000);
        assert_eq!(TaggedLen::new(17, 3, 7).encode(), 0b10001011111);
    }

    #[test]
    fn offset_and_mask() {
        let len = TaggedLen::new(1, 0, 0);
        assert_eq!(len.offset_and_mask(7).unwrap(), (0, 1));
        assert!(len.offset_and_mask(8).is_err());
        assert!(len.offset_and_mask(9).is_err());
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges, reason = "inputs for testing")]
    fn slice() {
        // len: 0
        let len0 = TaggedLen::new(0, 0, 0);
        assert_eq!(len0.slice(0..0).unwrap().0, len0);
        assert!(len0.slice(0..1).is_err());

        let (tagged_len, byte_offset) = len0.slice(0..0).unwrap();
        assert_eq!(tagged_len.byte_len, 0);
        assert_eq!(tagged_len.head_offset, u3::V0);
        assert_eq!(tagged_len.tail_offset, u3::V0);
        assert_eq!(byte_offset, 0);

        // len: 1
        let len1 = TaggedLen::new(1, 0, 0);
        assert_eq!(len1.slice(0..1).unwrap().0.bit_len(), 1);
        assert_eq!(len1.slice(0..8).unwrap().0, len1);
        assert!(len1.slice(1..0).is_err());
        assert!(len1.slice(0..9).is_err());

        assert_eq!(len1.slice(0..0).unwrap().0.byte_len, 0);
        assert_eq!(len1.slice(0..7).unwrap().0.byte_len, 1);
        assert_eq!(len1.slice(1..7).unwrap().0.byte_len, 1);

        let (tagged_len, byte_offset) = len1.slice(1..8).unwrap();
        assert_eq!(tagged_len.byte_len, 1);
        assert_eq!(tagged_len.head_offset, u3::V1);
        assert_eq!(tagged_len.tail_offset, u3::V0);
        assert_eq!(byte_offset, 0);

        // len: 1 (with sub-bit positioning)
        let len1_subbits = TaggedLen::new(1, 1, 6);
        assert!(len1_subbits.slice(1..6).is_err());

        let (tagged_len, byte_offset) = len1_subbits.slice(1..5).unwrap();
        assert_eq!(tagged_len.byte_len, 1);
        assert_eq!(tagged_len.head_offset, u3::V2);
        assert_eq!(tagged_len.tail_offset, u3::V6);
        assert_eq!(byte_offset, 0);

        // len: 2
        let len2 = TaggedLen::new(2, 0, 0);
        assert_eq!(len2.slice(0..16).unwrap().0, len2);
        assert!(len2.slice(0..17).is_err());

        let (tagged_len, byte_offset) = len2.slice(3..14).unwrap();
        assert_eq!(tagged_len.byte_len(), 2);
        assert_eq!(tagged_len.bit_len(), 11);
        assert_eq!(tagged_len.head_offset, u3::V3);
        assert_eq!(tagged_len.tail_offset, u3::V6);
        assert_eq!(byte_offset, 0);

        let (tagged_len, byte_offset) = len2.slice(9..15).unwrap();
        assert_eq!(tagged_len.byte_len, 1);
        assert_eq!(tagged_len.head_offset, u3::V1);
        assert_eq!(tagged_len.tail_offset, u3::V7);
        assert_eq!(byte_offset, 1);
    }

    #[cfg(all(any(unix, windows), not(miri)))]
    prop_compose! {
        fn tagged_len()(len in any::<usize>(), offsets in any::<[u8; 2]>()) -> TaggedLen {
            // Reserve 6-bits of space so we don't overflow
            let len_mask = !(0b111111 << (usize::BITS as usize - 6));
            let head_offset = usize::from(offsets[0]) & u3::MAX;
            let tail_offset = usize::from(offsets[1]) & u3::MAX;
            TaggedLen::new(len & len_mask, head_offset, tail_offset)
        }
    }

    #[cfg(all(any(unix, windows), not(miri)))]
    proptest! {
        #[test]
        fn encode_decode_roundtrip(tagged_len in tagged_len()) {
            let encoded = tagged_len.encode();
            prop_assert_eq!(TaggedLen::decode(encoded), tagged_len);
        }

        #[test]
        fn slice_proptests(tagged_len in tagged_len(), range in any::<Range<usize>>()) {
            if range.start <= range.end && range.end < tagged_len.bit_len() {
                let expected_len = range.end - range.start;
                let (sliced, _offset) = tagged_len.slice(range).unwrap();
                prop_assert_eq!(sliced.bit_len(), expected_len);
            } else {
                prop_assert!(tagged_len.slice(range).is_err());
            }
        }
    }
}
