//! ASN.1 `INTEGER` support.

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
