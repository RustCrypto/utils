//! PKCS#8 version identifier.

use core::convert::{TryFrom, TryInto};
use der::{asn1::Any, Encodable, Encoder, Tag, Tagged};

use crate::Error;

/// Version identifier for PKCS#8 documents.
///
/// (RFC 5958 designates `0` and `1` as the only valid versions for PKCS#8 documents)
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Version {
    /// Denotes PKCS#8 v1: no public key field.
    V1 = 0,

    /// Denotes PKCS#8 v2: `OneAsymmetricKey` with public key field.
    V2 = 1,
}

impl Version {
    /// Is this version expected to have a public key?
    pub fn has_public_key(self) -> bool {
        match self {
            Version::V1 => false,
            Version::V2 => true,
        }
    }
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
            0 => Ok(Version::V1),
            1 => Ok(Version::V2),
            _ => Err(Self::TAG.value_error().into()),
        }
    }
}

impl<'a> TryFrom<Any<'a>> for Version {
    type Error = der::Error;
    fn try_from(any: Any<'a>) -> der::Result<Version> {
        u8::try_from(any)?
            .try_into()
            .map_err(|_| Self::TAG.value_error())
    }
}

impl Encodable for Version {
    fn encoded_len(&self) -> der::Result<der::Length> {
        der::Length::from(1u8).for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> der::Result<()> {
        u8::from(*self).encode(encoder)
    }
}

impl Tagged for Version {
    const TAG: Tag = Tag::Integer;
}
