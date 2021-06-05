//! ASN.1 `INTEGER` support.

pub(super) mod bigint;
mod int;
mod uint;

use crate::{asn1::Any, Encodable, Encoder, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

macro_rules! impl_int_encoding {
    ($($int:ty => $uint:ty),+) => {
        $(
            impl TryFrom<Any<'_>> for $int {
                type Error = Error;

                fn try_from(any: Any<'_>) -> Result<Self> {
                    let result = if is_highest_bit_set(any.as_bytes()) {
                        <$uint>::from_be_bytes(int::decode_array(any)?) as $int
                    } else {
                        Self::from_be_bytes(uint::decode_array(any)?)
                    };

                    // Ensure we compute the same encoded length as the original any value
                    if any.encoded_len()? != result.encoded_len()? {
                        return Err(Self::TAG.non_canonical_error());
                    }

                    Ok(result)
                }
            }

            impl Encodable for $int {
                fn encoded_len(&self) -> Result<Length> {
                    if *self < 0 {
                        int::encoded_len(&(*self as $uint).to_be_bytes())?.for_tlv()
                    } else {
                        uint::encoded_len(&self.to_be_bytes())?.for_tlv()
                    }
                }

                fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
                    if *self < 0 {
                        int::encode(encoder, &(*self as $uint).to_be_bytes())
                    } else {
                        uint::encode(encoder, &self.to_be_bytes())
                    }
                }
            }

            impl Tagged for $int {
                const TAG: Tag = Tag::Integer;
            }
        )+
    };
}

macro_rules! impl_uint_encoding {
    ($($uint:ty),+) => {
        $(
            impl TryFrom<Any<'_>> for $uint {
                type Error = Error;

                fn try_from(any: Any<'_>) -> Result<Self> {
                    let result = Self::from_be_bytes(uint::decode_array(any)?);

                    // Ensure we compute the same encoded length as the original any value
                    if any.encoded_len()? != result.encoded_len()? {
                        return Err(Self::TAG.non_canonical_error());
                    }

                    Ok(result)
                }
            }

            impl Encodable for $uint {
                fn encoded_len(&self) -> Result<Length> {
                    uint::encoded_len(&self.to_be_bytes())?.for_tlv()
                }

                fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
                    uint::encode(encoder, &self.to_be_bytes())
                }
            }

            impl Tagged for $uint {
                const TAG: Tag = Tag::Integer;
            }
        )+
    };
}

impl_int_encoding!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128);
impl_uint_encoding!(u8, u16, u32, u64, u128);

/// Is the highest bit of the first byte in the slice 1? (if present)
#[inline]
fn is_highest_bit_set(bytes: &[u8]) -> bool {
    bytes
        .get(0)
        .map(|byte| byte & 0b10000000 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{Decodable, Encodable};

    // Vectors from Section 5.7 of:
    // https://luca.ntop.org/Teaching/Appunti/asn1.html
    pub(crate) const I0_BYTES: &[u8] = &[0x02, 0x01, 0x00];
    pub(crate) const I127_BYTES: &[u8] = &[0x02, 0x01, 0x7F];
    pub(crate) const I128_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0x80];
    pub(crate) const I256_BYTES: &[u8] = &[0x02, 0x02, 0x01, 0x00];
    pub(crate) const INEG128_BYTES: &[u8] = &[0x02, 0x01, 0x80];
    pub(crate) const INEG129_BYTES: &[u8] = &[0x02, 0x02, 0xFF, 0x7F];

    // Additional vectors
    pub(crate) const I255_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0xFF];
    pub(crate) const I32767_BYTES: &[u8] = &[0x02, 0x02, 0x7F, 0xFF];
    pub(crate) const I65535_BYTES: &[u8] = &[0x02, 0x03, 0x00, 0xFF, 0xFF];
    pub(crate) const INEG32768_BYTES: &[u8] = &[0x02, 0x02, 0x80, 0x00];

    #[test]
    fn decode_i8() {
        assert_eq!(0, i8::from_der(I0_BYTES).unwrap());
        assert_eq!(127, i8::from_der(I127_BYTES).unwrap());
        assert_eq!(-128, i8::from_der(INEG128_BYTES).unwrap());
    }

    #[test]
    fn decode_i16() {
        assert_eq!(0, i16::from_der(I0_BYTES).unwrap());
        assert_eq!(127, i16::from_der(I127_BYTES).unwrap());
        assert_eq!(128, i16::from_der(I128_BYTES).unwrap());
        assert_eq!(255, i16::from_der(I255_BYTES).unwrap());
        assert_eq!(256, i16::from_der(I256_BYTES).unwrap());
        assert_eq!(32767, i16::from_der(I32767_BYTES).unwrap());
        assert_eq!(-128, i16::from_der(INEG128_BYTES).unwrap());
        assert_eq!(-129, i16::from_der(INEG129_BYTES).unwrap());
        assert_eq!(-32768, i16::from_der(INEG32768_BYTES).unwrap());
    }

    #[test]
    fn decode_u8() {
        assert_eq!(0, u8::from_der(I0_BYTES).unwrap());
        assert_eq!(127, u8::from_der(I127_BYTES).unwrap());
        assert_eq!(255, u8::from_der(I255_BYTES).unwrap());
    }

    #[test]
    fn decode_u16() {
        assert_eq!(0, u16::from_der(I0_BYTES).unwrap());
        assert_eq!(127, u16::from_der(I127_BYTES).unwrap());
        assert_eq!(255, u16::from_der(I255_BYTES).unwrap());
        assert_eq!(256, u16::from_der(I256_BYTES).unwrap());
        assert_eq!(32767, u16::from_der(I32767_BYTES).unwrap());
        assert_eq!(65535, u16::from_der(I65535_BYTES).unwrap());
    }

    #[test]
    fn encode_i8() {
        let mut buffer = [0u8; 3];

        assert_eq!(I0_BYTES, 0i8.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I127_BYTES, 127i8.encode_to_slice(&mut buffer).unwrap());

        assert_eq!(
            INEG128_BYTES,
            (-128i8).encode_to_slice(&mut buffer).unwrap()
        );
    }

    #[test]
    fn encode_i16() {
        let mut buffer = [0u8; 4];
        assert_eq!(I0_BYTES, 0i16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I127_BYTES, 127i16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I128_BYTES, 128i16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I255_BYTES, 255i16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I256_BYTES, 256i16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I32767_BYTES, 32767i16.encode_to_slice(&mut buffer).unwrap());

        assert_eq!(
            INEG128_BYTES,
            (-128i16).encode_to_slice(&mut buffer).unwrap()
        );

        assert_eq!(
            INEG129_BYTES,
            (-129i16).encode_to_slice(&mut buffer).unwrap()
        );

        assert_eq!(
            INEG32768_BYTES,
            (-32768i16).encode_to_slice(&mut buffer).unwrap()
        );
    }

    #[test]
    fn encode_u8() {
        let mut buffer = [0u8; 4];
        assert_eq!(I0_BYTES, 0u8.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I127_BYTES, 127u8.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I255_BYTES, 255u8.encode_to_slice(&mut buffer).unwrap());
    }

    #[test]
    fn encode_u16() {
        let mut buffer = [0u8; 5];
        assert_eq!(I0_BYTES, 0u16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I127_BYTES, 127u16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I128_BYTES, 128u16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I255_BYTES, 255u16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I256_BYTES, 256u16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I32767_BYTES, 32767u16.encode_to_slice(&mut buffer).unwrap());
        assert_eq!(I65535_BYTES, 65535u16.encode_to_slice(&mut buffer).unwrap());
    }

    /// Integers must be encoded with a minimum number of octets
    #[test]
    fn reject_non_canonical() {
        assert!(i8::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(i16::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(u8::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(u16::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
    }
}
