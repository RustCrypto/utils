//! PEM encoding support (RFC 7468)

use crate::{Error, Result};
use alloc::{borrow::ToOwned, vec::Vec};
use subtle_encoding::base64;
use zeroize::Zeroizing;

/// Private key pre-encapsulation boundary
pub(crate) const BEGIN_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\n";

/// Private key post-encapsulation boundary
pub(crate) const END_PRIVATE_KEY: &str = "\n-----END PRIVATE KEY-----";

/// Public key pre-encapsulation boundary
pub(crate) const BEGIN_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\n";

/// Public key post-encapsulation boundary
pub(crate) const END_PUBLIC_KEY: &str = "\n-----END PUBLIC KEY-----";

/// Parse "PEM encoding" as described in RFC 7468:
/// <https://tools.ietf.org/html/rfc7468>
///
/// Note that this decoder supports only a subset of the original
/// "Privacy Enhanced Mail" encoding as this parser specifically
/// implements a dialect intended for textual encodings of PKIX,
/// PKCS, and CMS structures.
// TODO(tarcieri): better harden for fully constant-time operation
pub(crate) fn parse(s: &str, begin: &str, end: &str) -> Result<Zeroizing<Vec<u8>>> {
    let s = s.trim_end();

    // TODO(tarcieri): handle missing newlines
    let s = s.strip_prefix(begin).ok_or(Error)?;
    let s = s.strip_suffix(end).ok_or(Error)?;

    // TODO(tarcieri): fix subtle-encoding to tolerate whitespace
    let mut s = Zeroizing::new(s.to_owned());
    s.retain(|c| !c.is_whitespace());

    base64::decode(&*s).map_err(|_| Error).map(Zeroizing::new)
}
