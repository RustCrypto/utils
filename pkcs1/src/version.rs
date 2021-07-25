//! PKCS#1 version identifier.

use crate::Error;
use core::convert::{TryFrom, TryInto};
use der::{asn1::Any, Encodable, Encoder, Tag, Tagged};

/// Version identifier for PKCS#1 documents as defined in
/// [RFC 8017 Appendix 1.2].
///
/// > version is the version number, for compatibility with future
/// > revisions of this document.  It SHALL be 0 for this version of the
/// > document, unless multi-prime is used; in which case, it SHALL be 1.
///
/// ```text
/// Version ::= INTEGER { two-prime(0), multi(1) }
///    (CONSTRAINED BY
///    {-- version must be multi if otherPrimeInfos present --})
/// ```
///
/// [RFC 8017 Appendix 1.2]: https://datatracker.ietf.org/doc/html/rfc8017#appendix-A.1.2
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Version {
    /// Denotes a `two-prime` key
    TwoPrime = 0,

    /// Denotes a `multi` (i.e. multi-prime) key
    Multi = 1,
}

impl From<Version> for u8 {
    fn from(version: Version) -> Self {
        version as u8
    }
}

impl TryFrom<u8> for Version {
    type Error = Error;
    fn try_from(byte: u8) -> Result<Version, Error> {
        match byte {
            0 => Ok(Version::TwoPrime),
            1 => Ok(Version::Multi),
            _ => Err(Error::Version),
        }
    }
}

impl<'a> TryFrom<Any<'a>> for Version {
    type Error = der::Error;
    fn try_from(any: Any<'a>) -> der::Result<Version> {
        u8::try_from(any)?
            .try_into()
            .map_err(|_| der::ErrorKind::Value { tag: Tag::Integer }.into())
    }
}

impl Encodable for Version {
    fn encoded_len(&self) -> der::Result<der::Length> {
        der::Length::ONE.for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> der::Result<()> {
        u8::from(*self).encode(encoder)
    }
}

impl Tagged for Version {
    const TAG: Tag = Tag::Integer;
}
