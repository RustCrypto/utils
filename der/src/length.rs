//! Length calculations for encoded ASN.1 DER values

use crate::{Decodable, Decoder, Encodable, Encoder, Error, ErrorKind, Result};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
    ops::Add,
};

/// ASN.1-encoded length.
///
/// # Limits
///
/// Presently constrained to the range `0..=65535`
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Length(u32);

impl Length {
    /// Create a new [`Length`] for any value which fits inside of a [`u16`].
    ///
    /// This function is const-safe and therefore useful for [`Length`] constants.
    pub const fn new(value: u16) -> Self {
        Length(value as u32)
    }

    /// Return a length of `0`.
    pub const fn zero() -> Self {
        Length(0)
    }

    /// Return a length of `1`.
    pub const fn one() -> Self {
        Length(1)
    }

    /// Get the maximum length supported by this crate
    // TODO(tarcieri): support lengths greater than 65535
    pub const fn max() -> Self {
        Self(65535)
    }

    /// Get the length of DER Tag-Length-Value (TLV) encoded data if `self`
    /// is the length of the inner "value" portion of the message
    pub fn for_tlv(self) -> Result<Self> {
        Length(1) + self.encoded_len()? + self
    }
}

impl Add for Length {
    type Output = Result<Self>;

    fn add(self, other: Self) -> Result<Self> {
        self.0
            .checked_add(other.0)
            .ok_or_else(|| ErrorKind::Overflow.into())
            .and_then(TryInto::try_into)
    }
}

impl Add<u8> for Length {
    type Output = Result<Self>;

    fn add(self, other: u8) -> Result<Self> {
        self + Length::from(other)
    }
}

impl Add<u16> for Length {
    type Output = Result<Self>;

    fn add(self, other: u16) -> Result<Self> {
        self + Length::from(other)
    }
}

impl Add<u32> for Length {
    type Output = Result<Self>;

    fn add(self, other: u32) -> Result<Self> {
        self + Length::try_from(other)?
    }
}

impl Add<usize> for Length {
    type Output = Result<Self>;

    fn add(self, other: usize) -> Result<Self> {
        self + Length::try_from(other)?
    }
}

impl Add<Length> for Result<Length> {
    type Output = Self;

    fn add(self, other: Length) -> Self {
        self? + other
    }
}

impl From<u8> for Length {
    fn from(len: u8) -> Length {
        Length(len as u32)
    }
}

impl From<u16> for Length {
    fn from(len: u16) -> Length {
        Length(len as u32)
    }
}

impl TryFrom<u32> for Length {
    type Error = Error;

    fn try_from(len: u32) -> Result<Length> {
        if len <= Self::max().0 {
            Ok(Length(len))
        } else {
            Err(ErrorKind::Overflow.into())
        }
    }
}

impl TryFrom<usize> for Length {
    type Error = Error;

    fn try_from(len: usize) -> Result<Length> {
        u32::try_from(len)
            .map_err(|_| ErrorKind::Overflow)?
            .try_into()
    }
}

impl TryFrom<Length> for usize {
    type Error = Error;

    fn try_from(len: Length) -> Result<usize> {
        len.0.try_into().map_err(|_| ErrorKind::Overflow.into())
    }
}

impl Decodable<'_> for Length {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Length> {
        match decoder.byte()? {
            // Note: per X.690 Section 8.1.3.6.1 the byte 0x80 encodes indefinite
            // lengths, which are not allowed in DER, so disallow that byte.
            len if len < 0x80 => Ok(len.into()),
            0x81 => {
                let len = decoder.byte()?;

                // X.690 Section 10.1: DER lengths must be encoded with a minimum
                // number of octets
                if len >= 0x80 {
                    Ok(len.into())
                } else {
                    Err(ErrorKind::Noncanonical.into())
                }
            }
            0x82 => {
                let len_hi = decoder.byte()? as u16;
                let len = (len_hi << 8) | (decoder.byte()? as u16);

                // X.690 Section 10.1: DER lengths must be encoded with a minimum
                // number of octets
                if len > 0xFF {
                    Ok(len.into())
                } else {
                    Err(ErrorKind::Noncanonical.into())
                }
            }
            _ => {
                // We specialize to a maximum 3-byte length
                // TODO(tarcieri): support lengths greater than 65535
                Err(ErrorKind::Overlength.into())
            }
        }
    }
}

impl Encodable for Length {
    fn encoded_len(&self) -> Result<Length> {
        match self.0 {
            0..=0x7F => Ok(Length(1)),
            0x80..=0xFF => Ok(Length(2)),
            0x100..=0xFFFF => Ok(Length(3)),
            // TODO(tarcieri): support lengths greater than 65535
            _ => Err(ErrorKind::Overflow.into()),
        }
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        match self.0 {
            0..=0x7F => encoder.byte(self.0 as u8),
            0x80..=0xFF => {
                encoder.byte(0x81)?;
                encoder.byte(self.0 as u8)
            }
            0x100..=0xFFFF => {
                encoder.byte(0x82)?;
                encoder.byte((self.0 >> 8) as u8)?;
                encoder.byte((self.0 & 0xFF) as u8)
            }
            // TODO(tarcieri): support lengths greater than 65535
            _ => Err(ErrorKind::Overflow.into()),
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Length;
    use crate::{Decodable, Encodable, ErrorKind};

    #[test]
    fn decode() {
        assert_eq!(Length::zero(), Length::from_der(&[0x00]).unwrap());

        assert_eq!(Length::from(0x7Fu8), Length::from_der(&[0x7F]).unwrap());

        assert_eq!(
            Length::from(0x80u8),
            Length::from_der(&[0x81, 0x80]).unwrap()
        );

        assert_eq!(
            Length::from(0xFFu8),
            Length::from_der(&[0x81, 0xFF]).unwrap()
        );

        assert_eq!(
            Length::from(0x100u16),
            Length::from_der(&[0x82, 0x01, 0x00]).unwrap()
        );
    }

    #[test]
    fn encode() {
        let mut buffer = [0u8; 3];

        assert_eq!(
            &[0x00],
            Length::zero().encode_to_slice(&mut buffer).unwrap()
        );

        assert_eq!(
            &[0x7F],
            Length::from(0x7Fu8).encode_to_slice(&mut buffer).unwrap()
        );

        assert_eq!(
            &[0x81, 0x80],
            Length::from(0x80u8).encode_to_slice(&mut buffer).unwrap()
        );

        assert_eq!(
            &[0x81, 0xFF],
            Length::from(0xFFu8).encode_to_slice(&mut buffer).unwrap()
        );

        assert_eq!(
            &[0x82, 0x01, 0x00],
            Length::from(0x100u16).encode_to_slice(&mut buffer).unwrap()
        );
    }

    #[test]
    fn reject_indefinite_lengths() {
        assert!(Length::from_der(&[0x80]).is_err());
    }

    #[test]
    fn add_overflows_when_max_length_exceeded() {
        let result = Length::max() + Length::one();
        assert_eq!(
            result.err().map(|err| err.kind()),
            Some(ErrorKind::Overflow)
        );
    }
}
