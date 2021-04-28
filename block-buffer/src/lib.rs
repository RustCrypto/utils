//! Fixed size buffer for block processing of data.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/block-buffer/0.10.0-pre.4"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "block-padding")]
pub use block_padding;
pub use generic_array;

use core::fmt;
use generic_array::{ArrayLength, GenericArray};

mod buffer;
mod lazy;
mod utils;

pub use buffer::BlockBuffer;
pub use lazy::LazyBlockBuffer;

/// Block on which a `BlockBuffer` operates.
pub type Block<BlockSize> = GenericArray<u8, BlockSize>;
/// Blocks being acted over in parallel.
pub type ParBlock<BlockSize, ParBlocks> = GenericArray<Block<BlockSize>, ParBlocks>;

/// Trait which generalizes digest functionality of buffers.
pub trait DigestBuffer<BlockSize: ArrayLength<u8>>: Default {
    /// Digest data in `input` in blocks of size `BlockSize` using
    /// the `compress` function, which accepts slice of blocks.
    fn digest_blocks(&mut self, input: &[u8], compress: impl FnMut(&[Block<BlockSize>]));

    /// Reset buffer by setting cursor position to zero.
    fn reset(&mut self);
}

/// Error type used by the [`BlockBuffer::block_mode_processing`] and
/// [`LazyBlockBuffer::block_mode_processing`] methods.
#[derive(Copy, Clone, Debug)]
pub struct InvalidLength;

impl fmt::Display for InvalidLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("Invalid Length")
    }
}
