//! PKCS#8 decoder: supports the subset of ASN.1 DER needed to parse PKCS#8.
//!
//! Note: per RFC 5208, PKCS#8 is technically BER encoded (despite what
//! things like the OpenSSL's `-outform DER` would lead you to believe).
//!
//! Despite that, this implementation presently attempts to reject BER-encoded
//! PKCS#8 keys that are not valid DER. This is because PKCS#8 keys are
//! typically described as "DER", and if it actually turns out to be a
//! legitimate practical problem the parsing rules can be relaxed.

use super::Tag;
use crate::{
    AlgorithmIdentifier, Error, ObjectIdentifier, PrivateKeyInfo, Result, SubjectPublicKeyInfo,
};

/// Parse a single byte from a slice
fn decode_byte(bytes: &mut &[u8]) -> Result<u8> {
    let byte = *bytes.get(0).ok_or(Error::Decode)?;
    *bytes = &bytes[1..];
    Ok(byte)
}

/// Parse DER-encoded length
fn decode_len(bytes: &mut &[u8]) -> Result<usize> {
    match decode_byte(bytes)? {
        // Note: per X.690 Section 8.1.3.6.1 the byte 0x80 encodes indefinite
        // lengths, which are not allowed in DER
        len if len < 0x80 => Ok(len as usize),
        0x81 => {
            let len = decode_byte(bytes)? as usize;

            // X.690 Section 10.1: DER lengths must be encoded with a minimum
            // number of octets
            if len >= 0x80 {
                Ok(len)
            } else {
                Err(Error::Decode)
            }
        }
        0x82 => {
            let len_hi = decode_byte(bytes)? as usize;
            let len = (len_hi << 8) | (decode_byte(bytes)? as usize);

            // X.690 Section 10.1: DER lengths must be encoded with a minimum
            // number of octets
            if len > 0xFF {
                Ok(len)
            } else {
                Err(Error::Decode)
            }
        }
        _ => {
            // We specialize to a maximum 3-byte length
            Err(Error::Decode)
        }
    }
}

/// Parse DER-encoded INTEGER
fn decode_integer(bytes: &mut &[u8]) -> Result<usize> {
    if decode_byte(bytes)? != Tag::Integer as u8 {
        return Err(Error::Decode);
    }

    // We presently specialize for 1-byte integers to parse versions
    if decode_len(bytes)? == 1 {
        Ok(decode_byte(bytes)? as usize)
    } else {
        Err(Error::Decode)
    }
}

/// Parse length-delimited data
fn decode_length_delimited<'a>(bytes: &mut &'a [u8], expected_tag: Tag) -> Result<&'a [u8]> {
    if decode_byte(bytes)? != expected_tag as u8 {
        return Err(Error::Decode);
    }

    let len = decode_len(bytes)?;

    if len <= bytes.len() {
        let (head, tail) = bytes.split_at(len);
        *bytes = tail;
        Ok(head)
    } else {
        Err(Error::Decode)
    }
}

/// Parse X.509 `AlgorithmIdentifier`.
///
/// Defined in RFC 5182 Section 4.1.1.2:
/// <https://tools.ietf.org/html/rfc5280#section-4.1.1.2>
///
/// ```text
/// AlgorithmIdentifier  ::=  SEQUENCE  {
///      algorithm               OBJECT IDENTIFIER,
///      parameters              ANY DEFINED BY algorithm OPTIONAL  }
/// ```
pub(crate) fn decode_algorithm_identifier(input: &mut &[u8]) -> Result<AlgorithmIdentifier> {
    let mut bytes = decode_length_delimited(input, Tag::Sequence)?;

    // Check OBJECT ID header
    if decode_byte(&mut bytes)? != Tag::ObjectIdentifier as u8 {
        return Err(Error::Decode);
    }

    let len = decode_len(&mut bytes)?;

    if len > bytes.len() {
        return Err(Error::Decode);
    }

    let (alg_bytes, mut param_bytes) = bytes.split_at(len);
    let algorithm = ObjectIdentifier::from_ber(alg_bytes)?;
    let tag = decode_byte(&mut param_bytes)?;

    let parameters = if tag == Tag::Null as u8 {
        if decode_len(&mut param_bytes)? != 0 {
            return Err(Error::Decode);
        }

        // Disallow any trailing data after the parameters
        if !param_bytes.is_empty() {
            return Err(Error::Decode);
        }

        None
    } else if tag == Tag::ObjectIdentifier as u8 {
        let len = decode_len(&mut param_bytes)?;

        if len != param_bytes.len() {
            return Err(Error::Decode);
        }

        Some(ObjectIdentifier::from_ber(param_bytes)?)
    } else {
        return Err(Error::Decode);
    };

    Ok(AlgorithmIdentifier {
        oid: algorithm,
        parameters,
    })
}

/// Parse a PKCS#8 document containing ASN.1 DER-encoded `PrivateKeyInfo`
pub(crate) fn decode_private_key_info(mut input: &[u8]) -> Result<PrivateKeyInfo<'_>> {
    let mut bytes = decode_length_delimited(&mut input, Tag::Sequence)?;

    if !input.is_empty() {
        return Err(Error::Decode);
    }

    // Parse `version` INTEGER
    let version = decode_integer(&mut bytes)?;

    // RFC 5208 designates `0` as the only valid version for PKCS#8 documents
    if version != 0 {
        return Err(Error::Decode);
    }

    let algorithm = decode_algorithm_identifier(&mut bytes)?;
    let private_key = decode_length_delimited(&mut bytes, Tag::OctetString)?;

    // We currently don't support any trailing attribute data
    if !bytes.is_empty() {
        return Err(Error::Decode);
    }

    Ok(PrivateKeyInfo {
        algorithm,
        private_key,
    })
}

/// Parse ASN.1 DER encoded `SubjectPublicKeyInfo`.
///
/// Note that this implementation assumes an outer `SEQUENCE` tag which is not
/// present in X.509. This is for the purpose of public key storage.
pub(crate) fn decode_spki(mut input: &[u8]) -> Result<SubjectPublicKeyInfo<'_>> {
    let mut bytes = decode_length_delimited(&mut input, Tag::Sequence)?;

    if !input.is_empty() {
        return Err(Error::Decode);
    }

    let algorithm = decode_algorithm_identifier(&mut bytes)?;
    let subject_public_key = decode_length_delimited(&mut bytes, Tag::BitString)?;

    if !bytes.is_empty() {
        return Err(Error::Decode);
    }

    Ok(SubjectPublicKeyInfo {
        algorithm,
        subject_public_key,
    })
}
