#![no_std]
extern crate generic_array;

use generic_array::typenum::{U8, U16, U32};
use generic_array::GenericArray;

use core::mem;

const C64: u64 = 0b1_1011;
const C128: u64 = 0b1000_0111;
const C256: u64 = 0b100_0010_0101;

/// Double and inverse double over GF(2^n). WARNING: block must be aligned!
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
    fn dbl(self) -> Self {
        let mut val: u64 = unsafe { mem::transmute(self) };
        val = val.to_be();
        let a = val >> 63;
        val <<= 1;
        val ^= a * C64;
        unsafe { mem::transmute(val.to_be()) }
    }

    fn inv_dbl(self) -> Self {
        let mut val: u64 = unsafe { mem::transmute(self) };
        val = val.to_be();

        let a = val & 1;
        val >>= 1;
        val ^= a*((1 << 63) ^ (C64 >> 1));

        unsafe { mem::transmute(val.to_be()) }
    }
}

#[inline(always)]
fn to_be(val: &mut [u64]) {
    for v in val.iter_mut() {
        *v = v.to_be();
    }
}

impl Dbl for GenericArray<u8, U16> {
    fn dbl(self) -> Self {
        let mut val: [u64; 2] = unsafe { mem::transmute(self) };
        to_be(&mut val);

        let b = val[1] >> 63;
        let a = val[0] >> 63;

        val[0] <<= 1;
        val[0] ^= b;
        val[1] <<= 1;
        val[1] ^= a*C128;

        to_be(&mut val);
        unsafe { mem::transmute(val) }
    }

    fn inv_dbl(self) -> Self {
        let mut val: [u64; 2] = unsafe { mem::transmute(self) };
        to_be(&mut val);

        let a = (val[0] & 1) << 63;
        let b = val[1] & 1;

        val[0] >>= 1;
        val[1] >>= 1;
        val[1] ^= a;
        val[0] ^= b*(1 << 63);
        val[1] ^= b*(C128 >> 1);

        to_be(&mut val);
        unsafe { mem::transmute(val) }
    }
}

impl Dbl for GenericArray<u8, U32> {
    fn dbl(self) -> Self {
        let mut val: [u64; 4] = unsafe { mem::transmute(self) };
        to_be(&mut val);

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
        val[3] ^= a*C256;

        to_be(&mut val);
        unsafe { mem::transmute(val) }
    }

    fn inv_dbl(self) -> Self {
        let mut val: [u64; 4] = unsafe { mem::transmute(self) };
        to_be(&mut val);

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

        val[0] ^= d*(1 << 63);
        val[3] ^= d*(C256 >> 1);

        to_be(&mut val);
        unsafe { mem::transmute(val) }
    }
}
