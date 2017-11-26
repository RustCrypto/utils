//! Macro for opaque `Debug` trait implementation.
#![no_std]

/// Macro for defining opaque `Debug` implementation. It will use the following
/// format: "HasherName { ... }". While it's convinient to have it
/// (e.g. for including in other structs), it could be undesirable to leak
/// internall state, which can happen for example through uncareful logging.
#[macro_export]
macro_rules! impl_opaque_debug {
    ($state:ty) => {
        use core::fmt;

        impl fmt::Debug for $state {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(f, concat!(stringify!($state), " {{ ... }}"))
            }
        }
    }
}
