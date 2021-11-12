//! Collection of custom reference types for code generic over in-place and
//! buffer-to-buffer modes of operation.

// #![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/8f1a9894/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/8f1a9894/logo.svg",
    html_root_url = "https://docs.rs/inout/0.1.0"
)]
#![allow(clippy::needless_lifetimes)]
#![warn(missing_docs, rust_2018_idioms)]

mod inout;
mod intmpout;
mod inout_buf;
mod intmpout_buf;

pub use inout::InOut;
pub use intmpout::InTmpOut;
pub use inout_buf::{InOutBuf, InOutBufIter, InSrc, IntoArrayError, NotEqualError};
pub use intmpout_buf::InTmpOutBuf;
