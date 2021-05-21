use der::{BitString, ContextualTo1, Decodable, Encodable, Message};

use crate::{
    common::{Version, _AttributesStub},
    AlgorithmIdentifier, Error, Result,
};
use core::{convert::TryFrom, fmt};

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
    /// Version, can be set to [`Version::V1`] or [`Version::V2`]
    pub version: Version,

    /// X.509 [`AlgorithmIdentifier`] for the private key type
    pub algorithm: AlgorithmIdentifier<'a>,

    /// Private key data
    pub private_key: &'a [u8],

    /// Public key data, optionally available if version is v2
    pub public_key: Option<&'a [u8]>,
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

                    let ret = if let Some(ContextualTo1(pubkey)) =
                        decoder.decode::<Option<ContextualTo1<BitString<'a>>>>()?
                    {
                        Some(pubkey.as_bytes())
                    } else {
                        None
                    };

                    while let Some(_) = decoder.decode::<Option<ContextualTo1<BitString<'a>>>>()? {
                        // Throw away further public keys (for now)
                        // FIXME: the documentation says "...,",
                        //  meaning more fields of the same type can exist,
                        //  considering that the rest of the documentation isn't talking about "multiple public keys",
                        //  I assume it is okay to only get the first value, and ignore the rest.
                    }

                    ret
                }
            };

            Ok(Self {
                version,
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
            &Into::<u8>::into(self.version),
            &self.algorithm,
            &der::OctetString::new(self.private_key)?,
            &if let Some(key) = self.public_key {
                Some(ContextualTo1(der::BitString::new(key)?))
            } else {
                None
            },
        ])
    }
}

impl<'a> fmt::Debug for OneAsymmetricKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OneAsymmetricKey")
            .field("version", &self.version)
            .field("algorithm", &self.algorithm)
            .field("public_key", &self.public_key)
            .finish() // TODO: use `finish_non_exhaustive` when stable
    }
}
