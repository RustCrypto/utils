//! PEM encoding support (RFC 7468)

use crate::{Error, Result};
use alloc::{borrow::ToOwned, vec::Vec};
use core::str::FromStr;
use subtle_encoding::base64;
use zeroize::Zeroizing;

/// Pre-encapsulation boundary
const PRE_ENCAPSULATION_BOUNDARY: &str = "-----BEGIN PRIVATE KEY-----\n";

/// Post-encapsulation boundary
const POST_ENCAPSULATION_BOUNDARY: &str = "\n-----END PRIVATE KEY-----";

/// PKCS#8 document decoded from PEM.
///
/// Note for embedded users: enabling the `pem` feature requires linking
/// with liballoc (i.e. this type is presently heap-backed).
// TODO(tarcieri): heapless support?
#[derive(Clone)]
pub struct Document(Zeroizing<Vec<u8>>);

impl AsRef<[u8]> for Document {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl FromStr for Document {
    type Err = Error;

    /// Parse "PEM encoding" as described in RFC 7468:
    /// <https://tools.ietf.org/html/rfc7468>
    ///
    /// Note that this decoder supports only a subset of the original
    /// "Privacy Enhanced Mail" encoding as this parser specifically
    /// implements a dialect intended for textual encodings of PKIX,
    /// PKCS, and CMS structures.
    // TODO(tarcieri): better harden for fully constant-time operation
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim_end();

        // TODO(tarcieri): handle missing newlines
        let s = s.strip_prefix(PRE_ENCAPSULATION_BOUNDARY).ok_or(Error)?;
        let s = s.strip_suffix(POST_ENCAPSULATION_BOUNDARY).ok_or(Error)?;

        // TODO(tarcieri): fix subtle-encoding to tolerate whitespace
        let mut s = Zeroizing::new(s.to_owned());
        s.retain(|c| !c.is_whitespace());

        base64::decode(&*s)
            .map(Zeroizing::new)
            .map(Document)
            .map_err(|_| Error)
    }
}
