//! PKCS#8 `PrivateKeyInfo`.

use crate::{AlgorithmIdentifier, Attributes, Error, Result, Version};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};
use der::{
    asn1::{Any, BitString, ContextSpecific, OctetString},
    Decodable, Encodable, Message, TagNumber,
};

#[cfg(feature = "alloc")]
use crate::PrivateKeyDocument;

#[cfg(feature = "encryption")]
use {
    crate::EncryptedPrivateKeyDocument,
    rand_core::{CryptoRng, RngCore},
};

#[cfg(feature = "pem")]
use {
    crate::{error, pem, LineEnding},
    alloc::string::String,
    zeroize::Zeroizing,
};

/// Context-specific tag number for [`Attributes`].
const ATTRIBUTES_TAG: TagNumber = TagNumber::new(0);

/// Context-specific tag number for the public key.
const PUBLIC_KEY_TAG: TagNumber = TagNumber::new(1);

/// Type label for PEM-encoded private keys.
#[cfg(feature = "pem")]
pub(crate) const PEM_TYPE_LABEL: &str = "PRIVATE KEY";

/// PKCS#8 `PrivateKeyInfo`.
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`], private key
/// data in an algorithm specific format, and optional attributes.
///
/// Supports PKCS#8 v1 as described in [RFC 5208] and PKCS#8 v2 as described
/// in [RFC 5958]. PKCS#8 v2 keys include an additional public key field.
///
/// # PKCS#8 v1 `PrivateKeyInfo`
///
/// Described in [RFC 5208 Section 5]:
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
///
/// # PKCS#8 v2 `OneAsymmetricKey`
///
/// PKCS#8 `OneAsymmetricKey` as described in [RFC 5958 Section 2]:
///
/// ```text
/// PrivateKeyInfo ::= OneAsymmetricKey
///
/// OneAsymmetricKey ::= SEQUENCE {
///     version                   Version,
///     privateKeyAlgorithm       PrivateKeyAlgorithmIdentifier,
///     privateKey                PrivateKey,
///     attributes            [0] Attributes OPTIONAL,
///     ...,
///     [[2: publicKey        [1] PublicKey OPTIONAL ]],
///     ...
///   }
///
/// Version ::= INTEGER { v1(0), v2(1) } (v1, ..., v2)
///
/// PrivateKeyAlgorithmIdentifier ::= AlgorithmIdentifier
///
/// PrivateKey ::= OCTET STRING
///
/// Attributes ::= SET OF Attribute
///
/// PublicKey ::= BIT STRING
/// ```
///
/// [RFC 5208]: https://tools.ietf.org/html/rfc5208
/// [RFC 5958]: https://datatracker.ietf.org/doc/html/rfc5958
/// [RFC 5208 Section 5]: https://tools.ietf.org/html/rfc5208#section-5
/// [RFC 5958 Section 2]: https://datatracker.ietf.org/doc/html/rfc5958#section-2
#[derive(Clone)]
pub struct PrivateKeyInfo<'a> {
    /// X.509 [`AlgorithmIdentifier`] for the private key type.
    pub algorithm: AlgorithmIdentifier<'a>,

    /// Private key data.
    pub private_key: &'a [u8],

    /// Attributes.
    pub attributes: Option<Attributes<'a>>,

    /// Public key data, optionally available if version is V2.
    pub public_key: Option<&'a [u8]>,
}

impl<'a> PrivateKeyInfo<'a> {
    /// Create a new PKCS#8 [`PrivateKeyInfo`] message.
    ///
    /// This is a helper method which initializes `attributes` and `public_key`
    /// to `None`, helpful if you aren't using those.
    pub fn new(algorithm: AlgorithmIdentifier<'a>, private_key: &'a [u8]) -> Self {
        Self {
            algorithm,
            private_key,
            attributes: None,
            public_key: None,
        }
    }

    /// Get the PKCS#8 [`Version`] for this structure.
    ///
    /// [`Version::V1`] if `public_key` is `None`, [`Version::V2`] if `Some`.
    pub fn version(&self) -> Version {
        if self.public_key.is_some() {
            Version::V2
        } else {
            Version::V1
        }
    }

    /// Encrypt this private key using a symmetric encryption key derived
    /// from the provided password.
    ///
    /// See [`PrivateKeyDocument::encrypt`] for more information.
    #[cfg(feature = "encryption")]
    #[cfg_attr(docsrs, doc(cfg(feature = "encryption")))]
    pub fn encrypt(
        &self,
        rng: impl CryptoRng + RngCore,
        password: impl AsRef<[u8]>,
    ) -> Result<EncryptedPrivateKeyDocument> {
        PrivateKeyDocument::from(self).encrypt(rng, password)
    }

    /// Encode this [`PrivateKeyInfo`] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PrivateKeyDocument {
        self.into()
    }

    /// Encode this [`PrivateKeyInfo`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> Zeroizing<String> {
        self.to_pem_with_le(LineEnding::default())
    }

    /// Encode this [`PrivateKeyInfo`] as PEM-encoded ASN.1 DER with the given
    /// [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem_with_le(&self, line_ending: LineEnding) -> Zeroizing<String> {
        Zeroizing::new(
            pem::encode_string(PEM_TYPE_LABEL, line_ending, self.to_der().as_ref())
                .expect(error::PEM_ENCODING_MSG),
        )
    }
}

impl<'a> TryFrom<&'a [u8]> for PrivateKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_der(bytes)?)
    }
}

impl<'a> TryFrom<Any<'a>> for PrivateKeyInfo<'a> {
    type Error = der::Error;

    fn try_from(any: Any<'a>) -> der::Result<PrivateKeyInfo<'a>> {
        any.sequence(|decoder| {
            // Parse and validate `version` INTEGER.
            let version = Version::decode(decoder)?;
            let algorithm = decoder.decode()?;
            let private_key = decoder.octet_string()?.into();

            let attributes = decoder
                .context_specific(ATTRIBUTES_TAG)?
                .map(TryInto::try_into)
                .transpose()?;

            let public_key = decoder
                .context_specific(PUBLIC_KEY_TAG)?
                .map(|any| any.bit_string())
                .transpose()?
                .map(|bs| bs.as_bytes());

            if version.has_public_key() != public_key.is_some() {
                return Err(decoder.value_error(der::Tag::ContextSpecific(PUBLIC_KEY_TAG)));
            }

            // Ignore any remaining extension fields
            while !decoder.is_finished() {
                decoder.decode::<ContextSpecific<'_>>()?;
            }

            Ok(Self {
                algorithm,
                private_key,
                attributes,
                public_key,
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
            &u8::from(self.version()),
            &self.algorithm,
            &OctetString::new(self.private_key)?,
            &self.attributes.map(|value| ContextSpecific {
                tag_number: ATTRIBUTES_TAG,
                value: value.into(),
            }),
            &self
                .public_key
                .map(|pk| {
                    BitString::new(pk).map(|value| ContextSpecific {
                        tag_number: PUBLIC_KEY_TAG,
                        value: value.into(),
                    })
                })
                .transpose()?,
        ])
    }
}

impl<'a> fmt::Debug for PrivateKeyInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKeyInfo")
            .field("version", &self.version())
            .field("algorithm", &self.algorithm)
            .field("attributes", &self.attributes)
            .field("public_key", &self.public_key)
            .finish() // TODO: use `finish_non_exhaustive` when stable
    }
}
