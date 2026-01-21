#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![allow(clippy::doc_markdown)]

//! # Supported bit sizes
//!
//! This crate supports the following bit sizes:
//! - `16`
//! - `32`
//! - `64`
//!
//! This matches the available options for `target_pointer_width` in `rustc`:
//! ```text
//! expected values for `target_pointer_width` are: `16`, `32`, and `64`
//! ```
//!
//! # Example
//!
//! See the [`cpubits`] macro itself for more detailed usage examples including other syntax
//! variations.
//!
//! ```
//! cpubits::cpubits! {
//!     16 => { pub type Word = u16; }
//!     32 => { pub type Word = u32; }
//!     64 => { pub type Word = u64; }
//! }
//! ```

// End of toplevel rustdoc, beginning of macro documentation. We put the detailed docs on the macro
// itself so we can re-export it, and people can easily get to these docs from the re-exported
// version.

/// A macro for defining code based on the optimal word size to use for the target, as chosen
/// heuristically at compile-time using `cfg`-based predicates.
///
/// # Usage
///
/// The macro works like a `match` expression that takes an implicit argument representing the
/// number of CPU bits, which is one of `16`, `32`, or `64`.
///
/// Use this macro to conditionally emit code specific to certain CPU word sizes, e.g. defining
/// types at compile-time based on the word size.
///
/// The macro doesn't create a new block and supports arbitrary statements in toplevel code, just
/// like the `cfg-if` crate (whose guts it recycles).
///
/// ## Basic usage
///
/// ```
/// cpubits::cpubits! {
///     16 => { pub type Word = u16; }
///     32 => { pub type Word = u32; }
///     64 => { pub type Word = u64; }
/// }
/// ```
///
/// ## Grouping multiple bit sizes
///
/// If you would like to group together 16-bit and 32-bit platforms, you can do so as follows:
///
/// ```
/// cpubits::cpubits! {
///     16 | 32 => { pub type Word = u32; }
///     64      => { pub type Word = u64; }
/// }
/// ```
///
/// ## Handling single-size cases
///
/// If you only want a block to run for a specific size, e.g. to know when it's possible to write
/// `impl From<u64> for MyWordNewtype`, you can do the following:
///
/// ```
/// # type Word = u64;
/// pub struct MyWordNewtype(Word);
///
/// cpubits::cpubits! {
///     64 => {
///         impl From<u64> for MyWordNewtype {
///             #[inline]
///             fn from(n: u64) -> MyWordNewtype {
///                 MyWordNewtype(n)
///             }
///         }
///     }
/// }
/// ```
///
/// # Selection rules
///
/// The macro augments `target_pointer_width`-based selection with specific overrides which promote
/// certain targets from 32-bit to 64-bit ones.
///
/// This 64-bit promotion occurs if `any` of the following `cfg`s are true:
/// - ARMv7: `all(target_arch = "arm", not(target_feature = "thumb-mode"))`
/// - WASM: `target_arch = "wasm32"`
#[macro_export]
macro_rules! cpubits {
    // Only run the given block if we have selected a 16-bit word size, i.e. the code will be
    // ignored on 32-bit and 64-bit platforms.
    ( 16 => { $( $tokens:tt )* } ) => {
        $crate::cpubits! {
            16 => { $( $tokens )* },
            32 | 64 => { }
        }
    };

    // Only run the given block if we have selected a 32-bit word size, i.e. the code will be
    // ignored on 32-bit and 64-bit platforms.
    ( 32 => { $( $tokens:tt )* } ) => {
        $crate::cpubits! {
            16 => { }
            32 => { $( $tokens )* }
            64 => { }
        }
    };

    // Only run the given block if we have selected a 64-bit word size, i.e. the code will be
    // ignored on 16-bit and 32-bit platforms.
    ( 64 => { $( $tokens:tt )* } ) => {
        $crate::cpubits! {
            16 | 32 => { }
            64 => { $( $tokens )* }
        }
    };

    // Only run the block on 16-bit and 32-bit targets.
    ( 16 | 32 => { $( $tokens:tt )* } ) => {
        $crate::cpubits! {
            16 => { $( $tokens )* }
            32 => { $( $tokens )* }
            64 => { }
        }
    };

    // Only run the block on 32-bit and 64-bit targets.
    ( 32 | 64 => { $( $tokens:tt )* } ) => {
        $crate::cpubits! {
            16 => { }
            32 => { $( $tokens )* }
            64 => { $( $tokens )* }
        }
    };

    // Select between 16-bit and 32-bit options, where 64-bit will use the 32-bit option
    (
        16 => { $( $tokens16:tt )* }
        32 | 64 => { $( $tokens32:tt )* }
    ) => {
        $crate::cpubits! {
            16 => { $( $tokens16 )* }
            32 => { $( $tokens32 )* }
            64 => { $( $tokens32 )* }
        }
    };

    // Select between 32-bit and 64-bit options, where 16-bit will use the 32-bit option
    (
        16 | 32 => { $( $tokens32:tt )* }
        64 => { $( $tokens64:tt )* }
    ) => {
        $crate::cpubits! {
            16 => { $( $tokens32 )* }
            32 => { $( $tokens32 )* }
            64 => { $( $tokens64 )* }
        }
    };

    // The general API which runs a different block for each possible word size
    (
        16 => { $( $tokens16:tt )* }
        32 => { $( $tokens32:tt )* }
        64 => { $( $tokens64:tt )* }
    ) => {
        $crate::cpubits! {
            // `cfg` selector for 64-bit target overrides
            // Implicitly `cfg(any(...))`
            #[cfg(enable_64bit(
                // ARMv7
                all(target_arch = "arm", not(target_feature = "thumb-mode")),
                // WASM
                target_arch = "wasm32",
            ))]
            16 => { $( $tokens32 )* }
            32 => { $( $tokens32 )* }
            64 => { $( $tokens64 )* }
        }
    };

    // Same API as immediately above, but with a pseudo-attribute we use to pass the `cfg` overrides
    // for `target_pointer_width` that promote a 32-bit target into a 64-bit one.
    (
        #[cfg(enable_64bit( $($enable_64bit:tt)+ ))]
        16 => { $( $tokens16:tt )* }
        32 => { $( $tokens32:tt )* }
        64 => { $( $tokens64:tt )* }
    ) => {
        $crate::cfg_if! {
            @__items () ;
            (
                ( target_pointer_width = "16" )
                ( $( $tokens16 )* )
            ),
            (
                (all(
                    target_pointer_width = "32",
                    not(any($( $enable_64bit )+))
                ))
                ( $( $tokens32 )* )
            ),
            (
                (any(
                    target_pointer_width = "64",
                    any($( $enable_64bit )+)
                ))
                ( $( $tokens64 )* )
            ),
            (
                ()
                ( compile_error!("unsupported target pointer width") )
            ),
        }
    };
}

/// Vendored partial copy of the `cfg_if::cfg_if` macro.
/// Copyright (c) 2014 Alex Crichton. Dual-licensed Apache 2.0 + MIT.
#[doc(hidden)]
#[macro_export]
macro_rules! cfg_if {
    // Internal and recursive macro to emit all the items
    //
    // Collects all the previous cfgs in a list at the beginning, so they can be
    // negated. After the semicolon are all the remaining items.
    (@__items ( $( ($($_:tt)*) , )* ) ; ) => {};
    (
        @__items ( $( ($($no:tt)+) , )* ) ;
        (( $( $($yes:tt)+ )? ) ( $( $tokens:tt )* )),
        $( $rest:tt , )*
    ) => {
        // Emit all items within one block, applying an appropriate #[cfg]. The
        // #[cfg] will require all `$yes` matchers specified and must also negate
        // all previous matchers.
        #[cfg(all(
            $( $($yes)+ , )?
            not(any( $( $($no)+ ),* ))
        ))]
        // Subtle: You might think we could put `$( $tokens )*` here. But if
        // that contains multiple items then the `#[cfg(all(..))]` above would
        // only apply to the first one. By wrapping `$( $tokens )*` in this
        // macro call, we temporarily group the items into a single thing (the
        // macro call) that will be included/excluded by the `#[cfg(all(..))]`
        // as appropriate. If the `#[cfg(all(..))]` succeeds, the macro call
        // will be included, and then evaluated, producing `$( $tokens )*`. See
        // also the "issue #90" test below.
        $crate::cfg_if! { @__temp_group $( $tokens )* }

        // Recurse to emit all other items in `$rest`, and when we do so add all
        // our `$yes` matchers to the list of `$no` matchers as future emissions
        // will have to negate everything we just matched as well.
        $crate::cfg_if! {
            @__items ( $( ($($no)+) , )* $( ($($yes)+) , )? ) ;
            $( $rest , )*
        }
    };

    // See the "Subtle" comment above.
    (@__temp_group $( $tokens:tt )* ) => {
        $( $tokens )*
    };
}

#[cfg(test)]
mod tests {
    /// Return an integer that maps to the number of bits `cpubits` detected.
    fn detected_bits() -> u32 {
        cpubits! {
            16 => { 16 }
            32 => { 32 }
            64 => { 64 }
        }
    }

    /// Return an integer that maps to `target_pointer_width`.
    #[allow(dead_code)]
    fn detect_pointer_width() -> u32 {
        if cfg!(target_pointer_width = "16") {
            16
        } else if cfg!(target_pointer_width = "32") {
            32
        } else if cfg!(target_pointer_width = "64") {
            64
        } else {
            unreachable!("rustc only support 16, 32, and 64-bit pointer widths")
        }
    }

    /// Return the expected number of bits for the target.
    fn expected_bits() -> u32 {
        // Duplicated 64-bit override predicates need to go here
        if cfg!(any(
            // ARMv7
            all(target_arch = "arm", not(target_feature = "thumb-mode")),
            // WASM
            target_arch = "wasm32"
        )) {
            64
        } else {
            detect_pointer_width()
        }
    }

    #[test]
    fn cpubits_works() {
        assert_eq!(detected_bits(), expected_bits());
    }

    /// Explicit test for ARMv7 so we can see the predicate is working
    #[cfg(all(target_arch = "arm", not(target_feature = "thumb-mode")))]
    #[test]
    fn cpubits_on_armv7_is_64bit() {
        assert_eq!(detected_bits(), 64);
    }

    /// Explicit test for WASM so we can see the predicate is working
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn cpubits_on_wasm_is_64bit() {
        assert_eq!(detected_bits(), 64);
    }

    #[test]
    fn cpubits_16_or_32_vs_64() {
        fn bits32or64() -> u32 {
            cpubits! {
                16 | 32 => { 32 }
                64 => { 64 }
            }
        }

        match expected_bits() {
            16 | 32 => assert_eq!(32, bits32or64()),
            64 => assert_eq!(64, bits32or64()),
            bits => unreachable!("#{bits}-bits should be one of: 16, 32, 64"),
        }
    }
}
