//! LoongArch64 CPU feature detection support.
//!
//! This implementation relies on OS-specific APIs for feature detection.

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
        let hwcaps = $crate::loongarch64_linux::getauxval_hwcap();
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
                    (($hwcaps & $crate::loongarch64_linux::hwcaps::$hwcap) != 0)
                };
            )*
        }
    };
}

// Linux `expand_check_macro`
__expand_check_macro! {
    ("cpucfg",   CPUCFG),   // Enable CPUCFG support.
    ("lam",      LAM),      // Enable LAM support.
    ("ual",      UAL),      // Enable UAL support.
    ("fpu",      FPU),      // Enable FPU support.
    ("lsx",      LSX),      // Enable LSX support.
    ("lasx",     LASX),     // Enable LASX support.
    ("crc32",    CRC32),    // Enable CRC32 support.
    ("complex",  COMPLEX),  // Enable COMPLEX support.
    ("crypto",   CRYPTO),   // Enable CRYPTO support.
    ("lvz",      LVZ),      // Enable LVZ support.
    ("lbt.x86",  LBT_X86),  // Enable LBT_X86 support.
    ("lbt.arm",  LBT_ARM),  // Enable LBT_ARM support.
    ("lbt.mips", LBT_MIPS), // Enable LBT_MIPS support.
    ("ptw",      PTW),      // Enable PTW support.
}

/// Linux hardware capabilities mapped to target features.
///
/// Note that LLVM target features are coarser grained than what Linux supports
/// and imply more capabilities under each feature. This module attempts to
/// provide that mapping accordingly.
pub mod hwcaps {
    use libc::c_ulong;

    pub const CPUCFG: c_ulong = libc::HWCAP_LOONGARCH_CPUCFG;
    pub const LAM: c_ulong = libc::HWCAP_LOONGARCH_LAM;
    pub const UAL: c_ulong = libc::HWCAP_LOONGARCH_UAL;
    pub const FPU: c_ulong = libc::HWCAP_LOONGARCH_FPU;
    pub const LSX: c_ulong = libc::HWCAP_LOONGARCH_LSX;
    pub const LASX: c_ulong = libc::HWCAP_LOONGARCH_LASX;
    pub const CRC32: c_ulong = libc::HWCAP_LOONGARCH_CRC32;
    pub const COMPLEX: c_ulong = libc::HWCAP_LOONGARCH_COMPLEX;
    pub const CRYPTO: c_ulong = libc::HWCAP_LOONGARCH_CRYPTO;
    pub const LVZ: c_ulong = libc::HWCAP_LOONGARCH_LVZ;
    pub const LBT_X86: c_ulong = libc::HWCAP_LOONGARCH_LBT_X86;
    pub const LBT_ARM: c_ulong = libc::HWCAP_LOONGARCH_LBT_ARM;
    pub const LBT_MIPS: c_ulong = libc::HWCAP_LOONGARCH_LBT_MIPS;
    pub const PTW: c_ulong = libc::HWCAP_LOONGARCH_PTW;
}
