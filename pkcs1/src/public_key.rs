//! PKCS#1 RSA Public Keys.

use crate::{Error, Result};
use core::convert::TryFrom;
use der::{
    asn1::{Any, UIntBytes},
    Decodable, Encodable, Message,
};

#[cfg(feature = "alloc")]
use crate::RsaPublicKeyDocument;

#[cfg(feature = "pem")]
use {
    crate::{pem, LineEnding},
    alloc::string::String,
};

/// Type label for PEM-encoded private keys.
#[cfg(feature = "pem")]
pub(crate) const PEM_TYPE_LABEL: &str = "RSA PUBLIC KEY";

/// PKCS#1 RSA Public Keys as defined in [RFC 8017 Appendix 1.1].
///
/// ASN.1 structure containing a serialized RSA public key:
///
/// ```text
/// RSAPublicKey ::= SEQUENCE {
///     modulus           INTEGER,  -- n
///     publicExponent    INTEGER   -- e
/// }
/// ```
///
/// [RFC 8017 Appendix 1.1]: https://datatracker.ietf.org/doc/html/rfc8017#appendix-A.1.1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RsaPublicKey<'a> {
    /// `n`: RSA modulus
    pub modulus: UIntBytes<'a>,

    /// `e`: RSA public exponent
    pub public_exponent: UIntBytes<'a>,
}

impl<'a> RsaPublicKey<'a> {
    /// Encode this [`RsaPublicKey`] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(self) -> RsaPublicKeyDocument {
        self.into()
    }

    /// Encode this [`RsaPublicKey`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(self) -> Result<String> {
        self.to_pem_with_le(LineEnding::default())
    }

    /// Encode this [`RsaPublicKey`] as PEM-encoded ASN.1 DER with the given
    /// [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem_with_le(self, line_ending: LineEnding) -> Result<String> {
        Ok(pem::encode_string(
            PEM_TYPE_LABEL,
            line_ending,
            self.to_der().as_ref(),
        )?)
    }
}

impl<'a> TryFrom<&'a [u8]> for RsaPublicKey<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_der(bytes)?)
    }
}

impl<'a> TryFrom<Any<'a>> for RsaPublicKey<'a> {
    type Error = der::Error;

    fn try_from(any: Any<'a>) -> der::Result<RsaPublicKey<'a>> {
        any.sequence(|decoder| {
            Ok(Self {
                modulus: decoder.decode()?,
                public_exponent: decoder.decode()?,
            })
        })
    }
}

impl<'a> Message<'a> for RsaPublicKey<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[&self.modulus, &self.public_exponent])
    }
}
