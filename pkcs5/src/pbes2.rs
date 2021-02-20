//! Password-Based Encryption Scheme 2 as defined in [RFC 8018 Section 6.2].
//!
//! [RFC 8018 Section 6.2]: https://tools.ietf.org/html/rfc8018#section-6.2

use crate::{AlgorithmIdentifier, ObjectIdentifier, Result};
use core::convert::{TryFrom, TryInto};
use der::{Any, Decodable, Encodable, Encoder, Error, ErrorKind, Length, Message, OctetString};
use spki::AlgorithmParameters;

/// Password-Based Encryption Scheme 2 (PBES2) OID.
///
/// <https://tools.ietf.org/html/rfc8018#section-6.2>
pub const PBES2_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 13]);

/// Password-Based Key Derivation Function (PBKDF2) OID.
pub const PBKDF2_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 113549, 1, 5, 12]);

/// HMAC-SHA1 (for use with PBKDF2)
pub const HMAC_WITH_SHA1_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 113549, 2, 7]);

/// HMAC-SHA-256 (for use with PBKDF2)
pub const HMAC_WITH_SHA256_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[1, 2, 840, 113549, 2, 9]);

/// 128-bit Advanced Encryption Standard (AES) algorithm with Cipher-Block
/// Chaining (CBC) mode of operation.
pub const AES_128_CBC_OID: ObjectIdentifier =
    ObjectIdentifier::new(&[2, 16, 840, 1, 101, 3, 4, 1, 2]);

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameters<'a> {
    /// Key derivation function
    pub kdf: Kdf<'a>,

    /// Encryption scheme
    pub encryption: EncryptionScheme<'a>,
}

impl<'a> TryFrom<Any<'a>> for Parameters<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
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

impl<'a> Message<'a> for Parameters<'a> {
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        f(&[&self.kdf, &self.encryption])
    }
}

/// Password-based key derivation function.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Kdf<'a> {
    /// Password-Based Key Derivation Function 2 (PBKDF2).
    Pbkdf2(Pbkdf2Params<'a>),
}

impl<'a> Kdf<'a> {
    /// Get the [`ObjectIdentifier`] (a.k.a OID) for this algorithm.
    pub fn oid(&self) -> ObjectIdentifier {
        match self {
            Self::Pbkdf2(_) => PBKDF2_OID,
        }
    }

    /// Get [`Pbkdf2Params`] if it is the selected algorithm.
    pub fn pbkdf2(&self) -> Option<&Pbkdf2Params<'a>> {
        match self {
            Self::Pbkdf2(params) => Some(params),
        }
    }

    /// Is the selected KDF PBKDF2?
    pub fn is_pbkdf2(&self) -> bool {
        self.pbkdf2().is_some()
    }
}

impl<'a> TryFrom<Any<'a>> for Kdf<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        AlgorithmIdentifier::try_from(any).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Kdf<'a> {
    type Error = Error;

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

impl<'a> Message<'a> for Kdf<'a> {
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        match self {
            Self::Pbkdf2(params) => f(&[&self.oid(), params]),
        }
    }
}

/// Password-Based Key Derivation Scheme 2 parameters as defined in
/// [RFC 8018 Appendix A.2].
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

impl<'a> TryFrom<Any<'a>> for Pbkdf2Params<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.sequence(|params| {
            // TODO(tarcieri): support salt `CHOICE` w\ `AlgorithmIdentifier`
            let salt = params.octet_string()?;
            let iteration_count = params.decode()?;

            // TODO(tarcieri): support OPTIONAL key length field
            // Blocked on: https://github.com/RustCrypto/utils/issues/271
            let key_length = None;
            let prf: Option<AlgorithmIdentifier<'_>> = params.optional()?;

            Ok(Self {
                salt: salt.as_bytes(),
                iteration_count,
                key_length,
                prf: prf.map(TryInto::try_into).transpose()?.unwrap_or_default(),
            })
        })
    }
}

impl<'a> Message<'a> for Pbkdf2Params<'a> {
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        if self.prf == Pbkdf2Prf::default() {
            f(&[
                &OctetString::new(self.salt)?,
                &self.iteration_count,
                &self.key_length,
            ])
        } else {
            f(&[
                &OctetString::new(self.salt)?,
                &self.iteration_count,
                &self.key_length,
                &self.prf,
            ])
        }
    }
}

/// Pseudo-random function used by PBKDF2.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Pbkdf2Prf {
    /// HMAC with SHA1
    HmacWithSha1,

    /// HMAC with SHA-256
    HmacWithSha256,
}

impl Pbkdf2Prf {
    /// Get the [`ObjectIdentifier`] (a.k.a OID) for this algorithm.
    pub fn oid(self) -> ObjectIdentifier {
        match self {
            Self::HmacWithSha1 => HMAC_WITH_SHA1_OID,
            Self::HmacWithSha256 => HMAC_WITH_SHA256_OID,
        }
    }
}

/// Default PRF as specified in RFC 8018 Appendix A.2:
///
/// ```text
/// prf AlgorithmIdentifier {{PBKDF2-PRFs}} DEFAULT algid-hmacWithSHA1
/// ```
///
/// Note that modern usage should avoid the use of SHA-1, despite it being
/// the "default" here.
impl Default for Pbkdf2Prf {
    fn default() -> Self {
        Self::HmacWithSha1
    }
}

impl<'a> TryFrom<Any<'a>> for Pbkdf2Prf {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        AlgorithmIdentifier::try_from(any).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Pbkdf2Prf {
    type Error = Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Self> {
        if let Some(params) = alg.parameters {
            // TODO(tarcieri): support non-NULL parameters?
            if !params.is_null() {
                return Err(ErrorKind::Value { tag: params.tag() }.into());
            }
        } else {
            // TODO(tarcieri): support OPTIONAL parameters?
            return Err(ErrorKind::Truncated.into());
        }

        match alg.oid {
            HMAC_WITH_SHA1_OID => Ok(Self::HmacWithSha1),
            HMAC_WITH_SHA256_OID => Ok(Self::HmacWithSha256),
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}

impl<'a> From<Pbkdf2Prf> for AlgorithmIdentifier<'a> {
    fn from(prf: Pbkdf2Prf) -> Self {
        // TODO(tarcieri): support non-NULL parameters?
        let parameters = AlgorithmParameters::Null;

        AlgorithmIdentifier {
            oid: prf.oid(),
            parameters: Some(parameters),
        }
    }
}

impl Encodable for Pbkdf2Prf {
    fn encoded_len(&self) -> Result<Length> {
        AlgorithmIdentifier::try_from(*self)?.encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        AlgorithmIdentifier::try_from(*self)?.encode(encoder)
    }
}

/// Symmetric encryption scheme used by PBES2.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EncryptionScheme<'a> {
    /// AES-128 in CBC mode
    Aes128Cbc {
        /// Initialization vector
        iv: &'a [u8; AES_BLOCK_SIZE],
    },

    /// AES-256 in CBC mode
    Aes256Cbc {
        /// Initialization vector
        iv: &'a [u8; AES_BLOCK_SIZE],
    },
}

impl<'a> EncryptionScheme<'a> {
    /// Get the [`ObjectIdentifier`] (a.k.a OID) for this algorithm.
    pub fn oid(self) -> ObjectIdentifier {
        match self {
            Self::Aes128Cbc { .. } => AES_128_CBC_OID,
            Self::Aes256Cbc { .. } => AES_256_CBC_OID,
        }
    }
}

impl<'a> TryFrom<Any<'a>> for EncryptionScheme<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        AlgorithmIdentifier::try_from(any).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for EncryptionScheme<'a> {
    type Error = Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Self> {
        // TODO(tarcieri): support for non-AES algorithms?
        let iv = alg
            .parameters_any()?
            .octet_string()?
            .as_bytes()
            .try_into()
            .map_err(|_| ErrorKind::Value {
                tag: der::Tag::OctetString,
            })?;

        match alg.oid {
            AES_128_CBC_OID => Ok(Self::Aes128Cbc { iv }),
            AES_256_CBC_OID => Ok(Self::Aes256Cbc { iv }),
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}

impl<'a> TryFrom<EncryptionScheme<'a>> for AlgorithmIdentifier<'a> {
    type Error = Error;

    fn try_from(scheme: EncryptionScheme<'a>) -> Result<Self> {
        let parameters = match scheme {
            EncryptionScheme::Aes128Cbc { iv } => Any::from(OctetString::new(iv)?),
            EncryptionScheme::Aes256Cbc { iv } => Any::from(OctetString::new(iv)?),
        };

        Ok(AlgorithmIdentifier {
            oid: scheme.oid(),
            parameters: Some(parameters.try_into()?),
        })
    }
}

impl<'a> Encodable for EncryptionScheme<'a> {
    fn encoded_len(&self) -> Result<Length> {
        AlgorithmIdentifier::try_from(*self)?.encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        AlgorithmIdentifier::try_from(*self)?.encode(encoder)
    }
}
