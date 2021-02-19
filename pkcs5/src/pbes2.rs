//! Password-Based Encryption Scheme 2 as defined in [RFC 8018 Section 6.2].
//!
//! [RFC 8018 Section 6.2]: https://tools.ietf.org/html/rfc8018#section-6.2

use crate::{AlgorithmIdentifier, ObjectIdentifier, Result};
use core::convert::{TryFrom, TryInto};
use der::{Decodable, ErrorKind};

/// Password-Based Encryption Scheme 2 (PBES2) OID.
///
/// <https://tools.ietf.org/html/rfc8018#section-6.2>
pub const PBES2_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 13]);

/// Password-Based Key Derivation Function (PBKDF2) OID.
// TODO(tarcieri): move this to the `pbkdf2` crate?
pub const PBKDF2_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 12]);

/// HMAC-SHA-256 message authentication scheme (RFC 4231 and RFC8018)
pub const HMAC_WITH_SHA256_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 2, 9]);

/// 256-bit Advanced Encryption Standard (AES) algorithm with Cipher-Block
/// Chaining (CBC) mode of operation.
pub const AES_256_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[2, 16, 840, 1, 101, 3, 4, 1, 42]);

/// AES cipher block size
const AES_BLOCK_SIZE: usize = 16;

/// Password-Based Encryption Scheme 2 parameters as defined in [RFC 8018 Appendix A.4].
///
/// ```text
///  PBES2-params ::= SEQUENCE {
///       keyDerivationFunc AlgorithmIdentifier {{PBES2-KDFs}},
///       encryptionScheme AlgorithmIdentifier {{PBES2-Encs}} }
/// ```
///
/// [RFC 8018 Appendix A.4]: https://tools.ietf.org/html/rfc8018#appendix-A.4
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Parameters<'a> {
    /// Key derivation function
    pub kdf: Kdf<'a>,

    /// Encryption scheme
    pub encryption: EncryptionScheme<'a>,
}

impl<'a> TryFrom<der::Any<'a>> for Parameters<'a> {
    type Error = der::Error;

    fn try_from(any: der::Any<'a>) -> Result<Self> {
        any.sequence(|params| {
            let kdf = AlgorithmIdentifier::decode(params)?;
            let encryption = AlgorithmIdentifier::decode(params)?;

            Ok(Self {
                kdf: kdf.try_into()?,
                encryption: encryption.try_into()?,
            })
        })
    }
}

/// Password-based key derivation function.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Kdf<'a> {
    /// Password-Based Key Derivation Function 2 (PBKDF2).
    Pbkdf2(Pbkdf2Params<'a>),
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Kdf<'a> {
    type Error = der::Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Self> {
        match alg.oid {
            PBKDF2_OID => alg
                .parameters_any()
                .and_then(TryFrom::try_from)
                .map(Self::Pbkdf2),
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}

impl<'a> Kdf<'a> {
    /// Get [`Pbkdf2Params`] if it is the selected algorithm.
    pub fn pbkdf2(self) -> Option<Pbkdf2Params<'a>> {
        match self {
            Self::Pbkdf2(params) => Some(params),
        }
    }
}

/// Password-Based Encryption Scheme 2 parameters as defined in [RFC 8018 Appendix A.2].
///
/// ```text
/// PBKDF2-params ::= SEQUENCE {
///     salt CHOICE {
///         specified OCTET STRING,
///         otherSource AlgorithmIdentifier {{PBKDF2-SaltSources}}
///     },
///     iterationCount INTEGER (1..MAX),
///     keyLength INTEGER (1..MAX) OPTIONAL,
///     prf AlgorithmIdentifier {{PBKDF2-PRFs}} DEFAULT
///     algid-hmacWithSHA1 }
/// ```
///
/// [RFC 8018 Appendix A.2]: https://tools.ietf.org/html/rfc8018#appendix-A.2
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Pbkdf2Params<'a> {
    /// PBKDF2 salt
    // TODO(tarcieri): support `CHOICE` with `otherSource`
    pub salt: &'a [u8],

    /// PBKDF2 iteration count
    pub iteration_count: u16,

    /// PBKDF2 output length
    // TODO(tarcieri): support this OPTIONAL field
    // Blocked on: https://github.com/RustCrypto/utils/issues/271
    pub key_length: Option<u16>,

    /// Pseudo-random function to use with PBKDF2
    pub prf: Pbkdf2Prf,
}

impl<'a> TryFrom<der::Any<'a>> for Pbkdf2Params<'a> {
    type Error = der::Error;

    fn try_from(any: der::Any<'a>) -> Result<Self> {
        any.sequence(|params| {
            // TODO(tarcieri): support salt `CHOICE` w\ `AlgorithmIdentifier`
            let salt = params.octet_string()?;
            let iteration_count = params.decode()?;

            // TODO(tarcieri): support OPTIONAL key length field
            // Blocked on: https://github.com/RustCrypto/utils/issues/271
            let key_length = None;
            let prf = AlgorithmIdentifier::decode(params)?;

            Ok(Self {
                salt: salt.as_bytes(),
                iteration_count,
                key_length,
                prf: prf.try_into()?,
            })
        })
    }
}

/// Pseudo-random function used by PBKDF2.
// TODO(tarcieri): add all PRFs specified in RFC 8018, e.g. `algid-hmacWithSHA1`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Pbkdf2Prf {
    /// HMAC with SHA-256
    HmacWithSha256,
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Pbkdf2Prf {
    type Error = der::Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Self> {
        // TODO(tarcieri): support non-NULL parameters?
        if let Some(params) = alg.parameters {
            if !params.is_null() {
                return Err(der::ErrorKind::Value { tag: params.tag() }.into());
            }
        } else {
            // TODO(tarcieri): support OPTIONAL parameters?
            return Err(der::ErrorKind::Truncated.into());
        }

        match alg.oid {
            HMAC_WITH_SHA256_OID => Ok(Self::HmacWithSha256),
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}

/// Symmetric encryption scheme used by PBES2.
// TODO(tarcieri): add all ciphers specified in RFC 8018
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EncryptionScheme<'a> {
    /// AES-256 in CBC mode
    Aes256Cbc {
        /// Initialization vector
        iv: &'a [u8; AES_BLOCK_SIZE],
    },
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for EncryptionScheme<'a> {
    type Error = der::Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Self> {
        match alg.oid {
            AES_256_CBC_OID => {
                let iv = alg
                    .parameters_any()?
                    .octet_string()?
                    .as_bytes()
                    .try_into()
                    .map_err(|_| der::ErrorKind::Value {
                        tag: der::Tag::OctetString,
                    })?;

                Ok(Self::Aes256Cbc { iv })
            }
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}
