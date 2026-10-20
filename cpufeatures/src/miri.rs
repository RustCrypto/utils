//! CPU detection fallbacks for Miri.
//!
//! Runtime CPU detection stays disabled. Pure ARM64 and x86 guards remain available for tests with supplied CPU and OS data.

#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {
        cfg!(all($(target_feature = $tf,)*))
    };
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {
        false
    };
}
