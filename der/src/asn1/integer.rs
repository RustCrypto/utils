//! ASN.1 `INTEGER` support.

use crate::{
    Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged,
};
use core::convert::TryFrom;

/// ASN.1 `INTEGER` type.
///
/// # Limits
///
/// Presently constrained to `i8`.
///
/// This is not a deliberate decision: the goal of this library is to
/// eventually support other integer types, but they have not yet been
/// implemented.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Integer(i8);

impl Integer {
    /// Get the ASN.1 DER [`Header`] for this [`Integer`] value
    pub(crate) fn header(self) -> Header {
        Header {
            tag: Tag::Integer,
            length: 1u8.into(), // TODO(tarcieri): larger integers
        }
    }
}

impl From<i8> for Integer {
    fn from(x: i8) -> Integer {
        Integer(x)
    }
}

impl From<Integer> for i8 {
    fn from(x: Integer) -> i8 {
        x.0
    }
}

impl TryFrom<Any<'_>> for Integer {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Integer> {
        let tag = any.tag().assert_eq(Tag::Integer)?;

        match any.as_bytes() {
            [x] => Ok(Integer(*x as i8)),
            _ => Err(ErrorKind::Length { tag }.into()),
        }
    }
}

impl Encodable for Integer {
    fn encoded_len(&self) -> Result<Length> {
        self.header().encoded_len()? + 1u8
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.header().encode(encoder)?;
        encoder.byte(self.0 as u8)
    }
}

impl Tagged for Integer {
    const TAG: Tag = Tag::Integer;
}

/// Raw ASN.1 `INTEGER` type.
///
/// Provides direct access to the underlying DER-encoded bytes which comprise
/// an integer value.
///
/// This is an alternative API for `INTEGER` values which can't be represented
/// by this crate's [`Integer`] type, intended for use cases like very large
/// integers that are used for cryptographic keys. It can be used in order to
/// convert them to the big integer representation of your choice.
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
