#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![warn(missing_docs, unused_qualifications)]

//! # zeroize_stack
//!
//! A crate for sanitizing stack memory after sensitive operations—sometimes referred to as _Stack Bleaching_.
//!
//! Modern compilers and CPUs routinely copy, spill, and rearrange data during execution. Even if sensitive values are scoped to a function, they may:
//! - Be duplicated across multiple stack frames
//! - Be spilled from registers to the stack during register pressure
//! - Persist in memory long after the function returns
//!
//! This crate provides tools to explicitly zeroize stack regions used during
//! cryptographic or sensitive computations, helping mitigate:
//! - Leakage through stack inspection or memory dumps
//! - Residual data from compiler-inserted spills
//! - ABI-visible register reuse across function boundaries
//!
//! ## Why Stack Sanitization Matters
//!
//! Unlike heap memory, stack allocations are ephemeral and compiler-controlled.
//! Sensitive data may be:
//! - Copied implicitly by the optimizer
//! - Stored temporarily during register allocation
//! - Left behind in stack frames even after function return
//!
//! This crate offers abstractions for:
//! - Executing functions on isolated, aligned stack buffers
//! - Zeroizing stack memory after execution
//!
//! ## Safety
//!
//! These operations involve low-level stack manipulation and unsafe code. The
//! caller must ensure:
//! - The stack size provided is large enough for the closure to run with.
//! - The closure does not unwind or return control flow by any means other than
//!   directly returning. `std` users do not need to worry about this due to
//!   the existence of `catch_unwind`.
//!
//! ## `nostd` Support
//!
//! This crate is compatible with `nostd` environments, but it is less safe
//! in the event that your stack-switched stack panics. Panicking on a separate
//! stack can cause undefined behavior (UB), but if it can be caught with
//! `std::panic::catch_unwind`, that aspect of the safety should be more safe.
//!
//! When using `nostd`, try to ensure that `panic = "abort"` to avoid the
//! unsafety of unwinding across stack boundaries.
//!
//! ## Use Cases
//!
//! - Cryptographic routines
//! - Secure enclave transitions
//! - Sanitizing temporary buffers in high-assurance systems

#[cfg(feature = "heap")]
pub mod heap;
