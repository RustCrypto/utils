//! Key derivation functions.

use crate::{AlgorithmIdentifier, CryptoError, Error};
use core::convert::{TryFrom, TryInto};
use der::{
    asn1::{Any, ObjectIdentifier, OctetString},
    Encodable, Encoder, ErrorKind, Length, Message,
};

/// Password-Based Key Derivation Function (PBKDF2) OID.
pub const PBKDF2_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.1.5.12");

/// HMAC-SHA1 (for use with PBKDF2)
pub const HMAC_WITH_SHA1_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.2.7");

/// HMAC-SHA-224 (for use with PBKDF2)
pub const HMAC_WITH_SHA224_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.2.8");

/// HMAC-SHA-256 (for use with PBKDF2)
pub const HMAC_WITH_SHA256_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.2.9");

/// HMAC-SHA-384 (for use with PBKDF2)
pub const HMAC_WITH_SHA384_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.2.10");

/// HMAC-SHA-512 (for use with PBKDF2)
pub const HMAC_WITH_SHA512_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.2.11");

/// `id-scrypt` ([RFC 7914])
///
/// [RFC 7914]: https://datatracker.ietf.org/doc/html/rfc7914#section-7
pub const SCRYPT_OID: ObjectIdentifier = ObjectIdentifier::new("1.3.6.1.4.1.11591.4.11");

/// Type used for expressing scrypt cost
type ScryptCost = u16;

/// Password-based key derivation function.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Kdf<'a> {
    /// Password-Based Key Derivation Function 2 (PBKDF2).
    Pbkdf2(Pbkdf2Params<'a>),

    /// scrypt sequential memory-hard password hashing function.
    Scrypt(ScryptParams<'a>),
}

impl<'a> Kdf<'a> {
    /// Get the [`ObjectIdentifier`] (a.k.a OID) for this algorithm.
    pub fn oid(&self) -> ObjectIdentifier {
        match self {
            Self::Pbkdf2(_) => PBKDF2_OID,
            Self::Scrypt(_) => SCRYPT_OID,
        }
    }

    /// Get [`Pbkdf2Params`] if it is the selected algorithm.
    pub fn pbkdf2(&self) -> Option<&Pbkdf2Params<'a>> {
        match self {
            Self::Pbkdf2(params) => Some(params),
            _ => None,
        }
    }

    /// Get [`ScryptParams`] if it is the selected algorithm.
    pub fn scrypt(&self) -> Option<&ScryptParams<'a>> {
        match self {
            Self::Scrypt(params) => Some(params),
            _ => None,
        }
    }

    /// Is the selected KDF PBKDF2?
    pub fn is_pbkdf2(&self) -> bool {
        self.pbkdf2().is_some()
    }

    /// Is the selected KDF scrypt?
    pub fn is_scrypt(&self) -> bool {
        self.scrypt().is_some()
    }
}

impl<'a> From<Pbkdf2Params<'a>> for Kdf<'a> {
    fn from(params: Pbkdf2Params<'a>) -> Self {
        Kdf::Pbkdf2(params)
    }
}

impl<'a> From<ScryptParams<'a>> for Kdf<'a> {
    fn from(params: ScryptParams<'a>) -> Self {
        Kdf::Scrypt(params)
    }
}

impl<'a> TryFrom<Any<'a>> for Kdf<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> der::Result<Self> {
        AlgorithmIdentifier::try_from(any).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Kdf<'a> {
    type Error = Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> der::Result<Self> {
        match alg.oid {
            PBKDF2_OID => alg
                .parameters_any()
                .and_then(TryFrom::try_from)
                .map(Self::Pbkdf2),
            SCRYPT_OID => alg
                .parameters_any()
                .and_then(TryFrom::try_from)
                .map(Self::Scrypt),
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}

impl<'a> Message<'a> for Kdf<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        match self {
            Self::Pbkdf2(params) => f(&[&self.oid(), params]),
            Self::Scrypt(params) => f(&[&self.oid(), params]),
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
    pub key_length: Option<u16>,

    /// Pseudo-random function to use with PBKDF2
    pub prf: Pbkdf2Prf,
}

impl<'a> Pbkdf2Params<'a> {
    /// Initialize PBKDF2-SHA256 with the given iteration count and salt
    pub fn hmac_with_sha256(iteration_count: u16, salt: &'a [u8]) -> Result<Self, CryptoError> {
        Ok(Self {
            salt,
            iteration_count,
            key_length: None,
            prf: Pbkdf2Prf::HmacWithSha256,
        })
    }
}

impl<'a> TryFrom<Any<'a>> for Pbkdf2Params<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> der::Result<Self> {
        any.sequence(|params| {
            // TODO(tarcieri): support salt `CHOICE` w\ `AlgorithmIdentifier`
            let salt = params.octet_string()?;
            let iteration_count = params.decode()?;
            let key_length = params.optional()?;
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
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
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

    /// HMAC with SHA-224
    HmacWithSha224,

    /// HMAC with SHA-256
    HmacWithSha256,

    /// HMAC with SHA-384
    HmacWithSha384,

    /// HMAC with SHA-512
    HmacWithSha512,
}

impl Pbkdf2Prf {
    /// Get the [`ObjectIdentifier`] (a.k.a OID) for this algorithm.
    pub fn oid(self) -> ObjectIdentifier {
        match self {
            Self::HmacWithSha1 => HMAC_WITH_SHA1_OID,
            Self::HmacWithSha224 => HMAC_WITH_SHA224_OID,
            Self::HmacWithSha256 => HMAC_WITH_SHA256_OID,
            Self::HmacWithSha384 => HMAC_WITH_SHA384_OID,
            Self::HmacWithSha512 => HMAC_WITH_SHA512_OID,
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

    fn try_from(any: Any<'a>) -> der::Result<Self> {
        AlgorithmIdentifier::try_from(any).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Pbkdf2Prf {
    type Error = Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> der::Result<Self> {
        if let Some(params) = alg.parameters {
            // TODO(tarcieri): support non-NULL parameters?
            if !params.is_null() {
                return Err(params.tag().value_error());
            }
        } else {
            // TODO(tarcieri): support OPTIONAL parameters?
            return Err(ErrorKind::Truncated.into());
        }

        match alg.oid {
            HMAC_WITH_SHA1_OID => Ok(Self::HmacWithSha1),
            HMAC_WITH_SHA224_OID => Ok(Self::HmacWithSha224),
            HMAC_WITH_SHA256_OID => Ok(Self::HmacWithSha256),
            HMAC_WITH_SHA384_OID => Ok(Self::HmacWithSha384),
            HMAC_WITH_SHA512_OID => Ok(Self::HmacWithSha512),
            oid => Err(ErrorKind::UnknownOid { oid }.into()),
        }
    }
}

impl<'a> From<Pbkdf2Prf> for AlgorithmIdentifier<'a> {
    fn from(prf: Pbkdf2Prf) -> Self {
        // TODO(tarcieri): support non-NULL parameters?
        let parameters = der::asn1::Null;

        AlgorithmIdentifier {
            oid: prf.oid(),
            parameters: Some(parameters.into()),
        }
    }
}

impl Encodable for Pbkdf2Prf {
    fn encoded_len(&self) -> der::Result<Length> {
        AlgorithmIdentifier::try_from(*self)?.encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> der::Result<()> {
        AlgorithmIdentifier::try_from(*self)?.encode(encoder)
    }
}

/// scrypt parameters as defined in [RFC 7914 Section 7.1].
///
/// ```text
/// scrypt-params ::= SEQUENCE {
///     salt OCTET STRING,
///     costParameter INTEGER (1..MAX),
///     blockSize INTEGER (1..MAX),
///     parallelizationParameter INTEGER (1..MAX),
///     keyLength INTEGER (1..MAX) OPTIONAL
/// }
/// ```
///
/// [RFC 7914 Section 7.1]: https://datatracker.ietf.org/doc/html/rfc7914#section-7.1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScryptParams<'a> {
    /// scrypt salt
    pub salt: &'a [u8],

    /// CPU/Memory cost parameter `N`.
    pub cost_parameter: ScryptCost,

    /// Block size parameter `r`.
    pub block_size: u16,

    /// Parallelization parameter `p`.
    pub parallelization: u16,

    /// PBKDF2 output length
    pub key_length: Option<u16>,
}

impl<'a> ScryptParams<'a> {
    /// Get the [`ScryptParams`] for the provided upstream [`scrypt::Params`]
    /// and a provided salt string.
    #[cfg(feature = "scrypt")]
    #[cfg_attr(docsrs, doc(cfg(feature = "scrypt")))]
    pub fn from_params_and_salt(
        params: scrypt::Params,
        salt: &'a [u8],
    ) -> Result<Self, CryptoError> {
        Ok(Self {
            salt,
            cost_parameter: 1 << params.log_n(),
            block_size: params.r().try_into().map_err(|_| CryptoError)?,
            parallelization: params.p().try_into().map_err(|_| CryptoError)?,
            key_length: None,
        })
    }
}

impl<'a> TryFrom<Any<'a>> for ScryptParams<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> der::Result<Self> {
        any.sequence(|params| {
            let salt = params.octet_string()?;
            let cost_parameter = params.decode()?;
            let block_size = params.decode()?;
            let parallelization = params.decode()?;
            let key_length = params.optional()?;

            Ok(Self {
                salt: salt.as_bytes(),
                cost_parameter,
                block_size,
                parallelization,
                key_length,
            })
        })
    }
}

impl<'a> Message<'a> for ScryptParams<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[
            &OctetString::new(self.salt)?,
            &self.cost_parameter,
            &self.block_size,
            &self.parallelization,
            &self.key_length,
        ])
    }
}

#[cfg(feature = "scrypt")]
#[cfg_attr(docsrs, doc(cfg(feature = "scrypt")))]
impl<'a> TryFrom<ScryptParams<'a>> for scrypt::Params {
    type Error = CryptoError;

    fn try_from(params: ScryptParams<'a>) -> Result<scrypt::Params, CryptoError> {
        scrypt::Params::try_from(&params)
    }
}

#[cfg(feature = "scrypt")]
#[cfg_attr(docsrs, doc(cfg(feature = "scrypt")))]
impl<'a> TryFrom<&ScryptParams<'a>> for scrypt::Params {
    type Error = CryptoError;

    fn try_from(params: &ScryptParams<'a>) -> Result<scrypt::Params, CryptoError> {
        let n = params.cost_parameter;

        // Compute log2 and verify its correctness
        let log_n = ((8 * core::mem::size_of::<ScryptCost>() as u32) - n.leading_zeros() - 1) as u8;

        if 1 << log_n != n {
            return Err(CryptoError);
        }

        scrypt::Params::new(
            log_n,
            params.block_size.into(),
            params.parallelization.into(),
        )
        .map_err(|_| CryptoError)
    }
}
