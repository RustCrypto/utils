//! PKCS#8v2 `OneAsymmetricKey`.
// TODO(tarcieri): merge this into `PrivateKeyInfo` in the next breaking release.

use der::{Decodable, Encodable, Message};

use self::pubkey::PublicKeyBitString;
use crate::{AlgorithmIdentifier, Attributes, Error, Result, Version};
use core::{convert::TryFrom, fmt};

/// PKCS#8 `OneAsymmetricKey` as described in [RFC 5958 Section 2]:
///
/// ASN.1 structure containing a [`Version`], an [`AlgorithmIdentifier`],
/// private key data, and optionally public key data, in an algorithm-specific
/// format.
///
/// This structure can be thought of as an extension of
/// [`PrivateKeyInfo`][`crate::PrivateKeyInfo`] which includes an optional
/// public key.
///
/// Future releases of this crate will likely combine the two.
///
/// ```text
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
/// [RFC 5958 Section 2]: https://datatracker.ietf.org/doc/html/rfc5958#section-2
#[derive(Clone)]
pub struct OneAsymmetricKey<'a> {
    /// X.509 [`AlgorithmIdentifier`] for the private key type.
    pub algorithm: AlgorithmIdentifier<'a>,

    /// Private key data.
    pub private_key: &'a [u8],

    /// Attributes.
    pub attributes: Option<Attributes<'a>>,

    /// Public key data, optionally available if version is V2.
    pub public_key: Option<&'a [u8]>,
}

impl<'a> OneAsymmetricKey<'a> {
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
}

impl<'a> TryFrom<&'a [u8]> for OneAsymmetricKey<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_der(bytes)?)
    }
}

impl<'a> TryFrom<der::Any<'a>> for OneAsymmetricKey<'a> {
    type Error = der::Error;

    fn try_from(any: der::Any<'a>) -> der::Result<OneAsymmetricKey<'a>> {
        any.sequence(|decoder| {
            // Parse and validate `version` INTEGER.
            let version = Version::decode(decoder)?;
            let algorithm = decoder.decode()?;
            let private_key = decoder.octet_string()?.into();

            // TODO(tarcieri): handle extensions following attributes/public key.
            // See: https://datatracker.ietf.org/doc/html/rfc6025#section-2.4.1

            let attributes = decoder.optional()?;
            let public_key = if version == Version::V2 {
                decoder.context_specific_optional(1, |dec| {
                    dec.bit_string().map(|bit_string| bit_string.as_bytes())
                })?
            } else {
                None
            };

            Ok(Self {
                algorithm,
                private_key,
                attributes,
                public_key,
            })
        })
    }
}

impl<'a> Message<'a> for OneAsymmetricKey<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[
            &u8::from(self.version()),
            &self.algorithm,
            &der::OctetString::new(self.private_key)?,
            &self.attributes,
            &self
                .public_key
                .map(|pk| der::BitString::new(pk).map(PublicKeyBitString))
                .transpose()?,
        ])
    }
}

impl<'a> fmt::Debug for OneAsymmetricKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OneAsymmetricKey")
            .field("version", &self.version())
            .field("algorithm", &self.algorithm)
            .field("attributes", &self.attributes)
            .field("public_key", &self.public_key)
            .finish() // TODO: use `finish_non_exhaustive` when stable
    }
}

mod pubkey {
    use der::{BitString, Encodable, Encoder, Header, Length, Tag, Tagged};

    pub(super) struct PublicKeyBitString<'a>(pub BitString<'a>);

    impl<'a> Encodable for PublicKeyBitString<'a> {
        fn encoded_len(&self) -> der::Result<Length> {
            let inner_len = self.0.encoded_len()?;
            Header::new(Self::TAG, inner_len)?.encoded_len()? + inner_len
        }

        fn encode(&self, encoder: &mut Encoder<'_>) -> der::Result<()> {
            Header::new(Self::TAG, self.0.encoded_len()?)?.encode(encoder)?;

            self.0.encode(encoder)
        }
    }

    impl<'a> Tagged for PublicKeyBitString<'a> {
        const TAG: Tag = Tag::ContextSpecific1;
    }
}
