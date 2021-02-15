//! Portable "SIMD" buffers.
//!
//! These allow these buffer types to be used on platforms where there is
//! no native SIMD implementation for them to map to.

use crate::{LengthError, SimdBuffer, SimdRegister};
use core::{
    convert::TryFrom,
    mem,
    ops::{BitXor, BitXorAssign, Index, IndexMut},
};

/// 128-bit SIMD integer
#[derive(Copy, Clone, Default, Debug)]
#[repr(transparent)]
pub struct U128(u128);

impl SimdRegister for U128 {
    type Repr = [u8; 16];

    fn load(bytes: &[u8; 16]) -> Self {
        Self::from(bytes)
    }

    fn store(self, bytes: &mut [u8; 16]) {
        bytes.copy_from_slice(&Self::Repr::from(self))
    }
}

impl From<u128> for U128 {
    #[inline(always)]
    fn from(x: u128) -> U128 {
        Self(x)
    }
}

impl From<U128> for u128 {
    #[inline(always)]
    fn from(x: U128) -> u128 {
        x.0
    }
}

impl From<[u8; 16]> for U128 {
    #[inline(always)]
    fn from(bytes: [u8; 16]) -> U128 {
        Self(u128::from_ne_bytes(bytes))
    }
}

impl From<&[u8; 16]> for U128 {
    #[inline(always)]
    fn from(bytes: &[u8; 16]) -> U128 {
        Self(u128::from_ne_bytes(*bytes))
    }
}

impl From<U128> for [u8; 16] {
    #[inline(always)]
    fn from(x: U128) -> [u8; 16] {
        x.0.to_ne_bytes()
    }
}

impl BitXor for U128 {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for U128 {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl TryFrom<&[u8]> for U128 {
    type Error = LengthError;

    fn try_from(bytes: &[u8]) -> Result<U128, LengthError> {
        if bytes.len() <= 16 {
            let mut buf = [0u8; 16];
            buf[..bytes.len()].copy_from_slice(bytes);
            Ok(buf.into())
        } else {
            Err(LengthError)
        }
    }
}

/// 1024-bit SIMD buffer decomposed into 8 x [`U128`] values.
#[derive(Clone, Debug, Default)]
#[repr(align(128))]
pub struct U128x8([U128; 8]);

impl TryFrom<&[u8]> for U128x8 {
    type Error = LengthError;

    fn try_from(bytes: &[u8]) -> Result<U128x8, LengthError> {
        let mut chunks = bytes.chunks_exact(16);
        let mut result: [U128; 8] = unsafe { mem::zeroed() };
        let mut i = 0;

        for chunk in &mut chunks {
            if i == 8 {
                return Err(LengthError);
            }

            result[i] = U128::try_from(chunk)?;
            i += 1;
        }

        let rem = chunks.remainder();

        if !rem.is_empty() {
            if i < 8 {
                result[i] = U128::try_from(rem)?;
                i += 1;
            } else {
                return Err(LengthError);
            }
        }

        while i < 8 {
            result[i] = U128::zero();
            i += 1;
        }

        Ok(Self(result))
    }
}

impl BitXorAssign for U128x8 {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        for (x, y) in self.0.iter_mut().zip(&rhs.0) {
            *x ^= *y
        }
    }
}

impl Index<usize> for U128x8 {
    type Output = U128;

    fn index(&self, idx: usize) -> &U128 {
        self.0.index(idx)
    }
}

impl IndexMut<usize> for U128x8 {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.0.index_mut(idx)
    }
}

impl SimdBuffer for U128x8 {
    type Register = U128;

    fn as_slice(&self) -> &[U128] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [U128] {
        &mut self.0
    }
}
