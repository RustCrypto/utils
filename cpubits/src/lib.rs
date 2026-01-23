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
//!
//! ```text
//! expected values for `target_pointer_width` are: `16`, `32`, and `64`
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
/// ## Use as an expression
///
/// It's also possible to use the macro as an expression, although in somewhat limited contexts
/// due to its attribute handling:
///
/// ```
/// fn detected_cpubits() -> u32 {
///     cpubits::cpubits! {
///         16 => { 16 }
///         32 => { 32 }
///         64 => { 64 }
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
/// - ARMv7: `all(target_arch = "arm", target_feature = "v7")`
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
            #[cfg(enable_64_bit = any(
                // ARMv7
                all(target_arch = "arm", target_feature = "v7"),
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
        #[cfg(enable_64_bit = $($enable_64_bit:tt)+ )]
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
                    not($( $enable_64_bit )+)
                ))
                ( $( $tokens32 )* )
            ),
            (
                (any(
                    target_pointer_width = "64",
                    $( $enable_64_bit )+
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

/// Vendored copy of the `cfg_if::cfg_if` macro.
/// Copyright (c) 2014 Alex Crichton. Dual-licensed Apache 2.0 + MIT.
///
/// NOTE: though this is marked `doc(hidden)`, it is considered a stable part of the public API.
#[doc(hidden)]
#[macro_export]
macro_rules! cfg_if {
    // NOTE(cpubits): we deliberately include the original frontend even though we don't use it
    // internally within this crate so consumers of `cpubits` can use the vendored `cfg_if` instead
    // of requiring both `cpubits` and `cfg-if`.
    (
        if #[cfg( $($i_meta:tt)+ )] { $( $i_tokens:tt )* }
        $(
            else if #[cfg( $($ei_meta:tt)+ )] { $( $ei_tokens:tt )* }
        )*
        $(
            else { $( $e_tokens:tt )* }
        )?
    ) => {
        $crate::cfg_if! {
            @__items () ;
            (( $($i_meta)+ ) ( $( $i_tokens )* )),
            $(
                (( $($ei_meta)+ ) ( $( $ei_tokens )* )),
            )*
            $(
                (() ( $( $e_tokens )* )),
            )?
        }
    };

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

/// Constant representing the detection result from `cpubits!` on the current target.
pub const CPUBITS: u32 = {
    cpubits! {
        16 => { 16 }
        32 => { 32 }
        64 => { 64 }
    }
};

#[cfg(test)]
mod tests {
    use super::CPUBITS;

    /// Return the expected number of bits for the target.
    fn expected_bits() -> u32 {
        // Duplicated 64-bit override predicates need to go here
        if cfg!(any(
            // ARMv7
            all(target_arch = "arm", target_feature = "v7"),
            // WASM
            target_arch = "wasm32"
        )) {
            64
        } else {
            usize::BITS
        }
    }

    #[test]
    fn cpubits_works() {
        assert_eq!(CPUBITS, expected_bits());
    }

    /// Explicit test for ARMv7 so we can see the predicate is working
    #[cfg(all(target_arch = "arm", target_feature = "v7"))]
    #[test]
    fn cpubits_on_armv7_is_64bit() {
        assert_eq!(CPUBITS, 64);
    }

    /// Explicit test for WASM so we can see the predicate is working
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn cpubits_on_wasm_is_64bit() {
        assert_eq!(CPUBITS, 64);
    }

    /// Test for the `16 | 32` syntax.
    #[test]
    fn cpubits_16_or_32_vs_64() {
        const BITS: u32 = {
            cpubits! {
                16 | 32 => { 32 }
                64 => { 64 }
            }
        };

        match expected_bits() {
            16 | 32 => assert_eq!(32, BITS),
            64 => assert_eq!(64, BITS),
            bits => unreachable!("#{bits}-bits should be one of: 16, 32, 64"),
        }
    }
}
