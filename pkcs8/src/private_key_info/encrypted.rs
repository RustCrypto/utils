//! PKCS#8 `EncryptedPrivateKeyInfo`

use crate::{AlgorithmIdentifier, Error, Result};
use core::convert::TryFrom;
use der::{Decodable, Encodable, Message};

/// PKCS#8 `EncryptedPrivateKeyInfo`.
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`] identifying a
/// symmetric encryption scheme and encrypted private key data.
///
/// ## Encryption algorithm support
///
/// tl;dr: none yet!
///
/// This crate does not (yet) support decrypting/encrypting private key data.
///
/// [PKCS#5 v1.5] supports several password-based encryption algorithms,
/// including `PBE-SHA1-3DES`.
///
/// [PKCS#5 v2] adds support for AES encryption with iterated PRFs
/// such as `hmacWithSHA256`.
///
/// We may consider adding support for these in future releases of this crate.
///
/// ## Schema
/// Structure described in [RFC 5208 Section 6]:
///
/// ```text
/// EncryptedPrivateKeyInfo ::= SEQUENCE {
///   encryptionAlgorithm  EncryptionAlgorithmIdentifier,
///   encryptedData        EncryptedData }
///
/// EncryptionAlgorithmIdentifier ::= AlgorithmIdentifier
///
/// EncryptedData ::= OCTET STRING
/// ```
///
/// [RFC 5208 Section 6]: https://tools.ietf.org/html/rfc5208#section-6
/// [PKCS#5 v1.5]: https://tools.ietf.org/html/rfc2898
/// [PKCS#5 v2]: https://tools.ietf.org/html/rfc8018
#[derive(Copy, Clone)]
pub struct EncryptedPrivateKeyInfo<'a> {
    /// [`AlgorithmIdentifier`] for the symmetric encryption algorithm used to
    /// encrypt the `encrypted_data` field.
    pub encryption_algorithm: AlgorithmIdentifier<'a>,

    /// Private key data
    pub encrypted_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for EncryptedPrivateKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_bytes(bytes)?)
    }
}

impl<'a> TryFrom<der::Any<'a>> for EncryptedPrivateKeyInfo<'a> {
    type Error = der::Error;

    fn try_from(any: der::Any<'a>) -> der::Result<EncryptedPrivateKeyInfo<'a>> {
        any.sequence(|decoder| {
            Ok(Self {
                encryption_algorithm: decoder.decode()?,
                encrypted_data: decoder.octet_string()?.as_bytes(),
            })
        })
    }
}

impl<'a> Message<'a> for EncryptedPrivateKeyInfo<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[
            &self.encryption_algorithm,
            &der::OctetString::new(self.encrypted_data)?,
        ])
    }
}
