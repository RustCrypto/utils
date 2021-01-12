//! ASN.1 `INTEGER` support.

// TODO(tarcieri): add support for `i32`, `u8`, `u16`, `u32`, etc.

use crate::{Any, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged};
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

impl TryFrom<Any<'_>> for i16 {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<i16> {
        let tag = any.tag().assert_eq(Tag::Integer)?;

        match any.as_bytes() {
            [_] => i8::try_from(any).map(|x| x as i16),
            [hi, lo] => {
                if *hi == 0 {
                    // Non-canonical integer: unnecessary leading 0
                    Err(ErrorKind::Noncanonical.into())
                } else {
                    Ok(i16::from_be_bytes([*hi, *lo]))
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
mod tests {
    use crate::{Decodable, Encodable};

    #[test]
    fn decode_i8() {
        // 0
        let int = i8::from_bytes(&[0x02, 0x01, 0x00]).unwrap();
        assert_eq!(int, 0i8);

        // 127
        let int = i8::from_bytes(&[0x02, 0x01, 0x7F]).unwrap();
        assert_eq!(int, 127i8);

        // -128
        let int = i8::from_bytes(&[0x02, 0x01, 0x80]).unwrap();
        assert_eq!(int, -128i8);
    }

    #[test]
    fn decode_i16() {
        // 0
        let int = i16::from_bytes(&[0x02, 0x01, 0x00]).unwrap();
        assert_eq!(int, 0i16);

        // 127
        let int = i16::from_bytes(&[0x02, 0x01, 0x7F]).unwrap();
        assert_eq!(int, 127i16);

        // -128
        let int = i16::from_bytes(&[0x02, 0x01, 0x80]).unwrap();
        assert_eq!(int, -128i16);

        // 32767
        let int = i16::from_bytes(&[0x02, 0x02, 0x7F, 0xFF]).unwrap();
        assert_eq!(int, 32767i16);

        // -32768
        let int = i16::from_bytes(&[0x02, 0x02, 0x80, 0x00]).unwrap();
        assert_eq!(int, -32768i16);
    }

    #[test]
    fn encode_i8() {
        let mut buffer = [0u8; 3];

        // 0
        assert_eq!(
            &[0x02, 0x01, 0x00],
            0i8.encode_to_slice(&mut buffer).unwrap()
        );

        // 127
        assert_eq!(
            &[0x02, 0x01, 0x7F],
            127i8.encode_to_slice(&mut buffer).unwrap()
        );

        // -128
        assert_eq!(
            &[0x02, 0x01, 0x80],
            (-128i8).encode_to_slice(&mut buffer).unwrap()
        );
    }

    #[test]
    fn encode_i16() {
        let mut buffer = [0u8; 4];

        // 0
        assert_eq!(
            &[0x02, 0x01, 0x00],
            0i16.encode_to_slice(&mut buffer).unwrap()
        );

        // 127
        assert_eq!(
            &[0x02, 0x01, 0x7F],
            127i16.encode_to_slice(&mut buffer).unwrap()
        );

        // -128
        assert_eq!(
            &[0x02, 0x01, 0x80],
            (-128i16).encode_to_slice(&mut buffer).unwrap()
        );

        // 32767
        assert_eq!(
            &[0x02, 0x02, 0x7F, 0xFF],
            32767i16.encode_to_slice(&mut buffer).unwrap()
        );

        // -32768
        assert_eq!(
            &[0x02, 0x02, 0x80, 0x00],
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
