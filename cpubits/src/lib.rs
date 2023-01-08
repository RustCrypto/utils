#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

//! # Usage
//!
//! Use this macro to e.g. conditionally define types at compile-time based on the word size.
//!
//! ```
//! cpubits::cpubits! {
//!     16 => { pub type Word = u16; }
//!     32 => { pub type Word = u32; }
//!     64 => { pub type Word = u64; }
//! }
//! ```
//!
//! If you would like to group together 16-bit and 32-bit platforms, you can do so as follows:
//!
//! ```
//! cpubits::cpubits! {
//!     16 | 32 => { pub type Word = u32; }
//!     64      => { pub type Word = u64; }
//! }
//! ```

#[macro_export]
macro_rules! cpubits {
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

    (
        16 => { $( $tokens16:tt )* }
        32 => { $( $tokens32:tt )* }
        64 => { $( $tokens64:tt )* }
    ) => {
        $crate::cpubits! {
            #[enable_64bit(
                // `cfg` selector for 64-bit targets (implicitly `any`)
                target_family = "wasm",
            )]
            16 => { $( $tokens32 )* }
            32 => { $( $tokens32 )* }
            64 => { $( $tokens64 )* }
        }
    };

    (
        #[enable_64bit( $($enable_64bit:tt)+ )]
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

// Vendored partial copy of the `cfg_if::cfg_if` macro.
// Copyright (c) 2014 Alex Crichton. Dual-licensed Apache 2.0 + MIT.
// TODO(tarcieri): refactor and golf this down to just the parts we use
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
    mod two_arms {
        cpubits! {
            16 | 32 => {
                pub type Word = u32;
            }
            64 => {
                pub type Word = u64;
            }
        }

        #[test]
        fn word_matches_pointer_size() {
            match size_of::<usize>() {
                2 | 4 => {
                    assert_eq!(size_of::<Word>(), 4);
                }
                8 => {
                    assert_eq!(size_of::<Word>(), 8);
                }
                _ => todo!("unsupported target pointer width"),
            }
        }
    }

    mod three_arms {
        cpubits! {
            16 => {
                pub type Word = u16;
            }
            32 => {
                pub type Word = u32;
            }
            64 => {
                pub type Word = u64;
            }
        }

        #[test]
        fn word_matches_pointer_size() {
            assert_eq!(size_of::<Word>(), size_of::<usize>());
        }
    }
}
