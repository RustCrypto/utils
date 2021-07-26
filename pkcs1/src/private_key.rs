//! PKCS#1 RSA Private Keys.

use crate::{Error, Result, RsaPublicKey, Version};
use core::{convert::TryFrom, fmt};
use der::{
    asn1::{Any, UIntBytes},
    Decodable, Encodable, Message,
};

#[cfg(feature = "alloc")]
use crate::RsaPrivateKeyDocument;

#[cfg(feature = "pem")]
use {
    crate::{pem, LineEnding},
    alloc::string::String,
    zeroize::Zeroizing,
};

/// Type label for PEM-encoded private keys.
#[cfg(feature = "pem")]
pub(crate) const PEM_TYPE_LABEL: &str = "RSA PRIVATE KEY";

/// PKCS#1 RSA Private Keys as defined in [RFC 8017 Appendix 1.2].
///
/// ASN.1 structure containing a serialized RSA private key:
///
/// ```text
/// RSAPrivateKey ::= SEQUENCE {
///     version           Version,
///     modulus           INTEGER,  -- n
///     publicExponent    INTEGER,  -- e
///     privateExponent   INTEGER,  -- d
///     prime1            INTEGER,  -- p
///     prime2            INTEGER,  -- q
///     exponent1         INTEGER,  -- d mod (p-1)
///     exponent2         INTEGER,  -- d mod (q-1)
///     coefficient       INTEGER,  -- (inverse of q) mod p
///     otherPrimeInfos   OtherPrimeInfos OPTIONAL
/// }
/// ```
///
/// [RFC 8017 Appendix 1.2]: https://datatracker.ietf.org/doc/html/rfc8017#appendix-A.1.2
#[derive(Clone)]
pub struct RsaPrivateKey<'a> {
    /// Version number: `two-prime` or `multi`
    pub version: Version,

    /// `n`: RSA modulus
    pub modulus: UIntBytes<'a>,

    /// `e`: RSA public exponent
    pub public_exponent: UIntBytes<'a>,

    /// `d`: RSA private exponent
    pub private_exponent: UIntBytes<'a>,

    /// `p`: first prime factor of `n`
    pub prime1: UIntBytes<'a>,

    /// `q`: Second prime factor of `n`
    pub prime2: UIntBytes<'a>,

    /// First exponent: `d mod (p-1)`
    pub exponent1: UIntBytes<'a>,

    /// Second exponent: `d mod (q-1)`
    pub exponent2: UIntBytes<'a>,

    /// CRT coefficient: `(inverse of q) mod p`
    pub coefficient: UIntBytes<'a>,
}

impl<'a> RsaPrivateKey<'a> {
    /// Get the public key that corresponds to this [`RsaPrivateKey`].
    pub fn public_key(&self) -> RsaPublicKey<'a> {
        RsaPublicKey {
            modulus: self.modulus,
            public_exponent: self.public_exponent,
        }
    }

    /// Encode this [`RsaPrivateKey`] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> RsaPrivateKeyDocument {
        self.into()
    }

    /// Encode this [`RsaPrivateKey`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> Result<Zeroizing<String>> {
        self.to_pem_with_le(LineEnding::default())
    }

    /// Encode this [`RsaPrivateKey`] as PEM-encoded ASN.1 DER using the given
    /// [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem_with_le(&self, line_ending: LineEnding) -> Result<Zeroizing<String>> {
        let pem_doc = pem::encode_string(PEM_TYPE_LABEL, line_ending, self.to_der().as_ref())?;
        Ok(Zeroizing::new(pem_doc))
    }
}

impl<'a> From<RsaPrivateKey<'a>> for RsaPublicKey<'a> {
    fn from(private_key: RsaPrivateKey<'a>) -> RsaPublicKey<'a> {
        private_key.public_key()
    }
}

impl<'a> From<&RsaPrivateKey<'a>> for RsaPublicKey<'a> {
    fn from(private_key: &RsaPrivateKey<'a>) -> RsaPublicKey<'a> {
        private_key.public_key()
    }
}

impl<'a> TryFrom<&'a [u8]> for RsaPrivateKey<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_der(bytes)?)
    }
}

impl<'a> TryFrom<Any<'a>> for RsaPrivateKey<'a> {
    type Error = der::Error;

    fn try_from(any: Any<'a>) -> der::Result<RsaPrivateKey<'a>> {
        any.sequence(|decoder| {
            Ok(Self {
                version: decoder.decode()?,
                modulus: decoder.decode()?,
                public_exponent: decoder.decode()?,
                private_exponent: decoder.decode()?,
                prime1: decoder.decode()?,
                prime2: decoder.decode()?,
                exponent1: decoder.decode()?,
                exponent2: decoder.decode()?,
                coefficient: decoder.decode()?,
            })
        })
    }
}

impl<'a> Message<'a> for RsaPrivateKey<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[
            &self.version,
            &self.modulus,
            &self.public_exponent,
            &self.private_exponent,
            &self.prime1,
            &self.prime2,
            &self.exponent1,
            &self.exponent2,
            &self.coefficient,
        ])
    }
}

impl<'a> fmt::Debug for RsaPrivateKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RsaPrivateKey")
            .field("version", &self.version)
            .field("modulus", &self.modulus)
            .field("public_exponent", &self.public_exponent)
            .field("private_exponent", &"...")
            .field("prime1", &"...")
            .field("prime2", &"...")
            .field("exponent1", &"...")
            .field("exponent2", &"...")
            .field("coefficient", &"...")
            .finish() // TODO: use `finish_non_exhaustive` when stable
    }
}
