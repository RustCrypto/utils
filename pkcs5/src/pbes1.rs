//! Password-Based Encryption Scheme 1 as defined in [RFC 8018 Section 6.1].
//!
//! [RFC 8018 Section 6.1]: https://tools.ietf.org/html/rfc8018#section-6.1

use crate::{AlgorithmIdentifier, Error, ObjectIdentifier, Result};
use core::convert::{TryFrom, TryInto};
use der::ErrorKind;

/// `pbeWithMD2AndDES-CBC` Object Identifier (OID).
pub const PBE_WITH_MD2_AND_DES_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 1]);

/// `pbeWithMD2AndRC2-CBC` Object Identifier (OID).
pub const PBE_WITH_MD2_AND_RC2_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 4]);

/// `pbeWithMD5AndDES-CBC` Object Identifier (OID).
pub const PBE_WITH_MD5_AND_DES_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 3]);

/// `pbeWithMD5AndRC2-CBC` Object Identifier (OID).
pub const PBE_WITH_MD5_AND_RC2_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 6]);

/// `pbeWithSHA1AndDES-CBC` Object Identifier (OID).
pub const PBE_WITH_SHA1_AND_DES_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 10]);

/// `pbeWithSHA1AndRC2-CBC` Object Identifier (OID).
pub const PBE_WITH_SHA1_AND_RC2_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 11]);

/// Length of a PBES1 salt (as defined in the `PBEParameter` ASN.1 message).
pub const SALT_LENGTH: usize = 8;

/// Password-Based Encryption Scheme 1 parameters as defined in [RFC 8018 Appendix A.3].
///
/// ```text
/// PBEParameter ::= SEQUENCE {
///    salt OCTET STRING (SIZE(8)),
///    iterationCount INTEGER }
/// ```
///
/// Note that this struct additionally stores an [`EncryptionScheme`] parameter
/// parsed from the [`ObjectIdentifier`].
///
/// [RFC 8018 Appendix A.3]: https://tools.ietf.org/html/rfc8018#appendix-A.3
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Parameters {
    /// Encryption scheme
    pub encryption: EncryptionScheme,

    /// Salt value
    pub salt: [u8; SALT_LENGTH],

    /// Iteration count
    pub iteration_count: u16,
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Parameters {
    type Error = Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Self> {
        // Ensure that we have a supported PBES1 algorithm identifier
        let encryption = EncryptionScheme::try_from(alg.oid).map_err(|_| ErrorKind::Value {
            tag: der::Tag::ObjectIdentifier,
        })?;

        alg.parameters_any()?.sequence(|params| {
            let salt =
                params
                    .octet_string()?
                    .as_bytes()
                    .try_into()
                    .map_err(|_| ErrorKind::Value {
                        tag: der::Tag::OctetString,
                    })?;

            let iteration_count = params.decode()?;

            Ok(Self {
                encryption,
                salt,
                iteration_count,
            })
        })
    }
}

/// Password-Based Encryption Scheme 1 algorithms as defined in [RFC 8018 Appendix A.3].
///
/// [RFC 8018 Appendix A.3]: https://tools.ietf.org/html/rfc8018#appendix-A.3
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EncryptionScheme {
    /// `pbeWithMD2AndDES-CBC`
    PbeWithMd2AndDesCbc,

    /// `pbeWithMD2AndRC2-CBC`
    PbeWithMd2AndRc2Cbc,

    /// `pbeWithMD5AndDES-CBC`
    PbeWithMd5AndDesCbc,

    /// `pbeWithMD5AndRC2-CBC`
    PbeWithMd5AndRc2Cbc,

    /// `pbeWithSHA1AndDES-CBC`
    PbeWithSha1AndDesCbc,

    /// `pbeWithSHA1AndRC2-CBC`
    PbeWithSha1AndRc2Cbc,
}

impl TryFrom<ObjectIdentifier> for EncryptionScheme {
    type Error = Error;

    fn try_from(oid: ObjectIdentifier) -> Result<Self> {
        match oid {
            PBE_WITH_MD2_AND_DES_CBC_OID => Ok(Self::PbeWithMd2AndDesCbc),
            PBE_WITH_MD2_AND_RC2_CBC_OID => Ok(Self::PbeWithMd2AndRc2Cbc),
            PBE_WITH_MD5_AND_DES_CBC_OID => Ok(Self::PbeWithMd5AndDesCbc),
            PBE_WITH_MD5_AND_RC2_CBC_OID => Ok(Self::PbeWithMd5AndRc2Cbc),
            PBE_WITH_SHA1_AND_DES_CBC_OID => Ok(Self::PbeWithSha1AndDesCbc),
            PBE_WITH_SHA1_AND_RC2_CBC_OID => Ok(Self::PbeWithSha1AndRc2Cbc),
            _ => Err(ErrorKind::OidInvalid(oid).into()),
        }
    }
}

impl EncryptionScheme {
    /// Get the [`SymmetricCipher`] to be used.
    pub fn cipher(self) -> SymmetricCipher {
        match self {
            Self::PbeWithMd2AndDesCbc => SymmetricCipher::DesCbc,
            Self::PbeWithMd2AndRc2Cbc => SymmetricCipher::Rc2Cbc,
            Self::PbeWithMd5AndDesCbc => SymmetricCipher::DesCbc,
            Self::PbeWithMd5AndRc2Cbc => SymmetricCipher::Rc2Cbc,
            Self::PbeWithSha1AndDesCbc => SymmetricCipher::DesCbc,
            Self::PbeWithSha1AndRc2Cbc => SymmetricCipher::Rc2Cbc,
        }
    }

    /// Get the [`DigestAlgorithm`] to be used.
    pub fn digest(self) -> DigestAlgorithm {
        match self {
            Self::PbeWithMd2AndDesCbc => DigestAlgorithm::Md2,
            Self::PbeWithMd2AndRc2Cbc => DigestAlgorithm::Md2,
            Self::PbeWithMd5AndDesCbc => DigestAlgorithm::Md5,
            Self::PbeWithMd5AndRc2Cbc => DigestAlgorithm::Md5,
            Self::PbeWithSha1AndDesCbc => DigestAlgorithm::Sha1,
            Self::PbeWithSha1AndRc2Cbc => DigestAlgorithm::Sha1,
        }
    }

    /// Get the Object Identifier (OID) for this algorithm.
    pub fn oid(self) -> ObjectIdentifier {
        match self {
            Self::PbeWithMd2AndDesCbc => PBE_WITH_MD2_AND_DES_CBC_OID,
            Self::PbeWithMd2AndRc2Cbc => PBE_WITH_MD2_AND_RC2_CBC_OID,
            Self::PbeWithMd5AndDesCbc => PBE_WITH_MD5_AND_DES_CBC_OID,
            Self::PbeWithMd5AndRc2Cbc => PBE_WITH_MD5_AND_RC2_CBC_OID,
            Self::PbeWithSha1AndDesCbc => PBE_WITH_SHA1_AND_DES_CBC_OID,
            Self::PbeWithSha1AndRc2Cbc => PBE_WITH_SHA1_AND_RC2_CBC_OID,
        }
    }
}

/// Digest algorithms supported by PBES1.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DigestAlgorithm {
    /// MD2
    Md2,

    /// MD5
    Md5,

    /// SHA-1
    Sha1,
}

/// Symmetric encryption ciphers supported by PBES1.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SymmetricCipher {
    /// DES in CBC mode
    DesCbc,

    /// RC2 in CBC mode
    Rc2Cbc,
}
