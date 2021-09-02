//! Pure Rust implementation of a big integer library designed for cryptography.
//!
//! # About
//! This library has been designed  from the ground-up for use in cryptographic
//! applications. It provides constant-time, `no_std`-friendly implementations
//! of modern formulas implemented using const generics.
//!
//! # Minimum Supported Rust Version
//! **Rust 1.51** at a minimum.
//!
//! # Goals
//! - No heap allocations i.e. `no_std`-friendly.
//! - Constant-time by default using traits from the [`subtle`] crate.
//! - Leverage what is possible today with const generics on `stable` rust.
//! - Support `const fn` as much as possible, including decoding big integers from
//!   bytes/hex and performing arithmetic operations on them, with the goal of
//!   being able to compute values at compile-time.
//!
//! # Status
//! This library presently provides only a baseline level of functionality.
//! It's new, unaudited, and may contain bugs. We recommend that it only be
//! used in an experimental capacity for now.
//!
//! Please see the [feature wishlist tracking ticket] for more information.
//!
//! # `generic-array` interop
//! When the optional `generic-array` feature is enabled, this library provides
//! an [`ArrayEncoding`] trait which can be used to serialize/deserialize big
//! integer values as `GenericArray<u8, N>`.
//!
//! [feature wishlist tracking ticket]: https://github.com/RustCrypto/utils/issues/453

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/crypto-bigint/0.2.5"
)]
#![forbid(unsafe_code, clippy::unwrap_used)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(all(feature = "alloc", test))]
extern crate alloc;

#[macro_use]
mod macros;

#[cfg(feature = "generic-array")]
mod array;
mod checked;
pub mod limb;
mod traits;
mod uint;
mod wrapping;

pub use crate::{checked::Checked, limb::Limb, traits::*, uint::*, wrapping::Wrapping};
pub use subtle;

#[cfg(feature = "generic-array")]
pub use {
    self::array::{ArrayEncoding, ByteArray},
    generic_array::{self, typenum::consts},
};

/// Number of bytes in a [`Limb`].
#[cfg(target_pointer_width = "32")]
#[deprecated(since = "0.2.2", note = "use `Limb::BYTE_SIZE` instead")]
pub const LIMB_BYTES: usize = 4;

/// Number of bytes in a [`Limb`].
#[cfg(target_pointer_width = "64")]
#[deprecated(since = "0.2.2", note = "use `Limb::BYTE_SIZE` instead")]
pub const LIMB_BYTES: usize = 8;
