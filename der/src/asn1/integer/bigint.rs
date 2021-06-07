//! "Big" ASN.1 `INTEGER` types.

use super::uint;
use crate::{
    asn1::Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged,
};
use core::convert::TryFrom;

#[cfg(feature = "bigint")]
use {
    core::convert::TryInto,
    crypto_bigint::{generic_array::GenericArray, ArrayEncoding, UInt},
};

/// "Big" unsigned ASN.1 `INTEGER` type.
///
/// Provides direct access to the underlying big endian bytes which comprise an
/// unsigned integer value.
///
/// Intended for use cases like very large integers that are used in
/// cryptographic applications (e.g. keys, signatures).
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct UIntBytes<'a> {
    /// Inner value
    inner: ByteSlice<'a>,
}

impl<'a> UIntBytes<'a> {
    /// Create a new [`UIntBytes`] from a byte slice.
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        let inner = ByteSlice::new(uint::strip_leading_zeroes(bytes))
            .map_err(|_| ErrorKind::Length { tag: Self::TAG })?;

        Ok(Self { inner })
    }

    /// Borrow the inner byte slice which contains the least significant bytes
    /// of a big endian integer value with all leading zeros stripped.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    /// Get the length of this [`UIntBytes`] in bytes.
    pub fn len(&self) -> Length {
        self.inner.len()
    }

    /// Is the inner byte slice empty?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the length of the inner integer value when encoded.
    fn inner_len(self) -> Result<Length> {
        uint::encoded_len(self.inner.as_bytes())
    }
}

impl<'a> From<&UIntBytes<'a>> for UIntBytes<'a> {
    fn from(value: &UIntBytes<'a>) -> UIntBytes<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for UIntBytes<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<UIntBytes<'a>> {
        Self::new(uint::decode_slice(any)?)
    }
}

impl<'a> Encodable for UIntBytes<'a> {
    fn encoded_len(&self) -> Result<Length> {
        self.inner_len()?.for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Header::new(Self::TAG, self.inner_len()?)?.encode(encoder)?;

        // Add leading `0x00` byte if required
        if self.inner_len()? > self.len() {
            encoder.byte(0)?;
        }

        encoder.bytes(self.as_bytes())
    }
}

impl<'a> Tagged for UIntBytes<'a> {
    const TAG: Tag = Tag::Integer;
}

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
impl<'a, const LIMBS: usize> TryFrom<Any<'a>> for UInt<LIMBS>
where
    UInt<LIMBS>: ArrayEncoding,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<UInt<LIMBS>> {
        UIntBytes::try_from(any)?.try_into()
    }
}

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
impl<'a, const LIMBS: usize> TryFrom<UIntBytes<'a>> for UInt<LIMBS>
where
    UInt<LIMBS>: ArrayEncoding,
{
    type Error = Error;

    fn try_from(bytes: UIntBytes<'a>) -> Result<UInt<LIMBS>> {
        let mut array = GenericArray::default();
        let offset = array.len().saturating_sub(bytes.len().try_into()?);
        array[offset..].copy_from_slice(bytes.as_bytes());
        Ok(UInt::from_be_byte_array(array))
    }
}

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
impl<'a, const LIMBS: usize> Encodable for UInt<LIMBS>
where
    UInt<LIMBS>: ArrayEncoding,
{
    fn encoded_len(&self) -> Result<Length> {
        // TODO(tarcieri): more efficient length calculation
        let array = self.to_be_byte_array();
        UIntBytes::new(&array)?.encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        let array = self.to_be_byte_array();
        UIntBytes::new(&array)?.encode(encoder)
    }
}

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
impl<'a, const LIMBS: usize> Tagged for UInt<LIMBS>
where
    UInt<LIMBS>: ArrayEncoding,
{
    const TAG: Tag = Tag::Integer;
}

#[cfg(test)]
mod tests {
    use super::UIntBytes;
    use crate::{
        asn1::{integer::tests::*, Any},
        Decodable, Encodable, Encoder, ErrorKind, Tag,
    };
    use core::convert::TryFrom;

    #[test]
    fn decode_uint_bytes() {
        assert_eq!(&[0], UIntBytes::from_der(I0_BYTES).unwrap().as_bytes());
        assert_eq!(&[127], UIntBytes::from_der(I127_BYTES).unwrap().as_bytes());
        assert_eq!(&[128], UIntBytes::from_der(I128_BYTES).unwrap().as_bytes());
        assert_eq!(&[255], UIntBytes::from_der(I255_BYTES).unwrap().as_bytes());

        assert_eq!(
            &[0x01, 0x00],
            UIntBytes::from_der(I256_BYTES).unwrap().as_bytes()
        );

        assert_eq!(
            &[0x7F, 0xFF],
            UIntBytes::from_der(I32767_BYTES).unwrap().as_bytes()
        );
    }

    #[test]
    fn encode_uint_bytes() {
        for &example in &[
            I0_BYTES,
            I127_BYTES,
            I128_BYTES,
            I255_BYTES,
            I256_BYTES,
            I32767_BYTES,
        ] {
            let uint = UIntBytes::from_der(example).unwrap();

            let mut buf = [0u8; 128];
            let mut encoder = Encoder::new(&mut buf);
            uint.encode(&mut encoder).unwrap();

            let result = encoder.finish().unwrap();
            assert_eq!(example, result);
        }
    }

    #[test]
    fn reject_oversize_without_extra_zero() {
        let err = UIntBytes::try_from(Any::new(Tag::Integer, &[0x81]).unwrap())
            .err()
            .unwrap();

        assert_eq!(err.kind(), ErrorKind::Value { tag: Tag::Integer });
    }
}
