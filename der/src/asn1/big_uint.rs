//! "Big" ASN.1 `INTEGER` types.

use crate::{
    Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged,
};
use core::{convert::TryFrom, marker::PhantomData};
use typenum::Unsigned;

/// "Big" unsigned ASN.1 `INTEGER` type.
///
/// Provides direct access to the underlying bytes which comprise an unsigned
/// integer value, intended for use cases like very large integers that are
/// used for values in cryptography.
///
/// Generic over a `Size` value (e.g. [`der::consts::U64`][`typenum::U64`]),
/// indicating the size of an integer in bytes.
///
/// Currently supported sizes are 1 - 127 bytes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(docsrs, doc(cfg(feature = "big-uint")))]
pub struct BigUInt<'a, N: Size> {
    /// Inner value
    inner: ByteSlice<'a>,

    /// Integer size in bytes
    size: PhantomData<N>,
}

impl<'a, N: Size> BigUInt<'a, N> {
    /// Create a new [`BigUInt`] from a byte slice.
    ///
    /// Slice may be less than or equal to `N` bytes.
    pub fn new(mut bytes: &'a [u8]) -> Result<Self> {
        // Remove leading zeroes
        while bytes.get(0).cloned() == Some(0) {
            bytes = &bytes[1..];
        }

        if bytes.len() > N::to_usize() {
            return Err(ErrorKind::Length { tag: Self::TAG }.into());
        }

        ByteSlice::new(bytes)
            .map(|inner| Self {
                inner,
                size: PhantomData,
            })
            .map_err(|_| ErrorKind::Length { tag: Self::TAG }.into())
    }

    /// Borrow the inner byte slice.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    /// Get the length of the inner byte slice.
    pub fn len(&self) -> Length {
        self.inner.len()
    }

    /// Is the inner byte slice empty?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the length of the inner integer value when encoded.
    fn inner_len(self) -> Result<Length> {
        self.len()
            + match self.inner.as_ref().get(0).cloned() {
                Some(n) if n >= 0x80 => 1u8,
                None => 1u8,
                _ => 0u8,
            }
    }

    /// Get the ASN.1 DER [`Header`] for this [`BigUint`] value
    fn header(self) -> Result<Header> {
        Ok(Header {
            tag: Tag::Integer,
            length: self.inner_len()?,
        })
    }
}

impl<'a, N: Size> From<&BigUInt<'a, N>> for BigUInt<'a, N> {
    fn from(value: &BigUInt<'a, N>) -> BigUInt<'a, N> {
        *value
    }
}

impl<'a, N: Size> TryFrom<Any<'a>> for BigUInt<'a, N> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BigUInt<'a, N>> {
        any.tag().assert_eq(Tag::Integer)?;
        let mut bytes = any.as_bytes();

        // Disallow a leading byte which would overflow a signed
        // ASN.1 integer (since this is a "uint" type)
        if let Some(byte) = bytes.get(0).cloned() {
            if byte > 0x80 {
                return Err(ErrorKind::Value { tag: Self::TAG }.into());
            }
        }

        // The `INTEGER` type always encodes a signed value, so for unsigned
        // values the leading `0x00` byte may need to be removed.
        // TODO(tarcieri): validate leading 0 byte was required
        if bytes.len() > N::to_usize() {
            if bytes.len() != N::to_usize().checked_add(1).unwrap() {
                return Err(ErrorKind::Length { tag: Self::TAG }.into());
            }

            if bytes.get(0).cloned() != Some(0) {
                return Err(ErrorKind::Value { tag: Self::TAG }.into());
            }

            bytes = &bytes[1..];
        }

        Self::new(bytes)
    }
}

impl<'a, N: Size> Encodable for BigUInt<'a, N> {
    fn encoded_len(&self) -> Result<Length> {
        self.header()?.encoded_len()? + self.inner_len()?
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.header()?.encode(encoder)?;

        // Add leading `0x00` byte if required
        if self.inner_len()? > self.len() {
            encoder.byte(0)?;
        }

        encoder.bytes(self.as_bytes())
    }
}

impl<'a, N: Size> Tagged for BigUInt<'a, N> {
    const TAG: Tag = Tag::Integer;
}

/// Marker trait for allowed integer sizes
pub trait Size: Unsigned {}

macro_rules! impl_size {
    ($($int:ident),+) => {
        $(impl Size for typenum::consts::$int {})+
    };
}

// Sizes supported by the current implementation (1 - 127 bytes)
// TODO(tarcieri): support larger integer sizes
impl_size!(
    U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U13, U14, U15, U16, U17, U18, U19, U20, U21,
    U22, U23, U24, U25, U26, U27, U28, U29, U30, U31, U32, U33, U34, U35, U36, U37, U38, U39, U40,
    U41, U42, U43, U44, U45, U46, U47, U48, U49, U50, U51, U52, U53, U54, U55, U56, U57, U58, U59,
    U60, U61, U62, U63, U64, U65, U66, U67, U68, U69, U70, U71, U72, U73, U74, U75, U76, U77, U78,
    U79, U80, U81, U82, U83, U84, U85, U86, U87, U88, U89, U90, U91, U92, U93, U94, U95, U96, U97,
    U98, U99, U100, U101, U102, U103, U104, U105, U106, U107, U108, U109, U110, U111, U112, U113,
    U114, U115, U116, U117, U118, U119, U120, U121, U122, U123, U124, U125, U126, U127
);

#[cfg(test)]
mod tests {
    use super::BigUInt;
    use crate::{Any, ErrorKind, Result, Tag};
    use core::convert::TryInto;

    type BigU1<'a> = BigUInt<'a, typenum::U1>;

    /// Parse a `BitU1` from an ASN.1 `Any` value to test decoding behaviors.
    fn parse_bigu1_from_any(bytes: &[u8]) -> Result<BigU1<'_>> {
        Any::new(Tag::Integer, bytes)?.try_into()
    }

    #[test]
    fn decode_empty() {
        let x = parse_bigu1_from_any(&[]).unwrap();
        assert_eq!(x.as_bytes(), &[]);
    }

    #[test]
    fn decode_zero() {
        let x = parse_bigu1_from_any(&[0]).unwrap();
        assert_eq!(x.as_bytes(), &[]);
    }

    #[test]
    fn decode_leading_extra_zero() {
        let x = parse_bigu1_from_any(&[0x00, 0x81]).unwrap();
        assert_eq!(x.as_bytes(), &[0x81]);
    }

    #[test]
    fn reject_oversize_without_extra_zero() {
        let err = parse_bigu1_from_any(&[0x81]).err().unwrap();
        assert_eq!(err.kind(), ErrorKind::Value { tag: Tag::Integer });
    }
}
