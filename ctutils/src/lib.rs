#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![deny(unsafe_code)]
#![warn(
    clippy::mod_module_files,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused_qualifications
)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

//! # API Design
//!
//! ## [`Choice`]: constant-time analogue for [`bool`]
//! Values of this type are one of either [`Choice::FALSE`] or [`Choice::TRUE`].
//!
//! To achieve constant-time operation, `Choice` is ultimately used in combination with special
//! CPU-specific constant-time predication instructions implemented by the [`cmov`] crate
//! (with a portable "best effort" fallback that cannot provide guarantees).
//!
//! It additionally uses various methods to hint to the compiler that it should avoid inserting
//! branches based on its value where it otherwise would if `bool` were used instead, but cannot
//! provide guarantees in this regard.
//!
//! ## [`CtOption`]: constant-time analogue for [`Option`]
//! The core `Option` type is typically great for representing the conditional absence or presence
//! of a value, and provides a number of handy combinators for operating on them.
//!
//! However, it has a rather fundamental flaw when constant-time is desirable: its combinators are
//! lazily evaluated. To ensure constant-time operation, all combinators must be eagerly evaluated
//! so they aren't conditionally executed based on the value's presence.
//!
//! `CtOption` instead carries a `Choice` along with a value, which makes it possible to do
//! something it isn't with `Option`: evaluate combinators eagerly instead of lazily, running the
//! same functions regardless of the value's effective presence or absence.
//!
//! ## [`CtEq`]: constant-time analogue for [`PartialEq`]/[`Eq`]
//! Equality testing often short circuits for performance reasons, but when comparing values in
//! constant-time such short-circuiting is forbidden.
//!
//! The `CtEq` trait is a replacement for these scenarios. It's impl'd for several core types
//! including unsigned and signed integers as well as slices and arrays. It returns a `Choice`
//! as opposed to a `bool`], following the standard practice in this crate.
//!
//! *NOTE: for `subtle` users, this is the equivalent of the `ConstantTimeEq` trait*
//!
//! ## [`CtSelect`]: constant-time [predication]
//! Predication in computer architecture describes methods for conditionally modifying state
//! using non-branch instructions which perform conditional modifications based on a *predicate*
//! or boolean value, in the design of this library a `Choice`.
//!
//! The `CtSelect` trait provides methods for performing conditional moves, either conditionally
//! modifying a value in-place, or selecting from two different inputs and returning a new one.
//!
//! *NOTE: for `subtle` users, this is the equivalent of the `ConditionallySelectable` trait*
//!
//! [predication]: https://en.wikipedia.org/wiki/Predication_(computer_architecture)

mod choice;
mod ct_option;
mod traits;

pub use choice::Choice;
pub use ct_option::CtOption;
pub use traits::{ct_eq::CtEq, ct_gt::CtGt, ct_lt::CtLt, ct_select::CtSelect};
