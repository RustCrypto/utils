//! CPU detection fallbacks for Miri.
//!
//! Runtime CPU detection stays disabled. Pure ARM64 guards remain available for tests with supplied CPU and OS data.

#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {
        cfg!(all($(target_feature = $tf,)*))
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {
        false
    };
}
