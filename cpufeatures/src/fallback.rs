//! Fallback for targets without a runtime CPU feature detection backend.
//!
//! On architectures cpufeatures has no detection support for (e.g.
//! `powerpc`/`powerpc64`), runtime detection is unavailable, so feature checks
//! fall back to compile-time target features only.

// Evaluate the given `$body` expression if any of the supplied target features
// are not enabled at compile time. Otherwise returns true.
#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {
        {
            #[cfg(not(all($(target_feature=$tf,)*)))]
            $body

            #[cfg(all($(target_feature=$tf,)*))]
            true
        }
    };
}

// Runtime CPU feature detection is unavailable on these targets.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {
        false
    };
}
