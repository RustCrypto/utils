//! PKCS#8 parser: supports the subset of ASN.1 DER needed to parse PKCS#8.
//!
//! Note: per RFC 5208, PKCS#8 is technically BER encoded (despite what
//! things like the OpenSSL's `-outform DER` would lead you to believe).
//!
//! Despite that, this implementation presently attempts to reject BER-encoded
//! PKCS#8 keys that are not valid DER. This is because PKCS#8 keys are
//! typically described as "DER", and if it actually turns out to be a
//! legitimate practical problem the parsing rules can be relaxed.

use crate::{
    AlgorithmIdentifier, Error, ObjectIdentifier, PrivateKeyInfo, Result, SubjectPublicKeyInfo,
};

/// ASN.1 `INTEGER` tag
const INTEGER_TAG: u8 = 0x02;

/// ASN.1 `BIT STRING` tag
const BIT_STRING_TAG: u8 = 0x03;

/// ASN.1 `OCTET STRING` tag
const OCTET_STRING_TAG: u8 = 0x04;

/// ASN.1 `NULL` tag
const NULL_TAG: u8 = 0x05;

/// ASN.1 `OBJECT IDENTIFIER` tag
const OBJECT_IDENTIFIER_TAG: u8 = 0x06;

/// ASN.1 `SEQUENCE` tag
const SEQUENCE_TAG: u8 = 0x30;

/// Parse a single byte from a slice
fn parse_byte(bytes: &mut &[u8]) -> Result<u8> {
    let byte = *bytes.get(0).ok_or(Error)?;
    *bytes = &bytes[1..];
    Ok(byte)
}

/// Parse DER-encoded length
fn parse_len(bytes: &mut &[u8]) -> Result<usize> {
    match parse_byte(bytes)? {
        // Note: per X.690 Section 8.1.3.6.1 the byte 0x80 encodes indefinite
        // lengths, which are not allowed in DER
        len if len < 0x80 => Ok(len as usize),
        0x81 => {
            let len = parse_byte(bytes)? as usize;

            // X.690 Section 10.1: DER lengths must be encoded with a minimum
            // number of octets
            if len >= 0x80 {
                Ok(len)
            } else {
                Err(Error)
            }
        }
        0x82 => {
            let len_hi = parse_byte(bytes)? as usize;
            let len = (len_hi << 8) | (parse_byte(bytes)? as usize);

            // X.690 Section 10.1: DER lengths must be encoded with a minimum
            // number of octets
            if len > 0xFF {
                Ok(len)
            } else {
                Err(Error)
            }
        }
        _ => {
            // We specialize to a maximum 3-byte length
            Err(Error)
        }
    }
}

/// Parse DER-encoded INTEGER
fn parse_integer(bytes: &mut &[u8]) -> Result<usize> {
    if parse_byte(bytes)? != INTEGER_TAG {
        return Err(Error);
    }

    // We presently specialize for 1-byte integers to parse versions
    if parse_len(bytes)? == 1 {
        Ok(parse_byte(bytes)? as usize)
    } else {
        Err(Error)
    }
}

/// Parse length-delimited data
fn parse_length_delimited<'a>(bytes: &mut &'a [u8], expected_tag: u8) -> Result<&'a [u8]> {
    if parse_byte(bytes)? != expected_tag {
        return Err(Error);
    }

    let len = parse_len(bytes)?;

    if len <= bytes.len() {
        let (head, tail) = bytes.split_at(len);
        *bytes = tail;
        Ok(head)
    } else {
        Err(Error)
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
pub(crate) fn parse_algorithm_identifier(input: &mut &[u8]) -> Result<AlgorithmIdentifier> {
    let mut bytes = parse_length_delimited(input, SEQUENCE_TAG)?;

    // Check OBJECT ID header
    if parse_byte(&mut bytes)? != OBJECT_IDENTIFIER_TAG {
        return Err(Error);
    }

    let len = parse_len(&mut bytes)?;

    if len > bytes.len() {
        return Err(Error);
    }

    let (alg_bytes, mut param_bytes) = bytes.split_at(len);
    let algorithm = ObjectIdentifier::from_ber(alg_bytes)?;

    let parameters = match parse_byte(&mut param_bytes)? {
        NULL_TAG => {
            if parse_len(&mut param_bytes)? != 0 {
                return Err(Error);
            }

            // Disallow any trailing data after the parameters
            if !param_bytes.is_empty() {
                return Err(Error);
            }

            None
        }
        OBJECT_IDENTIFIER_TAG => {
            let len = parse_len(&mut param_bytes)?;

            if len != param_bytes.len() {
                return Err(Error);
            }

            Some(ObjectIdentifier::from_ber(param_bytes)?)
        }
        _ => return Err(Error),
    };

    Ok(AlgorithmIdentifier {
        oid: algorithm,
        parameters,
    })
}

/// Parse a PKCS#8 document containing ASN.1 DER-encoded `PrivateKeyInfo`
pub(crate) fn parse_private_key_info(mut input: &[u8]) -> Result<PrivateKeyInfo<'_>> {
    let mut bytes = parse_length_delimited(&mut input, SEQUENCE_TAG)?;

    if !input.is_empty() {
        return Err(Error);
    }

    // Parse `version` INTEGER
    let version = parse_integer(&mut bytes)?;

    // RFC 5208 designates `0` as the only valid version for PKCS#8 documents
    if version != 0 {
        return Err(Error);
    }

    let algorithm = parse_algorithm_identifier(&mut bytes)?;
    let private_key = parse_length_delimited(&mut bytes, OCTET_STRING_TAG)?;

    // We currently don't support any trailing attribute data
    if !bytes.is_empty() {
        return Err(Error);
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
pub(crate) fn parse_spki(mut input: &[u8]) -> Result<SubjectPublicKeyInfo<'_>> {
    let mut bytes = parse_length_delimited(&mut input, SEQUENCE_TAG)?;

    if !input.is_empty() {
        return Err(Error);
    }

    let algorithm = parse_algorithm_identifier(&mut bytes)?;
    let subject_public_key = parse_length_delimited(&mut bytes, BIT_STRING_TAG)?;

    if !bytes.is_empty() {
        return Err(Error);
    }

    Ok(SubjectPublicKeyInfo {
        algorithm,
        subject_public_key,
    })
}
