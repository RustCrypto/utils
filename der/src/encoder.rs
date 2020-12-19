//! DER encoder.

use crate::{BitString, Encodable, Error, Header, Integer, Length, Null, OctetString, Result, Tag};
use core::convert::TryInto;

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// DER encoder.
pub struct Encoder<'a> {
    /// Buffer that message is being encoded into
    bytes: &'a mut [u8],

    /// Total number of bytes written to buffer so far
    pos: Length,
}

impl<'a> Encoder<'a> {
    /// Create a new encoder with the given byte slice as a backing buffer.
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self {
            bytes,
            pos: Length::zero(),
        }
    }

    /// Encode a value which impls the [`Encodable`] trait.
    pub fn encode<T: Encodable>(&mut self, encodable: &T) -> Result<()> {
        encodable.encode(self)
    }

    /// Encode the provided value as an ASN.1 `BIT STRING`
    pub fn bit_string(&mut self, value: impl TryInto<BitString<'a>>) -> Result<()> {
        value
            .try_into()
            .map_err(|_| Error::Value {
                tag: Tag::BitString,
            })
            .and_then(|value| self.encode(&value))
    }

    /// Encode the provided value as an ASN.1 `INTEGER`.
    pub fn integer(&mut self, value: impl TryInto<Integer>) -> Result<()> {
        value
            .try_into()
            .map_err(|_| Error::Value { tag: Tag::Integer })
            .and_then(|value| self.encode(&value))
    }

    /// Encode an ASN.1 `NULL` value.
    pub fn null(&mut self) -> Result<()> {
        self.encode(&Null)
    }

    /// Encode the provided value as an ASN.1 `OCTET STRING`
    pub fn octet_string(&mut self, value: impl TryInto<OctetString<'a>>) -> Result<()> {
        value
            .try_into()
            .map_err(|_| Error::Value {
                tag: Tag::OctetString,
            })
            .and_then(|value| self.encode(&value))
    }

    /// Encode an ASN.1 [`ObjectIdentifier`]
    #[cfg(feature = "oid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
    pub fn oid(&mut self, oid: impl TryInto<ObjectIdentifier>) -> Result<()> {
        let oid: ObjectIdentifier = oid.try_into().map_err(|_| Error::Value {
            tag: Tag::ObjectIdentifier,
        })?;

        let expected_len = oid.ber_len();
        Header::new(Tag::ObjectIdentifier, expected_len).and_then(|header| header.encode(self))?;
        let buffer = self.reserve(expected_len)?;

        if oid.write_ber(buffer)?.len() == expected_len {
            Ok(())
        } else {
            Err(Error::Length {
                tag: Tag::ObjectIdentifier,
            })
        }
    }

    /// Encode a sequence of values which impl the [`Encodable`] trait.
    pub fn sequence(&mut self, encodables: &[&dyn Encodable]) -> Result<()> {
        let expected_len = encodables
            .iter()
            .fold(Ok(Length::zero()), |sum, encodable| {
                sum + encodable.encoded_len()?
            })?;

        Header::new(Tag::Sequence, expected_len).and_then(|header| header.encode(self))?;

        let mut nested_encoder = Encoder::new(self.reserve(expected_len)?);

        for encodable in encodables {
            encodable.encode(&mut nested_encoder)?;
        }

        if nested_encoder.finish().len() == expected_len.into() {
            Ok(())
        } else {
            Err(Error::Length {
                tag: Tag::ObjectIdentifier,
            })
        }
    }

    /// Finish encoding to the buffer, returning a slice containing the data
    /// written to the buffer.
    pub fn finish(self) -> &'a [u8] {
        &self.bytes[..self.pos.into()]
    }

    /// Encode a single byte into the backing buffer.
    pub(crate) fn byte(&mut self, byte: u8) -> Result<()> {
        self.reserve(1u8)?
            .first_mut()
            .map(|b| *b = byte)
            .ok_or(Error::Truncated)
    }

    /// Encode the provided byte slice into the backing buffer.
    pub(crate) fn bytes(&mut self, slice: &[u8]) -> Result<()> {
        self.reserve(slice.len())?.copy_from_slice(slice);
        Ok(())
    }

    /// Reserve a portion of the internal buffer, updating the internal cursor
    /// position and returning a mutable slice.
    fn reserve(&mut self, len: impl TryInto<Length>) -> Result<&mut [u8]> {
        let len = len.try_into().map_err(|_| Error::Overflow)?;
        let end = (self.pos + len)?;
        let range = self.pos.into()..end.into();
        let slice = self.bytes.get_mut(range).ok_or(Error::Overlength)?;
        self.pos = (self.pos + slice.len())?;
        Ok(slice)
    }
}
