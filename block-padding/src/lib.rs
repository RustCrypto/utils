//! Padding and unpadding of messages divided into blocks.
//!
//! This crate provides `Padding` trait which provides padding and unpadding
//! operations. Additionally several common padding schemes are available out
//! of the box.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;
pub use generic_array;
use generic_array::{ArrayLength, GenericArray};

/// Block size.
pub type Block<B> = GenericArray<u8, B>;

/// Trait for padding messages divided into blocks
pub trait Padding<BlockSize: ArrayLength<u8>> {
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

impl<B: ArrayLength<u8>> Padding<B> for Pkcs7 {
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
        if block[s..bs - 1].iter().any(|&v| v != n) {
            return Err(UnpadError);
        }
        Ok(&block[..s])
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
impl std::error::Error for UnpadError {}
