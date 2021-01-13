//! ASN.1 `INTEGER` support.

// TODO(tarcieri): add support for `i32`, `u8`, `u16`, `u32`, etc.

use crate::{Any, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

impl TryFrom<Any<'_>> for i8 {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<i8> {
        let tag = any.tag().assert_eq(Tag::Integer)?;

        match *any.as_bytes() {
            [x] => Ok(x as i8),
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

impl TryFrom<Any<'_>> for i16 {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<i16> {
        let tag = any.tag().assert_eq(Tag::Integer)?;

        match *any.as_bytes() {
            [_] => i8::try_from(any).map(|x| x as i16),
            [hi, lo] => {
                if hi == 0 && lo < 0x80 {
                    // Non-canonical integer: unnecessary leading 0
                    Err(ErrorKind::Noncanonical.into())
                } else {
                    Ok(i16::from_be_bytes([hi, lo]))
                }
            }
            _ => Err(ErrorKind::Length { tag }.into()),
        }
    }
}

impl Encodable for i16 {
    fn encoded_len(&self) -> Result<Length> {
        let inner_len = if i8::try_from(*self).is_ok() {
            1u8
        } else {
            2u8
        };

        Header {
            tag: Tag::Integer,
            length: inner_len.into(),
        }
        .encoded_len()?
            + inner_len
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        // If value is in the `i8` range, encode it as one
        if let Ok(x) = i8::try_from(*self) {
            return x.encode(encoder);
        }

        Header {
            tag: Tag::Integer,
            length: 2u8.into(),
        }
        .encode(encoder)?;

        encoder.bytes(&self.to_be_bytes())
    }
}

impl Tagged for i16 {
    const TAG: Tag = Tag::Integer;
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

    // Additional vectors (TODO: double check these or find better ones)
    pub(crate) const I255_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0xFF];
    pub(crate) const I32767_BYTES: &[u8] = &[0x02, 0x02, 0x7F, 0xFF];
    pub(crate) const INEG32768_BYTES: &[u8] = &[0x02, 0x02, 0x80, 0x00];

    #[test]
    fn decode_i8() {
        assert_eq!(0, i8::from_bytes(I0_BYTES).unwrap());
        assert_eq!(127, i8::from_bytes(I127_BYTES).unwrap());
        assert_eq!(-128, i8::from_bytes(INEG128_BYTES).unwrap());
    }

    #[test]
    fn decode_i16() {
        assert_eq!(0, i16::from_bytes(I0_BYTES).unwrap());
        assert_eq!(127, i16::from_bytes(I127_BYTES).unwrap());
        assert_eq!(128, i16::from_bytes(I128_BYTES).unwrap());
        assert_eq!(255, i16::from_bytes(I255_BYTES).unwrap());
        assert_eq!(256, i16::from_bytes(I256_BYTES).unwrap());
        assert_eq!(32767, i16::from_bytes(I32767_BYTES).unwrap());
        assert_eq!(-128, i16::from_bytes(INEG128_BYTES).unwrap());
        assert_eq!(-129, i16::from_bytes(INEG129_BYTES).unwrap());
        assert_eq!(-32768, i16::from_bytes(INEG32768_BYTES).unwrap());
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

    /// Integers must be encoded with a minimum number of octets
    #[test]
    fn reject_non_canonical() {
        assert!(i8::from_bytes(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(i16::from_bytes(&[0x02, 0x02, 0x00, 0x00]).is_err());
    }
}
