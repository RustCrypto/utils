#![forbid(unsafe_code)]
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/dbl/0.3.1"
)]

extern crate generic_array;

use generic_array::typenum::{U16, U32, U8};
use generic_array::GenericArray;

use core::{convert::TryInto, mem::size_of};

const C64: u64 = 0b1_1011;
const C128: u64 = 0b1000_0111;
const C256: u64 = 0b100_0010_0101;

/// Double and inverse double over GF(2^n).
///
/// This trait is implemented for 64, 128 and 256 bit block sizes. Big-endian
/// order is used.
pub trait Dbl {
    /// Double block. (alternatively: multiply block by x)
    ///
    /// If most significant bit of the block equals to zero will return
    /// `block<<1`, otherwise `(block<<1)^C`, where `C` is the non-leading
    /// coefficients of the lexicographically first irreducible degree-b binary
    /// polynomial with the minimal number of ones.
    fn dbl(self) -> Self;

    /// Reverse double block. (alternatively: divbide block by x)
    ///
    /// If least significant bit of the block equals to zero will return
    /// `block>>1`, otherwise `(block>>1)^(1<<n)^(C>>1)`
    fn inv_dbl(self) -> Self;
}

impl Dbl for GenericArray<u8, U8> {
    #[inline]
    fn dbl(self) -> Self {
        let mut val = u64::from_be_bytes(self.into());

        let a = val >> 63;
        val <<= 1;
        val ^= a * C64;

        val.to_be_bytes().into()
    }

    #[inline]
    fn inv_dbl(self) -> Self {
        let mut val = u64::from_be_bytes(self.into());

        let a = val & 1;
        val >>= 1;
        val ^= a * ((1 << 63) ^ (C64 >> 1));

        val.to_be_bytes().into()
    }
}

impl Dbl for GenericArray<u8, U16> {
    #[inline]
    fn dbl(self) -> Self {
        let mut val = [
            u64::from_be_bytes(self[..8].try_into().unwrap()),
            u64::from_be_bytes(self[8..].try_into().unwrap()),
        ];

        let b = val[1] >> 63;
        let a = val[0] >> 63;

        val[0] <<= 1;
        val[0] ^= b;
        val[1] <<= 1;
        val[1] ^= a * C128;

        let mut res = Self::default();
        res[..8].copy_from_slice(&val[0].to_be_bytes());
        res[8..].copy_from_slice(&val[1].to_be_bytes());
        res
    }

    #[inline]
    fn inv_dbl(self) -> Self {
        let mut val = [
            u64::from_be_bytes(self[..8].try_into().unwrap()),
            u64::from_be_bytes(self[8..].try_into().unwrap()),
        ];

        let a = (val[0] & 1) << 63;
        let b = val[1] & 1;

        val[0] >>= 1;
        val[1] >>= 1;
        val[1] ^= a;
        val[0] ^= b * (1 << 63);
        val[1] ^= b * (C128 >> 1);

        let mut res = Self::default();
        res[..8].copy_from_slice(&val[0].to_be_bytes());
        res[8..].copy_from_slice(&val[1].to_be_bytes());
        res
    }
}

impl Dbl for GenericArray<u8, U32> {
    #[inline]
    fn dbl(self) -> Self {
        let mut val = [0u64; 4];
        for (s, v) in self.chunks_exact(size_of::<u64>()).zip(val.iter_mut()) {
            *v = u64::from_be_bytes(s.try_into().unwrap());
        }

        let a = val[0] >> 63;
        let b = val[1] >> 63;
        let c = val[2] >> 63;
        let d = val[3] >> 63;

        val[0] <<= 1;
        val[0] ^= b;
        val[1] <<= 1;
        val[1] ^= c;
        val[2] <<= 1;
        val[2] ^= d;
        val[3] <<= 1;
        val[3] ^= a * C256;

        let mut val_u8 = [0u8; 32];
        for (vu8, v) in val_u8.chunks_exact_mut(size_of::<u64>()).zip(val.iter()) {
            vu8.copy_from_slice(&v.to_be_bytes());
        }
        val_u8.into()
    }

    #[inline]
    fn inv_dbl(self) -> Self {
        let mut val = [0u64; 4];
        for (s, v) in self.chunks_exact(size_of::<u64>()).zip(val.iter_mut()) {
            *v = u64::from_be_bytes(s.try_into().unwrap());
        }

        let a = (val[0] & 1) << 63;
        let b = (val[1] & 1) << 63;
        let c = (val[2] & 1) << 63;
        let d = val[3] & 1;

        val[0] >>= 1;
        val[1] >>= 1;
        val[2] >>= 1;
        val[3] >>= 1;
        val[1] ^= a;
        val[2] ^= b;
        val[3] ^= c;

        val[0] ^= d * (1 << 63);
        val[3] ^= d * (C256 >> 1);

        let mut val_u8 = [0u8; 32];
        for (vu8, v) in val_u8.chunks_exact_mut(size_of::<u64>()).zip(val.iter()) {
            vu8.copy_from_slice(&v.to_be_bytes());
        }
        val_u8.into()
    }
}
