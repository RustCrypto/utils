//! PKCS#8 `PrivateKeyInfo`.

use crate::{algorithm, AlgorithmIdentifier, Error, Result};
use core::{convert::TryFrom, fmt};

#[cfg(feature = "alloc")]
use {crate::document::PrivateKeyDocument, zeroize::Zeroizing};

#[cfg(feature = "pem")]
use crate::pem;

/// PKCS#8 `PrivateKeyInfo`
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`] and private key
/// data in an algorithm specific format.
///
/// Described in RFC 5208 Section 5:
/// <https://tools.ietf.org/html/rfc5208#section-5>
///
/// ```text
/// PrivateKeyInfo ::= SEQUENCE {
///         version                   Version,
///         privateKeyAlgorithm       PrivateKeyAlgorithmIdentifier,
///         privateKey                PrivateKey,
///         attributes           [0]  IMPLICIT Attributes OPTIONAL }
///
/// Version ::= INTEGER
///
/// PrivateKeyAlgorithmIdentifier ::= AlgorithmIdentifier
///
/// PrivateKey ::= OCTET STRING
///
/// Attributes ::= SET OF Attribute
/// ```
#[derive(Copy, Clone)]
pub struct PrivateKeyInfo<'a> {
    /// X.509 [`AlgorithmIdentifier`]
    pub algorithm: AlgorithmIdentifier,

    /// Private key data
    pub private_key: &'a [u8],
}

impl<'a> PrivateKeyInfo<'a> {
    /// Parse [`PrivateKeyInfo`] encoded as ASN.1 DER.
    pub fn from_der(mut input: &'a [u8]) -> Result<Self> {
        let mut bytes = der::decode::nested(&mut input, der::Tag::Sequence)?;

        if !input.is_empty() {
            return Err(Error::Decode);
        }

        // Parse `version` INTEGER
        let version = der::decode::integer(&mut bytes)?;

        // RFC 5208 designates `0` as the only valid version for PKCS#8 documents
        if version != 0 {
            return Err(Error::Decode);
        }

        let algorithm = algorithm::decode_identifier(&mut bytes)?;
        let private_key = der::decode::nested(&mut bytes, der::Tag::OctetString)?;

        // We currently don't support any trailing attribute data
        if !bytes.is_empty() {
            return Err(Error::Decode);
        }

        Ok(Self {
            algorithm,
            private_key,
        })
    }

    /// Write ASN.1 DER-encoded [`PrivateKeyInfo`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'b>(&self, buffer: &'b mut [u8]) -> Result<&'b [u8]> {
        let offset = encode_private_key_info(buffer, self)?;
        Ok(&buffer[..offset])
    }

    /// Encode this [`PrivateKeyInfo`] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PrivateKeyDocument {
        let len = private_key_info_len(self).unwrap();
        let mut buffer = Zeroizing::new(vec![0u8; len]);
        self.write_der(&mut buffer).unwrap();
        PrivateKeyDocument::from_der(&buffer).expect("malformed DER")
    }

    /// Encode this [`PrivateKeyInfo`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> Zeroizing<alloc::string::String> {
        let doc = self.to_der();
        let pem = pem::encode(doc.as_ref(), pem::PRIVATE_KEY_BOUNDARY);
        Zeroizing::new(pem)
    }
}

impl<'a> TryFrom<&'a [u8]> for PrivateKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl<'a> fmt::Debug for PrivateKeyInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKeyInfo")
            .field("algorithm", &self.algorithm)
            .finish() // TODO(tarcieri): use `finish_non_exhaustive` when stable
    }
}

/// Get the length of DER-encoded [`PrivateKeyInfo`]
#[cfg(feature = "alloc")]
fn private_key_info_len(private_key_info: &PrivateKeyInfo<'_>) -> Result<usize> {
    let alg_id_len = algorithm::identifier_len(&private_key_info.algorithm)?;
    let version_len = 3;
    let private_key_len = der::length::header(private_key_info.private_key.len())?
        .checked_add(private_key_info.private_key.len())
        .ok_or(Error::Encode)?;
    let sequence_len = alg_id_len
        .checked_add(version_len)
        .and_then(|len| len.checked_add(private_key_len))
        .ok_or(Error::Encode)?;
    der::length::header(sequence_len)
        .ok()
        .and_then(|n| n.checked_add(sequence_len))
        .ok_or(Error::Encode)
}

/// Encode [`PrivateKeyInfo`]
fn encode_private_key_info(
    buffer: &mut [u8],
    private_key_info: &PrivateKeyInfo<'_>,
) -> Result<usize> {
    let alg_id_len = algorithm::identifier_len(&private_key_info.algorithm)?;
    let version_len = 3;
    let private_key_len = der::length::header(private_key_info.private_key.len())?
        .checked_add(private_key_info.private_key.len())
        .ok_or(Error::Encode)?;
    let sequence_len = alg_id_len
        .checked_add(version_len)
        .and_then(|len| len.checked_add(private_key_len))
        .ok_or(Error::Encode)?;

    let mut offset = der::encode::header(buffer, der::Tag::Sequence, sequence_len)?;
    offset += der::encode::nested(&mut buffer[offset..], der::Tag::Integer, &[0])?;
    offset += algorithm::encode_identifier(&mut buffer[offset..], &private_key_info.algorithm)?;
    offset += der::encode::nested(
        &mut buffer[offset..],
        der::Tag::OctetString,
        private_key_info.private_key,
    )?;

    Ok(offset)
}
