//! ARM64 CPU feature detection support.
//!
//! Unfortunately ARM instructions to detect CPU features cannot be called from
//! unprivileged userspace code, so this implementation relies on OS-specific
//! APIs for feature detection.

// Evaluate the given `$body` expression any of the supplied target features
// are not enabled. Otherwise returns true.
#[macro_export]
#[doc(hidden)]
macro_rules! __can_detect {
    ($($tf:tt),+) => {
        true
    };
}

// Linux runtime detection of target CPU features using `getauxval`.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {{
        let hwcaps = $crate::aarch64_linux::getauxval_hwcap();
        $($crate::check!(hwcaps, $tf) & )+ true
    }};
}

/// Linux helper function for calling `getauxval` to get `AT_HWCAP`.
pub fn getauxval_hwcap() -> u64 {
    unsafe { libc::getauxval(libc::AT_HWCAP) }
}

// Linux `expand_check_macro`
macro_rules! __expand_check_macro {
    ($(($name:tt, $hwcap:ident)),* $(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                ($hwcaps:expr, $name) => {
                    (($hwcaps & $crate::aarch64_linux::hwcaps::$hwcap) != 0)
                };
            )*
        }
    };
}

// Linux `expand_check_macro`
__expand_check_macro! {
    ("aes",    AES),    // Enable AES support.
    ("dit",    DIT),    // Enable DIT support.
    ("sha2",   SHA2),   // Enable SHA1 and SHA256 support.
    ("sha3",   SHA3),   // Enable SHA512 and SHA3 support.
    ("sm4",    SM4),    // Enable SM3 and SM4 support.
}

/// Linux hardware capabilities mapped to target features.
///
/// Note that LLVM target features are coarser grained than what Linux supports
/// and imply more capabilities under each feature. This module attempts to
/// provide that mapping accordingly.
///
/// See this issue for more info: <https://github.com/RustCrypto/utils/issues/395>
pub mod hwcaps {
    use libc::c_ulong;

    pub const AES: c_ulong = libc::HWCAP_AES | libc::HWCAP_PMULL;
    pub const DIT: c_ulong = libc::HWCAP_DIT;
    pub const SHA2: c_ulong = libc::HWCAP_SHA2;
    pub const SHA3: c_ulong = libc::HWCAP_SHA3 | libc::HWCAP_SHA512;
    pub const SM4: c_ulong = libc::HWCAP_SM3 | libc::HWCAP_SM4;
}
