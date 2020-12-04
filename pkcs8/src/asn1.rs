//! PKCS#8 parser

use crate::{AlgorithmIdentifier, Error, ObjectIdentifier, PrivateKeyInfo, Result};

/// ASN.1 `INTEGER` tag
const INTEGER_TAG: u8 = 0x02;

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
        len if len <= 0x80 => Ok(len as usize),
        0x81 => Ok(parse_byte(bytes)? as usize),
        0x82 => {
            let len_hi = parse_byte(bytes)? as usize;
            Ok((len_hi << 8) | (parse_byte(bytes)? as usize))
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
        algorithm,
        parameters,
    })
}

/// Parse a PKCS#8 document containing ASN.1 DER-encoded `PrivateKeyInfo`
pub(crate) fn parse_private_key_info<'a>(mut input: &'a [u8]) -> Result<PrivateKeyInfo<'a>> {
    // Parse outer SEQUENCE
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

    // Parse `privateKeyAlgorithm`
    let algorithm = parse_algorithm_identifier(&mut bytes)?;

    // Parse `privateKey`
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
