//! Compile-time fallback based on `cfg(target_feature = "..")`

#[macro_export]
#[doc(hidden)]
macro_rules! __can_detect {
    ($($tf:tt),+) => {
        false
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {
        false
    };
}
