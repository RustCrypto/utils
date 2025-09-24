#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub use hybrid_array as array;

use core::fmt;
use hybrid_array::{Array, ArraySize};

/// Trait for message padding algorithms.
pub trait Padding {
    /// Pads `block` filled with data up to `pos` (i.e the message length
    /// stored in `block` is equal to `pos`).
    ///
    /// # Panics
    /// If `pos` is bigger than `block.len()`. Most padding algorithms also
    /// panic if they are equal.
    fn raw_pad(block: &mut [u8], pos: usize);

    /// Unpad data in `block`.
    ///
    /// Returns `Err(UnpadError)` if the block contains malformed padding.
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError>;

    /// Pads `block` filled with data up to `pos` (i.e the message length
    /// stored in `block` is equal to `pos`).
    ///
    /// # Panics
    /// If `pos` is bigger than `BlockSize`. Most padding algorithms also
    /// panic if they are equal.
    fn pad<BlockSize: ArraySize>(block: &mut Array<u8, BlockSize>, pos: usize) {
        Self::raw_pad(block.as_mut_slice(), pos);
    }

    /// Unpad data in `block`.
    ///
    /// Returns `Err(UnpadError)` if the block contains malformed padding.
    fn unpad<BlockSize: ArraySize>(block: &Array<u8, BlockSize>) -> Result<&[u8], UnpadError> {
        Self::raw_unpad(block.as_slice())
    }

    /// Unpad data in `blocks` and return unpadded byte slice.
    ///
    /// Returns `Err(UnpadError)` if the block contains malformed padding.
    fn unpad_blocks<BlockSize: ArraySize>(
        blocks: &[Array<u8, BlockSize>],
    ) -> Result<&[u8], UnpadError> {
        let bs = BlockSize::USIZE;
        let (last_block, full_blocks) = blocks.split_last().ok_or(UnpadError)?;
        let unpad_len = Self::unpad(last_block)?.len();
        assert!(unpad_len <= bs);
        let buf = Array::slice_as_flattened(blocks);
        let data_len = full_blocks.len() * bs + unpad_len;
        Ok(&buf[..data_len])
    }
}

/// Pad block with zeros.
///
/// ```
/// use block_padding::{ZeroPadding, Padding};
/// use block_padding::array::{Array, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: Array::<u8, U8> = [0xff; 8].into();
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

impl Padding for ZeroPadding {
    #[inline]
    fn raw_pad(block: &mut [u8], pos: usize) {
        if pos > block.len() {
            panic!("`pos` is bigger than block size");
        }
        block[pos..].fill(0);
    }

    #[inline]
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError> {
        for i in (0..block.len()).rev() {
            if block[i] != 0 {
                return Ok(&block[..i + 1]);
            }
        }
        Ok(&block[..0])
    }

    fn unpad_blocks<BlockSize: ArraySize>(
        blocks: &[Array<u8, BlockSize>],
    ) -> Result<&[u8], UnpadError> {
        let buf = Array::slice_as_flattened(blocks);
        for i in (0..buf.len()).rev() {
            if buf[i] != 0 {
                return Ok(&buf[..i + 1]);
            }
        }
        Ok(&buf[..0])
    }
}

/// Pad block with bytes with value equal to the number of bytes added.
///
/// PKCS#7 described in the [RFC 5652](https://tools.ietf.org/html/rfc5652#section-6.3).
///
/// ```
/// use block_padding::{Pkcs7, Padding};
/// use block_padding::array::{Array, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: Array::<u8, U8> = [0xff; 8].into();
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
    fn unpad(block: &[u8], strict: bool) -> Result<&[u8], UnpadError> {
        // TODO: use bounds to check it at compile time
        if block.len() > 255 {
            panic!("block size is too big for PKCS#7");
        }
        let bs = block.len();
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

impl Padding for Pkcs7 {
    #[inline]
    fn raw_pad(block: &mut [u8], pos: usize) {
        // TODO: use bounds to check it at compile time for Padding<B>
        if block.len() > 255 {
            panic!("block size is too big for PKCS#7");
        }
        if pos >= block.len() {
            panic!("`pos` is bigger or equal to block size");
        }
        let n = (block.len() - pos) as u8;
        block[pos..].fill(n);
    }

    #[inline]
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError> {
        Pkcs7::unpad(block, true)
    }
}

/// Pad block with arbitrary bytes ending with value equal to the number of bytes added.
///
/// A variation of PKCS#7 that is less strict when decoding.
///
/// ```
/// use block_padding::{Iso10126, Padding};
/// use block_padding::array::{Array, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: Array::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Iso10126::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x04\x04\x04\x04");
/// let res = Iso10126::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Iso10126;

impl Padding for Iso10126 {
    #[inline]
    fn raw_pad(block: &mut [u8], pos: usize) {
        // Instead of generating random bytes as specified by Iso10126 we
        // simply use Pkcs7 padding.
        Pkcs7::raw_pad(block, pos)
    }

    #[inline]
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError> {
        Pkcs7::unpad(block, false)
    }
}

/// Pad block with zeros except the last byte which will be set to the number
/// bytes.
///
/// ```
/// use block_padding::{AnsiX923, Padding};
/// use block_padding::array::{Array, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: Array::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// AnsiX923::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x00\x00\x00\x04");
/// let res = AnsiX923::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AnsiX923;

impl Padding for AnsiX923 {
    #[inline]
    fn raw_pad(block: &mut [u8], pos: usize) {
        // TODO: use bounds to check it at compile time
        if block.len() > 255 {
            panic!("block size is too big for ANSI X9.23");
        }
        if pos >= block.len() {
            panic!("`pos` is bigger or equal to block size");
        }
        let bs = block.len();
        block[pos..bs - 1].fill(0);
        block[bs - 1] = (bs - pos) as u8;
    }

    #[inline]
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError> {
        // TODO: use bounds to check it at compile time
        if block.len() > 255 {
            panic!("block size is too big for ANSI X9.23");
        }
        let bs = block.len();
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
/// use block_padding::array::{Array, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: Array::<u8, U8> = [0xff; 8].into();
/// block[..pos].copy_from_slice(msg);
/// Iso7816::pad(&mut block, pos);
/// assert_eq!(&block[..], b"test\x80\x00\x00\x00");
/// let res = Iso7816::unpad(&block).unwrap();
/// assert_eq!(res, msg);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Iso7816;

impl Padding for Iso7816 {
    #[inline]
    fn raw_pad(block: &mut [u8], pos: usize) {
        if pos >= block.len() {
            panic!("`pos` is bigger or equal to block size");
        }
        block[pos] = 0x80;
        block[pos + 1..].fill(0);
    }

    #[inline]
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError> {
        for i in (0..block.len()).rev() {
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
/// use block_padding::array::{Array, typenum::U8};
///
/// let msg = b"test";
/// let pos = msg.len();
/// let mut block: Array::<u8, U8> = [0xff; 8].into();
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

impl Padding for NoPadding {
    #[inline]
    fn raw_pad(block: &mut [u8], pos: usize) {
        if pos > block.len() {
            panic!("`pos` is bigger than block size");
        }
    }

    #[inline]
    fn raw_unpad(block: &[u8]) -> Result<&[u8], UnpadError> {
        Ok(block)
    }

    fn unpad_blocks<BlockSize: ArraySize>(
        blocks: &[Array<u8, BlockSize>],
    ) -> Result<&[u8], UnpadError> {
        Ok(Array::slice_as_flattened(blocks))
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

impl core::error::Error for UnpadError {}
