//! X.509 `SubjectPublicKeyInfo`

use crate::{algorithm, AlgorithmIdentifier, Error, Result};
use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use {crate::PublicKeyDocument, core::convert::TryInto};

#[cfg(feature = "pem")]
use crate::pem;

/// X.509 `SubjectPublicKeyInfo` (SPKI)
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`] and public key
/// data in an algorithm specific format.
///
/// Described in RFC 5208 Section 4.1:
/// <https://tools.ietf.org/html/rfc5280#section-4.1>
///
/// ```text
///    SubjectPublicKeyInfo  ::=  SEQUENCE  {
///         algorithm            AlgorithmIdentifier,
///         subjectPublicKey     BIT STRING  }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SubjectPublicKeyInfo<'a> {
    /// X.509 [`AlgorithmIdentifier`]
    pub algorithm: AlgorithmIdentifier,

    /// Public key data
    pub subject_public_key: &'a [u8],
}

impl<'a> SubjectPublicKeyInfo<'a> {
    /// Parse [`SubjectPublicKeyInfo`] encoded as ASN.1 DER.
    pub fn from_der(mut input: &'a [u8]) -> Result<Self> {
        let mut bytes = der::decode::nested(&mut input, der::Tag::Sequence)?;

        if !input.is_empty() {
            return Err(Error::Decode);
        }

        let algorithm = algorithm::decode_identifier(&mut bytes)?;
        let subject_public_key = der::decode::nested(&mut bytes, der::Tag::BitString)?;

        if !bytes.is_empty() {
            return Err(Error::Decode);
        }

        Ok(Self {
            algorithm,
            subject_public_key,
        })
    }

    /// Write ASN.1 DER-encoded [`SubjectPublicKeyInfo`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'b>(&self, buffer: &'b mut [u8]) -> Result<&'b [u8]> {
        let alg_id_len = algorithm::identifier_len(&self.algorithm)?;
        let private_key_len = der::encode::header_len(self.subject_public_key.len())?
            .checked_add(self.subject_public_key.len())
            .ok_or(Error::Encode)?;
        let sequence_len = alg_id_len
            .checked_add(private_key_len)
            .ok_or(Error::Encode)?;

        let mut offset = der::encode::header(buffer, der::Tag::Sequence, sequence_len)?;
        offset += algorithm::encode_identifier(&mut buffer[offset..], &self.algorithm)?;
        offset += der::encode::nested(
            &mut buffer[offset..],
            der::Tag::BitString,
            self.subject_public_key,
        )?;

        Ok(&buffer[..offset])
    }

    /// Encode this [`SubjectPublicKeyInfo] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PublicKeyDocument {
        let len = spki_len(self).unwrap();
        let mut buffer = vec![0u8; len];
        self.write_der(&mut buffer).unwrap();
        buffer.try_into().expect("malformed DER")
    }

    /// Encode this [`SubjectPublicKeyInfo`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> alloc::string::String {
        let doc = self.to_der();
        pem::encode(doc.as_ref(), pem::PUBLIC_KEY_BOUNDARY)
    }
}

impl<'a> TryFrom<&'a [u8]> for SubjectPublicKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

/// Get the length of DER-encoded [`SubjectPublicKeyInfo`]
#[cfg(feature = "alloc")]
fn spki_len(spki: &SubjectPublicKeyInfo<'_>) -> Result<usize> {
    let alg_id_len = algorithm::identifier_len(&spki.algorithm)?;
    let public_key_len = der::encode::header_len(spki.subject_public_key.len())?
        .checked_add(spki.subject_public_key.len())
        .ok_or(Error::Encode)?;
    let sequence_len = alg_id_len
        .checked_add(public_key_len)
        .ok_or(Error::Encode)?;
    der::encode::header_len(sequence_len)
        .ok()
        .and_then(|n| n.checked_add(sequence_len))
        .ok_or(Error::Encode)
}
