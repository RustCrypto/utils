//! PEM encoding support (RFC 7468)

use crate::{Document, Error, Result};
use alloc::borrow::ToOwned;
use subtle_encoding::base64;
use zeroize::Zeroizing;

/// Pre-encapsulation boundary
const PRE_ENCAPSULATION_BOUNDARY: &str = "-----BEGIN PRIVATE KEY-----\n";

/// Post-encapsulation boundary
const POST_ENCAPSULATION_BOUNDARY: &str = "\n-----END PRIVATE KEY-----";

/// Parse "PEM encoding" as described in RFC 7468:
/// <https://tools.ietf.org/html/rfc7468>
///
/// Note that this decoder supports only a subset of the original
/// "Privacy Enhanced Mail" encoding as this parser specifically
/// implements a dialect intended for textual encodings of PKIX,
/// PKCS, and CMS structures.
// TODO(tarcieri): better harden for fully constant-time operation
pub(crate) fn parse(s: &str) -> Result<Document> {
    let s = s.trim_end();

    // TODO(tarcieri): handle missing newlines
    let s = s.strip_prefix(PRE_ENCAPSULATION_BOUNDARY).ok_or(Error)?;
    let s = s.strip_suffix(POST_ENCAPSULATION_BOUNDARY).ok_or(Error)?;

    // TODO(tarcieri): fix subtle-encoding to tolerate whitespace
    let mut s = Zeroizing::new(s.to_owned());
    s.retain(|c| !c.is_whitespace());

    let pkcs8_der = base64::decode(&*s).map_err(|_| Error).map(Zeroizing::new)?;
    Document::from_der(&*pkcs8_der)
}
