//! ASN.1 DER decoder.

use crate::{Error, Result};

/// Decoder type
// TODO(tarcieri): refactor into a struct
pub type Decoder<'a> = &'a [u8];

/// Extract the leading byte from a slice.
// TODO(tarcieri): refactor into a hypothetical `Decoder` struct
pub fn byte(bytes: &mut &[u8]) -> Result<u8> {
    let byte = *bytes.get(0).ok_or(Error::Truncated)?;
    *bytes = &bytes[1..];
    Ok(byte)
}
