#![allow(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    unsafe_code
)]

use crate::{Choice, CtOption};
use core::{
    cmp,
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU128,
    },
};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

/// Marker trait for types which do not contain uninitialized memory.
pub unsafe trait NoUninit {}

// Impl `NoUninit` for the given type
macro_rules! impl_no_uninit {
    ( $($ty:ty),+ ) => {
        $(
            unsafe impl NoUninit for $ty {}
        )+
    };
}

impl_no_uninit!(
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128
);

unsafe impl NoUninit for Choice {}
unsafe impl NoUninit for cmp::Ordering {}
unsafe impl<T: NoUninit> NoUninit for CtOption<T> {}
unsafe impl<T: NoUninit> NoUninit for [T] {}
unsafe impl<T: NoUninit, const N: usize> NoUninit for [T; N] {}

#[cfg(feature = "alloc")]
unsafe impl<T: NoUninit> NoUninit for Box<T> {}
#[cfg(feature = "alloc")]
unsafe impl<T: NoUninit> NoUninit for Vec<T> {}
