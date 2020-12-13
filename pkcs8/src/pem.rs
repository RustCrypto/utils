//! PEM encoding support (RFC 7468)

use crate::{Error, Result};
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::str;
use subtle_encoding::base64;
use zeroize::Zeroizing;

/// Encapsulation boundaries
pub(crate) struct Boundary {
    /// Pre-encapsulation boundary
    pre: &'static str,

    /// Post-encapsulation boundary
    post: &'static str,
}

/// Private key encapsulation boundary
pub(crate) const PRIVATE_KEY_BOUNDARY: Boundary = Boundary {
    pre: "-----BEGIN PRIVATE KEY-----\n",
    post: "\n-----END PRIVATE KEY-----",
};

/// Public key encapsulation boundary
pub(crate) const PUBLIC_KEY_BOUNDARY: Boundary = Boundary {
    pre: "-----BEGIN PUBLIC KEY-----\n",
    post: "\n-----END PUBLIC KEY-----",
};

/// Parse "PEM encoding" as described in RFC 7468:
/// <https://tools.ietf.org/html/rfc7468>
///
/// Note that this decoder supports only a subset of the original
/// "Privacy Enhanced Mail" encoding as this parser specifically
/// implements a dialect intended for textual encodings of PKIX,
/// PKCS, and CMS structures.
// TODO(tarcieri): better harden for fully constant-time operation
pub(crate) fn parse(s: &str, boundary: Boundary) -> Result<Zeroizing<Vec<u8>>> {
    let s = s.trim_end();

    // TODO(tarcieri): handle missing newlines
    let s = s.strip_prefix(boundary.pre).ok_or(Error)?;
    let s = s.strip_suffix(boundary.post).ok_or(Error)?;

    // TODO(tarcieri): fix subtle-encoding to tolerate whitespace
    let mut s = Zeroizing::new(s.to_owned());
    s.retain(|c| !c.is_whitespace());

    base64::decode(&*s).map_err(|_| Error).map(Zeroizing::new)
}

/// Serialize "PEM encoding" as described in RFC 7468:
/// <https://tools.ietf.org/html/rfc7468>
pub(crate) fn serialize(data: &[u8], boundary: Boundary) -> Result<String> {
    let mut output = String::new();
    output.push_str(boundary.pre);

    let b64 = Zeroizing::new(base64::encode(data));
    let chunks = b64.chunks(64);
    let nchunks = chunks.len();

    for (i, chunk) in chunks.enumerate() {
        let line = str::from_utf8(chunk).expect("malformed base64");
        output.push_str(line);

        if i < nchunks.checked_sub(1).expect("unexpected chunks") {
            output.push('\n');
        }
    }

    output.push_str(boundary.post);
    Ok(output)
}
