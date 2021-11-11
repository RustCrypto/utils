//! Padding and unpadding of messages divided into blocks.
//!
//! This crate provides `Padding` trait which provides padding and unpadding
//! operations. Additionally several common padding schemes are available out
//! of the box.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/block-padding/0.3.0"
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
    /// Pad `block` filled with data up to `pos` (i.e length of a message
    /// stored in the block is equal to `pos`).
    ///
    /// Returns `PadError` if `pos` is bigger than `BlockSize`. Most padding
    /// algorithms return `PadError` also if it is equal to `BlockSize`.
    fn pad(block: &mut Block<BlockSize>, pos: usize) -> Result<(), PadError>;

    /// Pad slice and return resulting block.
    ///
    /// Returns `PadError` if `buf` length is bigger than `BlockSize`. Most
    /// padding algorithms return `PadError` also if it is equal to `BlockSize`.
    #[inline(always)]
    fn pad_slice(buf: &[u8]) -> Result<Block<BlockSize>, PadError> {
        if buf.len() > BlockSize::USIZE {
            return Err(PadError);
        }
        let mut block = Block::<BlockSize>::default();
        block[..buf.len()].copy_from_slice(buf);
        Self::pad(&mut block, buf.len())?;
        Ok(block)
    }
}

/// Trait for unpaddable padding algorithms.
pub trait PadUnpad<BlockSize: ArrayLength<u8>>: Padding<BlockSize> {
    /// Unpad data in the `block`.
    ///
    /// Returns `Err(UnpadError)` if the block contains malformed padding.
    fn unpad(block: &Block<BlockSize>) -> Result<&[u8], UnpadError>;
}

/// Pad block with zeros.
///
/// Returns an error on padding if `pos > BlockSize`.
///
/// ```
/// use block_padding::{ZeroPadding, Padding, PadUnpad};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// ZeroPadding::pad(&mut block, 8).unwrap();
/// assert_eq!(&block[..], b"test\xff\xff\xff\xff");
/// let res = ZeroPadding::unpad(&mut block).unwrap();
/// assert_eq!(res, b"test\xff\xff\xff\xff");
///
/// ZeroPadding::pad(&mut block, pos).unwrap();
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
    #[inline(always)]
    fn pad(block: &mut Block<B>, pos: usize) -> Result<(), PadError> {
        if pos > B::USIZE {
            return Err(PadError);
        }
        for b in &mut block[pos..] {
            *b = 0;
        }
        Ok(())
    }
}

impl<B: ArrayLength<u8>> PadUnpad<B> for ZeroPadding {
    #[inline(always)]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        for i in 0..B::USIZE {
            if block[i] == 0 {
                return Ok(&block[..i]);
            }
        }
        Ok(block)
    }
}

/// Pad block with bytes with value equal to the number of padding bytes.
///
/// Returns an error on padding if `pos >= BlockSize`.
///
/// PKCS#7 described in the [RFC 5652](https://tools.ietf.org/html/rfc5652#section-6.3).
///
/// ```
/// use block_padding::{Pkcs7, Padding, PadUnpad};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Pkcs7::pad(&mut block, pos).unwrap();
/// assert_eq!(&block[..], b"test\x04\x04\x04\x04");
/// let res = Pkcs7::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Pkcs7;

impl<B: ArrayLength<u8>> Padding<B> for Pkcs7 {
    #[inline(always)]
    fn pad(block: &mut Block<B>, pos: usize) -> Result<(), PadError> {
        if B::USIZE > 255 || pos >= B::USIZE {
            return Err(PadError);
        }
        let n = (B::USIZE - pos) as u8;
        for b in &mut block[pos..] {
            *b = n;
        }
        Ok(())
    }
}

impl<B: ArrayLength<u8>> PadUnpad<B> for Pkcs7 {
    #[inline(always)]
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

/// Pad block with zeros except the last byte which will be set
/// to the number of padding bytes.
///
/// Returns an error on padding if `pos >= BlockSize`.
///
/// ```
/// use block_padding::{AnsiX923, Padding, PadUnpad};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// AnsiX923::pad(&mut block, pos).unwrap();
/// assert_eq!(&block[..], b"test\x00\x00\x00\x04");
/// let res = AnsiX923::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AnsiX923;

impl<B: ArrayLength<u8>> Padding<B> for AnsiX923 {
    #[inline(always)]
    fn pad(block: &mut Block<B>, pos: usize) -> Result<(), PadError> {
        if B::USIZE > 255 || pos >= B::USIZE {
            return Err(PadError);
        }
        let bs = B::USIZE;
        for b in &mut block[pos..bs - 1] {
            *b = 0;
        }
        block[bs - 1] = (bs - pos) as u8;
        Ok(())
    }
}

impl<B: ArrayLength<u8>> PadUnpad<B> for AnsiX923 {
    #[inline(always)]
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
/// Returns an error on padding if `pos >= BlockSize`.
///
/// ```
/// use block_padding::{Iso7816, Padding, PadUnpad};
/// use generic_array::{GenericArray, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: GenericArray::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Iso7816::pad(&mut block, pos).unwrap();
/// assert_eq!(&block[..], b"test\x80\x00\x00\x00");
/// let res = Iso7816::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Iso7816;

impl<B: ArrayLength<u8>> Padding<B> for Iso7816 {
    #[inline(always)]
    fn pad(block: &mut Block<B>, pos: usize) -> Result<(), PadError> {
        if pos >= B::USIZE {
            return Err(PadError);
        }
        block[pos] = 0x80;
        for b in &mut block[pos + 1..] {
            *b = 0;
        }
        Ok(())
    }
}

impl<B: ArrayLength<u8>> PadUnpad<B> for Iso7816 {
    #[inline(always)]
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
/// Returns an error on padding if `pos != BlockSize`.
///
/// ```
/// use block_padding::{NoPadding, Padding, PadUnpad};
/// use generic_array::{GenericArray, typenum::U4};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block = GenericArray::<u8, U4>::clone_from_slice(msg);
/// NoPadding::pad(&mut block, pos).unwrap();
/// assert_eq!(&block[..], msg);
/// let res = NoPadding::unpad(&block).unwrap();
/// assert_eq!(res, b"test");
/// let res = NoPadding::pad(&mut block, pos - 1);
/// assert!(res.is_err())
/// ```
///
/// `NoPadding` generally should not be used with data not dividable into
/// blocks.
#[derive(Clone, Copy, Debug)]
pub struct NoPadding;

impl<B: ArrayLength<u8>> Padding<B> for NoPadding {
    #[inline(always)]
    fn pad(_block: &mut Block<B>, pos: usize) -> Result<(), PadError> {
        if pos == B::USIZE {
            Ok(())
        } else {
            Err(PadError)
        }
    }
}

impl<B: ArrayLength<u8>> PadUnpad<B> for NoPadding {
    #[inline(always)]
    fn unpad(block: &Block<B>) -> Result<&[u8], UnpadError> {
        Ok(block)
    }
}

/// Failed padding error.
#[derive(Clone, Copy, Debug)]
pub struct PadError;

impl fmt::Display for PadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("Unpad Error")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PadError {}

/// Failed unpadding error.
#[derive(Clone, Copy, Debug)]
pub struct UnpadError;

impl fmt::Display for UnpadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("Unpad Error")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnpadError {}
