//! Pure Rust implementation of the X.509 Public Key Infrastructure Certificate
//! format as described in [RFC 5280].
//!
//! [RFC 5280]: https://datatracker.ietf.org/doc/html/rfc5280

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/x509/0.0.1"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod attribute;
mod rdn;
mod time;

pub use crate::{attribute::AttributeTypeAndValue, rdn::RelativeDistinguishedName, time::Time};
pub use der::{self, asn1::ObjectIdentifier};
pub use spki::{self, AlgorithmIdentifier, SubjectPublicKeyInfo};

use alloc::collections::BTreeSet as Set;
