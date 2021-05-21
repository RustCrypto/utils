use der::{ContextualTo1, Decodable, Encodable, Message};

use crate::{attributes::_AttributesStub, version::Version, AlgorithmIdentifier, Error, Result};
use core::{convert::TryFrom, fmt};

mod pubkey {
    use der::{BitString, Encodable, Encoder, Header, Length, Tag, Tagged};

    // wow, java much?
    pub(super) struct EncodableContextSpecificPublicKey<'a>(pub BitString<'a>);

    impl<'a> Encodable for EncodableContextSpecificPublicKey<'a> {
        fn encoded_len(&self) -> der::Result<Length> {
            let inner_len = self.0.encoded_len()?;
            Header::new(Self::TAG, inner_len)?.encoded_len()? + inner_len
        }

        fn encode(&self, encoder: &mut Encoder<'_>) -> der::Result<()> {
            Header::new(Self::TAG, self.0.encoded_len()?)?.encode(encoder)?;

            self.0.encode(encoder)
        }
    }

    impl<'a> Tagged for EncodableContextSpecificPublicKey<'a> {
        const TAG: Tag = Tag::ContextSpecific1;
    }
}

/// PKCS#8 `OneAsymmetricKey`.
///
/// ASN.1 structure containing a [`Version`], an [`AlgorithmIdentifier`], private key
/// data, and optionally public key data, in an algorithm-specific format.
///
/// Described in [RFC 5958 Section 2]:
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
    /// X.509 [`AlgorithmIdentifier`] for the private key type
    pub algorithm: AlgorithmIdentifier<'a>,

    /// Private key data
    pub private_key: &'a [u8],

    /// Public key data, optionally available if version is v2
    pub public_key: Option<&'a [u8]>,
}

impl<'a> OneAsymmetricKey<'a> {
    /// Gets the dynamic version this document would have.
    ///
    /// [`Version::V1`] if `public_key` is `None`, [`Version::V2`] if `Some`.
    pub fn version(&self) -> Version {
        if let Some(_) = self.public_key {
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

            let public_key: Option<&[u8]> = match &version {
                Version::V1 => {
                    // run once, throw away an Attributes field (for now)
                    // TODO: Properly process and store attributes
                    decoder.decode::<Option<_AttributesStub>>()?;

                    None
                }
                Version::V2 => {
                    while let Some(_) = decoder.decode::<Option<_AttributesStub>>()? {
                        // Throw away all Attributes (for now)
                        // TODO: Properly process and store attributes
                    }

                    let mut ret: Option<&[u8]> = None;

                    while let Some(pk) =
                        decoder.context_specific_optional(1, |dec| dec.bit_string())?
                    {
                        // Throw away further public keys (for now)
                        // FIXME: the documentation says "...,",
                        //  meaning more fields of the same type can exist,
                        //  considering that the rest of the documentation isn't talking about "multiple public keys",
                        //  I assume it is okay to only get the last value, and ignore the rest.

                        ret.get_or_insert(pk.as_bytes());
                    }

                    ret
                }
            };

            Ok(Self {
                algorithm,
                private_key,
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
            &if let Some(key) = self.public_key {
                Some(pubkey::EncodableContextSpecificPublicKey(
                    der::BitString::new(key)?,
                ))
            } else {
                None
            },
        ])
    }
}

impl<'a> fmt::Debug for OneAsymmetricKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OneAsymmetricKey")
            .field("version", &self.version())
            .field("algorithm", &self.algorithm)
            .field("public_key", &self.public_key)
            .finish() // TODO: use `finish_non_exhaustive` when stable
    }
}
