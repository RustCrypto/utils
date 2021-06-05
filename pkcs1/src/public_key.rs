//! RSA Public Keys.

use core::convert::TryFrom;
use der::{
    asn1::{Any, UIntBytes},
    Decodable, Encodable, Error, Message, Result,
};

/// RSA Public Keys as defined in [RFC 8017 Appendix 1.1].
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

impl<'a> TryFrom<&'a [u8]> for RsaPublicKey<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl<'a> TryFrom<Any<'a>> for RsaPublicKey<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<RsaPublicKey<'a>> {
        any.sequence(|decoder| {
            Ok(Self {
                modulus: decoder.decode()?,
                public_exponent: decoder.decode()?,
            })
        })
    }
}

impl<'a> Message<'a> for RsaPublicKey<'a> {
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        f(&[&self.modulus, &self.public_exponent])
    }
}
