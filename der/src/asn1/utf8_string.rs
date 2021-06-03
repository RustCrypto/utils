//! ASN.1 `UTF8String` support.

use crate::{
    asn1::Any, str_slice::StrSlice, Encodable, Encoder, Error, Length, Result, Tag, Tagged,
};
use core::{convert::TryFrom, fmt, str};

#[cfg(feature = "alloc")]
use alloc::{borrow::ToOwned, string::String};

/// ASN.1 `UTF8String` type.
///
/// Supports the full UTF-8 encoding.
///
/// Note that the [`Decodable`][`crate::Decodable`] and [`Encodable`] traits
/// are impl'd for Rust's [`str`][`prim@str`] primitive, which decodes/encodes
/// as a [`Utf8String`].
///
/// You are free to use [`str`][`prim@str`] instead of this type, however it's
/// still provided for explicitness in cases where it might be ambiguous with
/// other ASN.1 string encodings such as
/// [`PrintableString`][`crate::asn1::PrintableString`].
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Utf8String<'a> {
    /// Inner value
    inner: StrSlice<'a>,
}

impl<'a> Utf8String<'a> {
    /// Create a new ASN.1 `UTF8String`.
    pub fn new<T>(input: &'a T) -> Result<Self>
    where
        T: AsRef<[u8]> + ?Sized,
    {
        StrSlice::from_bytes(input.as_ref()).map(|inner| Self { inner })
    }

    /// Borrow the string as a `str`.
    pub fn as_str(&self) -> &'a str {
        self.inner.as_str()
    }

    /// Borrow the string as bytes.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    /// Get the length of the inner byte slice.
    pub fn len(&self) -> Length {
        self.inner.len()
    }

    /// Is the inner string empty?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl AsRef<str> for Utf8String<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for Utf8String<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&Utf8String<'a>> for Utf8String<'a> {
    fn from(value: &Utf8String<'a>) -> Utf8String<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for Utf8String<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Utf8String<'a>> {
        any.tag().assert_eq(Tag::Utf8String)?;
        Self::new(any.as_bytes())
    }
}

impl<'a> From<Utf8String<'a>> for Any<'a> {
    fn from(printable_string: Utf8String<'a>) -> Any<'a> {
        Any::from_tag_and_value(Tag::Utf8String, printable_string.inner.into())
    }
}

impl<'a> From<Utf8String<'a>> for &'a [u8] {
    fn from(utf8_string: Utf8String<'a>) -> &'a [u8] {
        utf8_string.as_bytes()
    }
}

impl<'a> Encodable for Utf8String<'a> {
    fn encoded_len(&self) -> Result<Length> {
        Any::from(*self).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(*self).encode(encoder)
    }
}

impl<'a> Tagged for Utf8String<'a> {
    const TAG: Tag = Tag::Utf8String;
}

impl<'a> fmt::Display for Utf8String<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> fmt::Debug for Utf8String<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Utf8String({:?})", self.as_str())
    }
}

impl<'a> TryFrom<Any<'a>> for &'a str {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<&'a str> {
        Utf8String::try_from(any).map(|s| s.as_str())
    }
}

impl Encodable for str {
    fn encoded_len(&self) -> Result<Length> {
        Utf8String::new(self)?.encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Utf8String::new(self)?.encode(encoder)
    }
}

impl Tagged for str {
    const TAG: Tag = Tag::Utf8String;
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> TryFrom<Any<'a>> for String {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<String> {
        Utf8String::try_from(any).map(|s| s.as_str().to_owned())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Encodable for String {
    fn encoded_len(&self) -> Result<Length> {
        Utf8String::new(self)?.encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Utf8String::new(self)?.encode(encoder)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Tagged for String {
    const TAG: Tag = Tag::Utf8String;
}

#[cfg(test)]
mod tests {
    use super::Utf8String;
    use crate::Decodable;

    #[test]
    fn parse_ascii_bytes() {
        let example_bytes = &[
            0x0c, 0x0b, 0x54, 0x65, 0x73, 0x74, 0x20, 0x55, 0x73, 0x65, 0x72, 0x20, 0x31,
        ];

        let utf8_string = Utf8String::from_der(example_bytes).unwrap();
        assert_eq!(utf8_string.as_str(), "Test User 1");
    }

    #[test]
    fn parse_utf8_bytes() {
        let example_bytes = &[0x0c, 0x06, 0x48, 0x65, 0x6c, 0x6c, 0xc3, 0xb3];
        let utf8_string = Utf8String::from_der(example_bytes).unwrap();
        assert_eq!(utf8_string.as_str(), "Helló");
    }
}
