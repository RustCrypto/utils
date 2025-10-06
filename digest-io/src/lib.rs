#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![warn(missing_debug_implementations)]

mod io_wrapper;
mod reader;
mod writer;

pub use io_wrapper::IoWrapper;
pub use reader::HashReader;
pub use writer::HashWriter;
