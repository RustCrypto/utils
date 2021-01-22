//! SIMD buffer types.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/simd-buffers/0.0.0"
)]
#![warn(rust_2018_idioms)] // TODO: missing_docs

pub mod temp_traits;

use cfg_if::cfg_if;
use core::{convert::TryFrom, fmt::Debug, ops, slice};

cfg_if! {
    if #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "force-soft")))] {
        mod x86;
        pub use x86::{U128x8, U128, U256};
    } else {
        mod portable;
        pub use portable::{U128x8, U128};
    }
}

/// Length error
pub struct LengthError;

/// SIMD register trait.
pub trait SimdRegister:
    Copy
    + Clone
    + Debug
    + Default
    + Sized
    + ops::BitXor
    + ops::BitXorAssign
    + for<'a> TryFrom<&'a [u8], Error = LengthError>
{
    /// Byte representation (i.e. a byte array)
    type Repr: Default + AsRef<[u8]> + AsMut<[u8]>;

    /// Return the zero value.
    #[inline(always)]
    fn zero() -> Self {
        Self::default()
    }

    /// Load a register from its (potentially unaligned) byte representation.
    fn load(bytes: &Self::Repr) -> Self;

    /// Store a register into the given byte representation.
    fn store(self, bytes: &mut Self::Repr);
}

/// SIMD buffer trait
pub trait SimdBuffer:
    Default
    + Sized
    + ops::BitXorAssign
    + ops::Index<usize>
    + ops::IndexMut<usize>
    + for<'a> TryFrom<&'a [u8], Error = LengthError>
{
    /// SIMD register type used by this buffer.
    type Register: SimdRegister;

    /// Borrow all registers in this buffer as a slice.
    fn as_slice(&self) -> &[Self::Register];

    /// Borrow all registers in this buffer as a mut slice.
    fn as_mut_slice(&mut self) -> &mut [Self::Register];

    /// Iterate over the registers.
    #[inline(always)]
    fn iter(&self) -> slice::Iter<'_, Self::Register> {
        self.as_slice().iter()
    }

    /// Iterate over the registers mutably.
    #[inline(always)]
    fn iter_mut(&mut self) -> slice::IterMut<'_, Self::Register> {
        self.as_mut_slice().iter_mut()
    }
}
