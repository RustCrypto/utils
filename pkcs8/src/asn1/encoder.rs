//! PKCS#8 encoder: supports the subset of ASN.1 DER needed to parse PKCS#8.
//!
//! Note: per RFC 5208, PKCS#8 is technically BER encoded (despite what
//! things like the OpenSSL's `-outform DER` would lead you to believe).
//!
//! Despite that, this implementation presently attempts to encode
//! PKCS#8 keys that are valid DER. This is because PKCS#8 keys are
//! typically described as "DER", and if it actually turns out to be a
//! legitimate practical problem the parsing rules can be relaxed.

use super::Tag;
use crate::{
    AlgorithmIdentifier, Error, ObjectIdentifier, PrivateKeyInfo, Result, SubjectPublicKeyInfo,
};

/// Encode a single byte at the given offset
fn encode_byte(buffer: &mut [u8], offset: usize, byte: u8) -> Result<()> {
    buffer.get_mut(offset).map(|b| *b = byte).ok_or(Error)
}

/// Encode length prefix
fn encode_len(buffer: &mut [u8], length: usize) -> Result<usize> {
    match length {
        0..=0x7F => {
            encode_byte(buffer, 0, length as u8)?;
            Ok(1)
        }
        0x80..=0xFF => {
            encode_byte(buffer, 0, 0x81)?;
            encode_byte(buffer, 1, length as u8)?;
            Ok(2)
        }
        0x100..=0xFFFF => {
            encode_byte(buffer, 0, 0x82)?;
            encode_byte(buffer, 1, (length >> 8) as u8)?;
            encode_byte(buffer, 2, (length & 0xFF) as u8)?;
            Ok(3)
        }
        _ => Err(Error),
    }
}

/// Compute the length of a header including the tag byte
fn header_len(body_len: usize) -> Result<usize> {
    match body_len {
        0..=0x7F => Ok(2),
        0x80..=0xFF => Ok(3),
        0x100..=0xFFFF => Ok(4),
        _ => Err(Error),
    }
}

/// Encode a tag and a length header
pub(crate) fn encode_header(buffer: &mut [u8], tag: Tag, length: usize) -> Result<usize> {
    encode_byte(buffer, 0, tag as u8)?;
    encode_len(&mut buffer[1..], length).and_then(|len| len.checked_add(1).ok_or(Error))
}

/// Encode length-delimited tagged data
pub(crate) fn encode_length_delimited(buffer: &mut [u8], tag: Tag, data: &[u8]) -> Result<usize> {
    let offset = encode_header(buffer, tag, data.len())?;

    if buffer[offset..].len() < data.len() {
        return Err(Error);
    }

    buffer[offset..(offset + data.len())].copy_from_slice(data);
    offset.checked_add(data.len()).ok_or(Error)
}

/// Get the length of a DER-encoded OID
pub(crate) fn oid_len(oid: ObjectIdentifier) -> Result<usize> {
    let body_len = oid.ber_len();
    header_len(body_len)?.checked_add(body_len).ok_or(Error)
}

/// Encode an OID
pub(crate) fn encode_oid(buffer: &mut [u8], oid: ObjectIdentifier) -> Result<usize> {
    let offset = encode_header(buffer, Tag::ObjectIdentifier, oid.ber_len())?;

    offset
        .checked_add(oid.write_ber(&mut buffer[offset..])?.len())
        .ok_or(Error)
}

/// Get the length of a DER-encoded [`AlgorithmIdentifier`]
pub(crate) fn algorithm_identifier_len(algorithm_id: &AlgorithmIdentifier) -> Result<usize> {
    let alg_oid_len = oid_len(algorithm_id.oid)?;
    let params_len = match algorithm_id.parameters {
        Some(p) => oid_len(p)?,
        None => 2, // OID or NULL (2-bytes)
    };
    let sequence_len = alg_oid_len.checked_add(params_len).unwrap();
    header_len(sequence_len).and_then(|n| n.checked_add(sequence_len).ok_or(Error))
}

/// Encode an [`AlgorithmIdentifier`]
pub(crate) fn encode_algorithm_identifier(
    buffer: &mut [u8],
    algorithm_id: &AlgorithmIdentifier,
) -> Result<usize> {
    let alg_oid_len = oid_len(algorithm_id.oid)?;
    let params_len = match algorithm_id.parameters {
        Some(p) => oid_len(p)?,
        None => 2, // OID or NULL (2-bytes)
    };
    let sequence_len = alg_oid_len.checked_add(params_len).unwrap();

    let mut offset = encode_header(buffer, Tag::Sequence, sequence_len)?;
    offset += encode_oid(&mut buffer[offset..], algorithm_id.oid)?;
    offset += match algorithm_id.parameters {
        Some(oid) => encode_oid(&mut buffer[offset..], oid)?,
        None => encode_header(&mut buffer[offset..], Tag::Null, 0)?,
    };

    Ok(offset)
}

/// Get the length of DER-encoded [`PrivateKeyInfo`]
#[cfg(feature = "alloc")]
pub(crate) fn private_key_info_len(private_key_info: &PrivateKeyInfo<'_>) -> Result<usize> {
    let alg_id_len = algorithm_identifier_len(&private_key_info.algorithm)?;
    let version_len = 3;
    let private_key_len = header_len(private_key_info.private_key.len())?
        .checked_add(private_key_info.private_key.len())
        .ok_or(Error)?;
    let sequence_len = alg_id_len
        .checked_add(version_len)
        .and_then(|len| len.checked_add(private_key_len))
        .ok_or(Error)?;
    header_len(sequence_len).and_then(|n| n.checked_add(sequence_len).ok_or(Error))
}

/// Encode [`PrivateKeyInfo`]
pub(crate) fn encode_private_key_info(
    buffer: &mut [u8],
    private_key_info: &PrivateKeyInfo<'_>,
) -> Result<usize> {
    let alg_id_len = algorithm_identifier_len(&private_key_info.algorithm)?;
    let version_len = 3;
    let private_key_len = header_len(private_key_info.private_key.len())?
        .checked_add(private_key_info.private_key.len())
        .ok_or(Error)?;
    let sequence_len = alg_id_len
        .checked_add(version_len)
        .and_then(|len| len.checked_add(private_key_len))
        .ok_or(Error)?;

    let mut offset = encode_header(buffer, Tag::Sequence, sequence_len)?;
    offset += encode_length_delimited(&mut buffer[offset..], Tag::Integer, &[0])?;
    offset += encode_algorithm_identifier(&mut buffer[offset..], &private_key_info.algorithm)?;
    offset += encode_length_delimited(
        &mut buffer[offset..],
        Tag::OctetString,
        private_key_info.private_key,
    )?;

    Ok(offset)
}

/// Get the length of DER-encoded [`SubjectPublicKeyInfo`]
#[cfg(feature = "alloc")]
pub(crate) fn spki_len(spki: &SubjectPublicKeyInfo<'_>) -> Result<usize> {
    let alg_id_len = algorithm_identifier_len(&spki.algorithm)?;
    let public_key_len = header_len(spki.subject_public_key.len())?
        .checked_add(spki.subject_public_key.len())
        .ok_or(Error)?;
    let sequence_len = alg_id_len.checked_add(public_key_len).ok_or(Error)?;
    header_len(sequence_len).and_then(|n| n.checked_add(sequence_len).ok_or(Error))
}

/// Encode [`SubjectPublicKeyInfo`]
pub(crate) fn encode_spki(buffer: &mut [u8], spki: &SubjectPublicKeyInfo<'_>) -> Result<usize> {
    let alg_id_len = algorithm_identifier_len(&spki.algorithm)?;
    let private_key_len = header_len(spki.subject_public_key.len())?
        .checked_add(spki.subject_public_key.len())
        .ok_or(Error)?;
    let sequence_len = alg_id_len.checked_add(private_key_len).ok_or(Error)?;

    let mut offset = encode_header(buffer, Tag::Sequence, sequence_len)?;
    offset += encode_algorithm_identifier(&mut buffer[offset..], &spki.algorithm)?;
    offset += encode_length_delimited(
        &mut buffer[offset..],
        Tag::BitString,
        spki.subject_public_key,
    )?;

    Ok(offset)
}
