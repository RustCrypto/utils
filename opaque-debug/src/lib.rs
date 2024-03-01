//! Macro for opaque `Debug` trait implementation.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

#[doc(hidden)]
pub extern crate core as __core;

#[macro_export]
#[doc(hidden)]
macro_rules! format_params {
    ($single:ident) => {
        "{}"
    };
    ($first:ident, $($rest:ident),+) => {
        concat!("{}", ", ", $crate::format_params!($($rest),+))
    };
}

/// Macro for defining opaque `Debug` implementation.
///
/// It will use the following format: "StructName { ... }". While it's
/// convenient to have it (e.g. for including into other structs), it could be
/// undesirable to leak internal state, which can happen for example through
/// uncareful logging.
#[macro_export]
macro_rules! implement {
    ($struct:ident <$($params:ident),+>) => {
        impl <$($params),+> $crate::__core::fmt::Debug for $struct <$($params),+> {
            fn fmt(
                &self,
                f: &mut $crate::__core::fmt::Formatter,
            ) -> Result<(), $crate::__core::fmt::Error> {
                write!(
                    f,
                    concat!(stringify!($struct), "<", $crate::format_params!($($params),+), "> {{ ... }}"),
                    $($crate::__core::any::type_name::<$params>()),+
                )
            }
        }
    };
    ($struct:ty) => {
        impl $crate::__core::fmt::Debug for $struct {
            fn fmt(
                &self,
                f: &mut $crate::__core::fmt::Formatter,
            ) -> Result<(), $crate::__core::fmt::Error> {
                write!(f, concat!(stringify!($struct), " {{ ... }}"))
            }
        }
    };
}
