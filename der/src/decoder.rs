//! ASN.1 DER decoder.

use crate::{
    Any, BitString, Decodable, Error, Integer, Null, OctetString, Result, Sequence, Tagged,
};

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// ASN.1 DER decoder.
pub struct Decoder<'a> {
    /// Byte slice being decoded
    bytes: &'a [u8],

    /// Position within the decoded slice
    pos: usize,
}

impl<'a> Decoder<'a> {
    /// Create a new decoder for the given byte slice
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    /// Decode a value which impls the [`Decodable`] trait
    pub fn decode<T: Decodable<'a>>(&mut self) -> Result<T> {
        T::decode(self)
    }

    /// Attempt to decode an ASN.1 `ANY` value
    pub fn any(&mut self) -> Result<Any<'a>> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `BIT STRING`
    pub fn bit_string(&mut self) -> Result<BitString<'a>> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `INTEGER`
    pub fn integer(&mut self) -> Result<Integer> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `NULL` value
    pub fn null(&mut self) -> Result<Null> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `OCTET STRING`
    pub fn octet_string(&mut self) -> Result<OctetString<'a>> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `OBJECT IDENTIFIER`
    #[cfg(feature = "oid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
    pub fn oid(&mut self) -> Result<ObjectIdentifier> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `OPTIONAL` value
    pub fn optional<T: Decodable<'a>>(&mut self) -> Result<Option<T>> {
        self.decode()
    }

    /// Attempt to decode an ASN.1 `SEQUENCE`, creating a new nested
    /// [`Decoder`] and calling the provided argument with it.
    pub fn sequence<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(Decoder<'a>) -> Result<T>,
    {
        Sequence::decode(self).and_then(|seq| f(seq.decoder()))
    }

    /// Finish decoding, returning the given value if there is no
    /// remaining data, or an error otherwise
    pub fn finish<T: Tagged>(self, value: T) -> Result<T> {
        if self.is_finished() {
            Ok(value)
        } else {
            Err(Error::Length { tag: T::TAG })
        }
    }

    /// Have we decoded all of the bytes in this [`Decoder`]?
    pub fn is_finished(&self) -> bool {
        self.remaining().is_empty()
    }

    /// Decode a single byte, updating the internal cursor.
    pub(crate) fn byte(&mut self) -> Result<u8> {
        let byte = *self.bytes.get(self.pos).ok_or(Error::Truncated)?;
        self.pos = self.pos.checked_add(1).ok_or(Error::Overflow)?;
        Ok(byte)
    }

    /// Obtain a slice of bytes of the given length from the current cursor
    /// position, or return an error if we have insufficient data.
    pub(crate) fn bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if len > self.remaining().len() {
            return Err(Error::Truncated);
        }

        let result = &self.remaining()[..len];
        self.pos = self.pos.checked_add(len).ok_or(Error::Overflow)?;
        Ok(result)
    }

    /// Obtain the remaining bytes in this decoder from the current cursor
    /// position.
    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }
}

impl<'a> From<&'a [u8]> for Decoder<'a> {
    fn from(bytes: &'a [u8]) -> Decoder<'a> {
        Decoder::new(bytes)
    }
}
