#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod decode;
#[cfg(feature = "alloc")]
pub use decode::parse_into_vec;
pub use decode::{Header, parse_into_array};

#[cfg(feature = "alloc")]
mod encode;
#[cfg(feature = "alloc")]
pub use encode::encode_blobs;

/// Error type used by `blobby` functions
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// Decoded VLQ number is too big
    InvalidVlq,
    /// Invalid de-duplicated blob index
    InvalidIndex,
    /// Unexpected end of data
    UnexpectedEnd,
    /// Not enough elements for `BlobNIterator`
    NotEnoughElements,
    /// Bad array length was provided to [`parse_as_array`]
    BadArrayLen,
}

const NEXT_MASK: u8 = 0b1000_0000;
const VAL_MASK: u8 = 0b0111_1111;
