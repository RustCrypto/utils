//! ASN.1 `BIT STRING` support.

use crate::{
    Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged,
};
use core::convert::TryFrom;

/// ASN.1 `BIT STRING` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BitString<'a> {
    /// Inner value
    inner: ByteSlice<'a>,
}

impl<'a> BitString<'a> {
    /// Create a new ASN.1 `BIT STRING` from a byte slice.
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        ByteSlice::new(slice)
            .map(|inner| Self { inner })
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

    /// Get the ASN.1 DER [`Header`] for this [`BitString`] value
    fn header(self) -> Result<Header> {
        Ok(Header {
            tag: Tag::BitString,
            length: (self.inner.len() + 1u16)?,
        })
    }
}

impl AsRef<[u8]> for BitString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&BitString<'a>> for BitString<'a> {
    fn from(value: &BitString<'a>) -> BitString<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for BitString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BitString<'a>> {
        any.tag().assert_eq(Tag::BitString)?;

        if let Some(bs) = any.as_bytes().get(1..) {
            // The first octet of a BIT STRING encodes the number of unused bits.
            // We presently constrain this to 0.
            if any.as_bytes()[0] == 0 {
                return ByteSlice::new(bs)
                    .map(|inner| Self { inner })
                    .map_err(|_| ErrorKind::Length { tag: Self::TAG }.into());
            }
        }

        Err(ErrorKind::Length { tag: Self::TAG }.into())
    }
}

impl<'a> From<BitString<'a>> for Any<'a> {
    fn from(bit_string: BitString<'a>) -> Any<'a> {
        Any {
            tag: Tag::BitString,
            value: bit_string.inner,
        }
    }
}

impl<'a> From<BitString<'a>> for &'a [u8] {
    fn from(bit_string: BitString<'a>) -> &'a [u8] {
        bit_string.as_bytes()
    }
}

impl<'a> Encodable for BitString<'a> {
    fn encoded_len(&self) -> Result<Length> {
        self.header()?.encoded_len()? + 1u16 + self.inner.len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.header()?.encode(encoder)?;
        encoder.byte(0)?;
        encoder.bytes(self.as_bytes())
    }
}

impl<'a> Tagged for BitString<'a> {
    const TAG: Tag = Tag::BitString;
}

#[cfg(test)]
mod tests {
    use super::{Any, BitString, ErrorKind, Result, Tag};
    use core::convert::TryInto;

    /// Parse a `BitString` from an ASN.1 `Any` value to test decoding behaviors.
    fn parse_bitstring_from_any(bytes: &[u8]) -> Result<BitString<'_>> {
        Any::new(Tag::BitString, bytes)?.try_into()
    }

    #[test]
    fn reject_non_prefixed_bitstring() {
        let err = parse_bitstring_from_any(&[]).err().unwrap();
        assert_eq!(
            err.kind(),
            ErrorKind::Length {
                tag: Tag::BitString
            }
        );
    }

    #[test]
    fn reject_non_zero_prefix() {
        let err = parse_bitstring_from_any(&[1, 1, 2, 3]).err().unwrap();
        assert_eq!(
            err.kind(),
            ErrorKind::Length {
                tag: Tag::BitString
            }
        );
    }

    #[test]
    fn decode_empty_bitstring() {
        let bs = parse_bitstring_from_any(&[0]).unwrap();
        assert_eq!(bs.as_ref(), &[]);
    }

    #[test]
    fn decode_non_empty_bitstring() {
        let bs = parse_bitstring_from_any(&[0, 1, 2, 3]).unwrap();
        assert_eq!(bs.as_ref(), &[1, 2, 3]);
    }
}
