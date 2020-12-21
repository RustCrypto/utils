//! ASN.1 `INTEGER` support.

use crate::{
    Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged,
};
use core::convert::TryFrom;

impl TryFrom<Any<'_>> for i8 {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<i8> {
        let tag = any.tag().assert_eq(Tag::Integer)?;

        match any.as_bytes() {
            [x] => Ok(*x as i8),
            _ => Err(ErrorKind::Length { tag }.into()),
        }
    }
}

impl Encodable for i8 {
    fn encoded_len(&self) -> Result<Length> {
        Header {
            tag: Tag::Integer,
            length: 1u8.into(),
        }
        .encoded_len()?
            + 1u8
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Header {
            tag: Tag::Integer,
            length: 1u8.into(),
        }
        .encode(encoder)?;
        encoder.byte(*self as u8)
    }
}

impl Tagged for i8 {
    const TAG: Tag = Tag::Integer;
}

/// Raw ASN.1 `INTEGER` type.
///
/// Provides direct access to the underlying DER-encoded bytes which comprise
/// an integer value, intended for use cases like very large integers that are
/// used for cryptographic keys. It can be used in order to convert them to the
/// big integer representation of your choice.
///
/// Note that the [`Decodable`] and [`Encodable`] traits are implemented for
/// Rust's integer types ([`i8`] only for now) if you'd like to work directly
/// with an integer value.
///
/// # ⚠️ Important Usage Notes ⚠️
///
/// This type does *NOT* ensure the value is canonically encoded according to
/// DER's rules. If it's important for your use case that the message is valid
/// ASN.1 DER, you *MUST* validate the value is canonically encoded yourself.
// TODO(tarcieri): implement generic validation rules for arbitrary-sized integers
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RawInteger<'a> {
    /// Inner value
    inner: ByteSlice<'a>,
}

impl<'a> RawInteger<'a> {
    /// Create a new [`RawInteger`] from a slice.
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        ByteSlice::new(slice)
            .map(|inner| Self { inner })
            .map_err(|_| ErrorKind::Length { tag: Self::TAG }.into())
    }

    /// Borrow the inner byte slice.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }
}

impl AsRef<[u8]> for RawInteger<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&RawInteger<'a>> for RawInteger<'a> {
    fn from(value: &RawInteger<'a>) -> RawInteger<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for RawInteger<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<RawInteger<'a>> {
        any.tag().assert_eq(Tag::Integer)?;
        Self::new(any.as_bytes())
    }
}

impl<'a> From<RawInteger<'a>> for Any<'a> {
    fn from(integer: RawInteger<'a>) -> Any<'a> {
        Any {
            tag: Tag::Integer,
            value: integer.inner,
        }
    }
}

impl<'a> Encodable for RawInteger<'a> {
    fn encoded_len(&self) -> Result<Length> {
        Any::from(*self).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(*self).encode(encoder)
    }
}

impl<'a> Tagged for RawInteger<'a> {
    const TAG: Tag = Tag::Integer;
}

#[cfg(test)]
mod tests {
    use crate::{Decodable, Encodable};

    // TODO(tarcieri): larger integer types
    #[test]
    fn decode_i8() {
        // 0
        let int = i8::from_bytes(&[0x02, 0x01, 0x00]).unwrap();
        assert_eq!(i8::from(int), 0);

        // 127
        let int = i8::from_bytes(&[0x02, 0x01, 0x7F]).unwrap();
        assert_eq!(i8::from(int), 127);

        // -128
        let int = i8::from_bytes(&[0x02, 0x01, 0x80]).unwrap();
        assert_eq!(i8::from(int), -128);
    }

    #[test]
    fn encode_i8() {
        let mut buffer = [0u8; 3];

        // 0
        assert_eq!(
            &[0x02, 0x01, 0x00],
            i8::from(0i8).encode_to_slice(&mut buffer).unwrap()
        );

        // 127
        assert_eq!(
            &[0x02, 0x01, 0x7F],
            i8::from(127i8).encode_to_slice(&mut buffer).unwrap()
        );

        // -128
        assert_eq!(
            &[0x02, 0x01, 0x80],
            i8::from(-128i8).encode_to_slice(&mut buffer).unwrap()
        );
    }

    /// Integers must be encoded with a minimum number of octets
    #[test]
    fn reject_non_canonical() {
        assert!(i8::from_bytes(&[0x02, 0x02, 0x00, 0x00]).is_err());
    }
}
