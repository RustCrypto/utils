//! PKCS#8 `PrivateKeyInfo`.

use crate::{AlgorithmIdentifier, Error, Result};
use core::{convert::TryFrom, fmt};
use der::{Decodable, Encodable, Message};

#[cfg(feature = "alloc")]
use {
    crate::{error, PrivateKeyDocument},
    core::convert::TryInto,
};

#[cfg(feature = "pem")]
use {crate::pem, zeroize::Zeroizing};

/// RFC 5208 designates `0` as the only valid version for PKCS#8 documents
const VERSION: i8 = 0;

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
    pub fn from_der(bytes: &'a [u8]) -> Result<Self> {
        let mut decoder = der::Decoder::new(bytes);
        let result = Self::decode(&mut decoder)?;
        decoder.finish(result).map_err(|_| Error::Decode)
    }

    /// Write ASN.1 DER-encoded [`PrivateKeyInfo`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'b>(&self, buffer: &'b mut [u8]) -> Result<&'b [u8]> {
        let mut encoder = der::Encoder::new(buffer);
        self.encode(&mut encoder)?;
        Ok(encoder.finish()?)
    }

    /// Encode this [`PrivateKeyInfo`] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PrivateKeyDocument {
        self.to_vec()
            .ok()
            .and_then(|buf| buf.try_into().ok())
            .expect(error::DER_ENCODING_MSG)
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

impl<'a> TryFrom<der::Any<'a>> for PrivateKeyInfo<'a> {
    type Error = der::Error;

    fn try_from(any: der::Any<'a>) -> der::Result<PrivateKeyInfo<'a>> {
        any.sequence(|mut decoder| {
            // Parse and validate `version` INTEGER.
            if i8::decode(&mut decoder)? != VERSION {
                return Err(der::ErrorKind::Value {
                    tag: der::Tag::Integer,
                }
                .into());
            }

            let algorithm = decoder.decode()?;
            let private_key = decoder.octet_string()?.into();

            decoder.finish(Self {
                algorithm,
                private_key,
            })
        })
    }
}

impl<'a> Message<'a> for PrivateKeyInfo<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[
            &VERSION,
            &self.algorithm,
            &der::OctetString::new(self.private_key)?,
        ])
    }
}

impl<'a> fmt::Debug for PrivateKeyInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKeyInfo")
            .field("algorithm", &self.algorithm)
            .finish() // TODO(tarcieri): use `finish_non_exhaustive` when stable
    }
}
