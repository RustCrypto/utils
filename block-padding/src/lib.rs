//! Padding and unpadding of messages divided into blocks.
//!
//! This crate provides `Padding` trait which provides padding and unpadding
//! operations. Additionally several common padding schemes are available out
//! of the box.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_root_url = "https://docs.rs/block-padding/0.3.2"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;
pub use generic_array;
use generic_array::{ArrayLength, GenericArray};

/// Padding types
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PadType {
    /// Reversible padding
    Reversible,
    /// Ambiguous padding
    Ambiguous,
    /// No padding, message must be mutliple of block size
    NoPadding,
}

/// Block size.
pub type Block<B> = GenericArray<u8, B>;

/// Trait for padding messages divided into blocks
pub trait Padding<BlockSize: ArrayLength<u8>> {
    /// Padding type
    const TYPE: PadType;

    /// Pads `block` filled with data up to `pos` (i.e length of a message
    /// stored in the block is equal to `pos`).
    ///
    /// # Panics
    /// If `pos` is bigger than `BlockSize`. Most paddin algorithms also
    /// panic if they are equal.
    fn pad(block: &mut Block<BlockSize>, pos: usize);

    /// Unpad data in the `block`.
    ///
    /// Returns `Err(UnpadError)` if the block containts malformed padding.
    fn unpad(block: &Block<BlockSize>) -> Result<&[u8], UnpadError>;

    /// Unpad data in the `blocks`.
    ///
    /// Returns `Err(UnpadError)` if the block containts malformed padding.
    fn unpad_blocks(blocks: &[Block<BlockSize>]) -> Result<&[u8], UnpadError> {
        let bs = BlockSize::USIZE;
        let res_len = match (blocks.last(), Self::TYPE) {
            (_, PadType::NoPadding) => bs * blocks.len(),
            (Some(last_block), _) => {
                let n = Self::unpad(last_block)?.len();
                assert!(n <= bs);
                n + bs * (blocks.len() - 1)
            }
            (None, PadType::Ambiguous) => 0,
            (None, PadType::Reversible) => return Err(UnpadError),
        };
        // SAFETY: `res_len` is always smaller or equal to `bs * blocks.len()`
        Ok(unsafe {
            let p = blocks.as_ptr() as *const u8;
            core::slice::from_raw_parts(p, res_len)
        })
    }
}

/// Pad block with zeros.
///
/// ```
/// use block_padding::{ZeroPadding, Padding};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// ZeroPadding::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x00\x00\x00\x00");
/// let res = ZeroPadding::unpad(&mut block).unwrap();
/// assert_eq!(res, msg);
/// ```
///
/// Note that zero padding is not reversible for messages which end
/// with one or more zero bytes.
#[derive(Clone, Copy, Debug)]
pub struct ZeroPadding;

impl<B: ArrayLength<u8>> Padding<B> for ZeroPadding {
    const TYPE: PadType = PadType::Ambiguous;

    #[inline]
    fn pad(block: &mut Block<B>, pos: usize) {
        if pos > B::USIZE {
            panic!("`pos` is bigger than block size");
        }
        for b in &mut block[pos..] {
            *b = 0;
        }
    }

    #[inline]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        for i in (0..B::USIZE).rev() {
            if block[i] != 0 {
                return Ok(&block[..i + 1]);
            }
        }
        Ok(&block[..0])
    }
}

/// Pad block with bytes with value equal to the number of bytes added.
///
/// PKCS#7 described in the [RFC 5652](https://tools.ietf.org/html/rfc5652#section-6.3).
///
/// ```
/// use block_padding::{Pkcs7, Padding};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Pkcs7::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x04\x04\x04\x04");
/// let res = Pkcs7::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Pkcs7;

impl Pkcs7 {
    #[inline]
    fn unpad<B: ArrayLength<u8>>(block: &Block<B>, strict: bool) -> Result<&[u8], UnpadError> {
        // TODO: use bounds to check it at compile time
        if B::USIZE > 255 {
            panic!("block size is too big for PKCS#7");
        }
        let bs = B::USIZE;
        let n = block[bs - 1];
        if n == 0 || n as usize > bs {
            return Err(UnpadError);
        }
        let s = bs - n as usize;
        if strict && block[s..bs - 1].iter().any(|&v| v != n) {
            return Err(UnpadError);
        }
        Ok(&block[..s])
    }
}

impl<B: ArrayLength<u8>> Padding<B> for Pkcs7 {
    const TYPE: PadType = PadType::Reversible;

    #[inline]
    fn pad(block: &mut Block<B>, pos: usize) {
        // TODO: use bounds to check it at compile time
        if B::USIZE > 255 {
            panic!("block size is too big for PKCS#7");
        }
        if pos >= B::USIZE {
            panic!("`pos` is bigger or equal to block size");
        }
        let n = (B::USIZE - pos) as u8;
        for b in &mut block[pos..] {
            *b = n;
        }
    }

    #[inline]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        Pkcs7::unpad(block, true)
    }
}

/// Pad block with arbitrary bytes ending with value equal to the number of bytes added.
///
/// A variation of PKCS#7 that is less strict when decoding.
///
/// ```
/// use block_padding::{Iso10126, Padding};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Iso10126::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x04\x04\x04\x04");
/// let res = Iso10126::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Iso10126;

impl<B: ArrayLength<u8>> Padding<B> for Iso10126 {
    const TYPE: PadType = PadType::Reversible;

    #[inline]
    fn pad(block: &mut Block<B>, pos: usize) {
        // Instead of generating random bytes as specified by Iso10126 we
        // simply use Pkcs7 padding.
        Pkcs7::pad(block, pos)
    }

    #[inline]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        Pkcs7::unpad(block, false)
    }
}

/// Pad block with zeros except the last byte which will be set to the number
/// bytes.
///
/// ```
/// use block_padding::{AnsiX923, Padding};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// AnsiX923::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x00\x00\x00\x04");
/// let res = AnsiX923::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AnsiX923;

impl<B: ArrayLength<u8>> Padding<B> for AnsiX923 {
    const TYPE: PadType = PadType::Reversible;

    #[inline]
    fn pad(block: &mut Block<B>, pos: usize) {
        // TODO: use bounds to check it at compile time
        if B::USIZE > 255 {
            panic!("block size is too big for PKCS#7");
        }
        if pos >= B::USIZE {
            panic!("`pos` is bigger or equal to block size");
        }
        let bs = B::USIZE;
        for b in &mut block[pos..bs - 1] {
            *b = 0;
        }
        block[bs - 1] = (bs - pos) as u8;
    }

    #[inline]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        // TODO: use bounds to check it at compile time
        if B::USIZE > 255 {
            panic!("block size is too big for PKCS#7");
        }
        let bs = B::USIZE;
        let n = block[bs - 1] as usize;
        if n == 0 || n > bs {
            return Err(UnpadError);
        }
        let s = bs - n;
        if block[s..bs - 1].iter().any(|&v| v != 0) {
            return Err(UnpadError);
        }
        Ok(&block[..s])
    }
}

/// Pad block with byte sequence `\x80 00...00 00`.
///
/// ```
/// use block_padding::{Iso7816, Padding};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Iso7816::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x80\x00\x00\x00");
/// let res = Iso7816::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Iso7816;

impl<B: ArrayLength<u8>> Padding<B> for Iso7816 {
    const TYPE: PadType = PadType::Reversible;

    #[inline]
    fn pad(block: &mut Block<B>, pos: usize) {
        if pos >= B::USIZE {
            panic!("`pos` is bigger or equal to block size");
        }
        block[pos] = 0x80;
        for b in &mut block[pos + 1..] {
            *b = 0;
        }
    }

    #[inline]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        for i in (0..B::USIZE).rev() {
            match block[i] {
                0x80 => return Ok(&block[..i]),
                0x00 => continue,
                _ => return Err(UnpadError),
            }
        }
        Err(UnpadError)
    }
}

/// Don't pad the data. Useful for key wrapping.
///
/// ```
/// use block_padding::{NoPadding, Padding};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// NoPadding::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\xff\xff\xff\xff");
/// let res = NoPadding::unpad(&block).unwrap();
/// assert_eq!(res, b"test\xff\xff\xff\xff");
/// ```
///
/// Note that even though the passed length of the message is equal to 4,
/// the size of unpadded message is equal to the block size of 8 bytes.
/// Also padded message contains "garbage" bytes stored in the block buffer.
/// Thus `NoPadding` generally should not be used with data length of which
/// is not multiple of block size.
#[derive(Clone, Copy, Debug)]
pub struct NoPadding;

impl<B: ArrayLength<u8>> Padding<B> for NoPadding {
    const TYPE: PadType = PadType::NoPadding;

    #[inline]
    fn pad(_block: &mut Block<B>, pos: usize) {
        if pos > B::USIZE {
            panic!("`pos` is bigger than block size");
        }
    }

    #[inline]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        Ok(block)
    }
}

/// Failed unpadding operation error.
#[derive(Clone, Copy, Debug)]
pub struct UnpadError;

impl fmt::Display for UnpadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("Unpad Error")
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for UnpadError {}
