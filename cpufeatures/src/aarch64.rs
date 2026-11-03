//! ARM64 CPU feature detection support.
//!
//! Unfortunately ARM instructions to detect CPU features cannot be called from
//! unprivileged userspace code, so this implementation relies on OS-specific
//! APIs for feature detection.

// Evaluate the given `$body` expression any of the supplied target features
// are not enabled. Otherwise returns true.
#[cfg(not(miri))]
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

// Linux runtime detection of target CPU features using `getauxval`.
#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg(not(miri))]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {{
        let hwcap2 = if false $(|| $crate::check!(@hwcap2 $tf))+ {
            $crate::aarch64::getauxval_hwcap2()
        } else { 0 };
        let hwcaps = [$crate::aarch64::getauxval_hwcap(), hwcap2];
        $($crate::check!(@words hwcaps, $tf) & )+ true
    }};
}

/// Linux helper function for calling `getauxval` to get `AT_HWCAP`.
#[cfg(any(target_os = "linux", target_os = "android"))]
#[must_use]
pub fn getauxval_hwcap() -> u64 {
    getauxval(libc::AT_HWCAP)
}

/// Read `AT_HWCAP2` with `getauxval`.
#[cfg(any(target_os = "linux", target_os = "android"))]
#[must_use]
pub fn getauxval_hwcap2() -> u64 {
    getauxval(libc::AT_HWCAP2)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn getauxval(kind: libc::c_ulong) -> u64 {
    // SAFETY: getauxval returns an integer and requires no pointers.
    unsafe { libc::getauxval(kind) }
}

// Apple platform's runtime detection of target CPU features using `sysctlbyname`.
#[cfg(target_vendor = "apple")]
#[cfg(not(miri))]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {{
        $($crate::check!($tf) & )+ true
    }};
}

// Linux `expand_check_macro`
#[cfg(any(target_os = "linux", target_os = "android"))]
macro_rules! __expand_check_macro {
    ($(($name:tt, $hwcap:ident $(, $hwcap2:ident)?)),* $(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                (@hwcap2 $name) => { false $(|| $crate::aarch64::hwcaps::$hwcap2 != 0)? };
                ($hwcaps:expr, $name) => { $crate::check!(@words [$hwcaps, 0], $name) };
                (@words $hwcaps:expr, $name) => {{
                    use $crate::aarch64::hwcaps as caps;
                    let [hwcap, hwcap2] = $hwcaps;
                    let mask = caps::$hwcap;
                    hwcap & mask == mask
                        $( && hwcap2 & caps::$hwcap2 == caps::$hwcap2)?
                        // When scalar FP16 is present, Rust also requires vector FP16.
                        && (mask & 3 != 3 || hwcap & (1 << 9) == 0 || hwcap & (1 << 10) != 0)
                        // When SVE is present, also check its BF16 and I8MM flags.
                        && match $name {
                            "bf16" | "sve-b16b16" if hwcap & caps::SVE == caps::SVE => hwcap2 & caps::SVE_BF16 != 0,
                            "i8mm" if hwcap & caps::SVE == caps::SVE => hwcap2 & caps::SVE_I8MM != 0,
                            _ => true,
                        }
                }};
            )*
        }
    };
}

// Linux `expand_check_macro`
#[cfg(any(target_os = "linux", target_os = "android"))]
__expand_check_macro! {
    ("neon",   NEON),
    ("crc",    CRC),
    ("cssc",   NONE, CSSC),
    ("bf16",   NEON, BF16),
    ("i8mm",   NEON, I8MM),
    ("fcma",   FCMA),
    ("fhm",    FHM),
    ("jsconv", JSCVT),
    ("lse2",   LSE2),
    ("mops",   NONE, MOPS),
    ("sve-b16b16", NEON, SVE_B16B16),
    ("frintts", NEON, FRINTTS),
    ("faminmax", NEON, FAMINMAX),
    ("lut",    NEON, LUT),
    ("rand",   NONE, RAND),
    ("sve",    SVE),
    ("sve2",   SVE, SVE2),
    ("sve2p1", SVE, SVE2P1),
    ("sve2-aes", SVE_AES, SVE2_AES),
    ("sve2-bitperm", SVE, SVE2_BITPERM),
    ("sve2-sha3", SVE_SHA3, SVE2_SHA3),
    ("sve2-sm4", SVE_SM4, SVE2_SM4),
    ("f32mm",  SVE, F32MM),
    ("f64mm",  SVE, F64MM),
    ("aes",    AES),    // Enable AES support.
    ("dit",    DIT),    // Enable Data-Independent Timing support.
    ("sha2",   SHA2),   // Enable SHA1 and SHA256 support.
    ("sha3",   SHA3),   // Enable SHA512 and SHA3 support.
    ("sm4",    SM4),    // Enable SM3 and SM4 support.
    ("sb",     SB),     // Enable Speculation Barrier support
}

/// Linux hardware capabilities mapped to target features.
///
/// Note that LLVM target features are coarser grained than what Linux supports
/// and imply more capabilities under each feature. This module attempts to
/// provide that mapping accordingly.
///
/// See this issue for more info: <https://github.com/RustCrypto/utils/issues/395>
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod hwcaps {
    use libc::c_ulong;

    pub const NONE: c_ulong = 0;
    pub const NEON: c_ulong = libc::HWCAP_FP | libc::HWCAP_ASIMD;
    pub const AES: c_ulong = NEON | libc::HWCAP_AES | libc::HWCAP_PMULL;
    pub const DIT: c_ulong = libc::HWCAP_DIT;
    pub const SHA2: c_ulong = NEON | libc::HWCAP_SHA1 | libc::HWCAP_SHA2;
    pub const SHA3: c_ulong = SHA2 | libc::HWCAP_SHA3 | libc::HWCAP_SHA512;
    pub const SM4: c_ulong = NEON | libc::HWCAP_SM3 | libc::HWCAP_SM4;
    pub const SB: c_ulong = libc::HWCAP_SB;
    pub const CRC: c_ulong = libc::HWCAP_CRC32;
    pub const LSE2: c_ulong = 1 << 25;
    pub const FP16: c_ulong = NEON | libc::HWCAP_FPHP | libc::HWCAP_ASIMDHP;
    pub const SVE: c_ulong = FP16 | libc::HWCAP_SVE;
    pub const FCMA: c_ulong = NEON | (1 << 14);
    pub const FHM: c_ulong = FP16 | (1 << 23);
    pub const JSCVT: c_ulong = NEON | (1 << 13);
    pub const SVE_AES: c_ulong = SVE | AES;
    pub const SVE_SHA3: c_ulong = SVE | SHA3;
    pub const SVE_SM4: c_ulong = SVE | SM4;

    // AT_HWCAP2 masks from asm/hwcap.h; not all libc targets expose them.
    pub const CSSC: c_ulong = 1 << 34;
    pub const MOPS: c_ulong = 1 << 43;
    pub const BF16: c_ulong = 1 << 14;
    pub const I8MM: c_ulong = 1 << 13;
    pub const SVE_BF16: c_ulong = 1 << 12;
    pub const SVE_B16B16: c_ulong = BF16 | (1 << 45);
    pub const SVE_I8MM: c_ulong = 1 << 9;
    pub const SVE2: c_ulong = 1 << 1;
    pub const SVE2_AES: c_ulong = SVE2 | (1 << 2) | (1 << 3);
    pub const SVE2_BITPERM: c_ulong = SVE2 | (1 << 4);
    pub const SVE2_SHA3: c_ulong = SVE2 | (1 << 5);
    pub const SVE2_SM4: c_ulong = SVE2 | (1 << 6);
    pub const F32MM: c_ulong = 1 << 10;
    pub const F64MM: c_ulong = 1 << 11;
    pub const FRINTTS: c_ulong = 1 << 8;
    pub const FAMINMAX: c_ulong = 1 << 50;
    pub const LUT: c_ulong = 1 << 49;
    pub const RAND: c_ulong = 1 << 16;
    pub const SVE2P1: c_ulong = SVE2 | (1 << 36);
}

// Apple OS (macOS, iOS, watchOS, and tvOS) `check!` macro.
//
// NOTE: several of these instructions (e.g. `aes`, `sha2`) can be assumed to
// be present on all Apple ARM64 hardware.
//
// Newer CPU instructions now have nodes within sysctl's `hw.optional`
// namespace, however the ones that do not can safely be assumed to be
// present on all Apple ARM64 devices, now and for the foreseeable future.
//
// See discussion on this issue for more information:
// <https://github.com/RustCrypto/utils/issues/378>
#[cfg(target_vendor = "apple")]
#[macro_export]
#[doc(hidden)]
macro_rules! check {
    ("neon") => {
        true
    };
    ("crc") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.armv8_crc32\0")
            || $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_CRC32\0")
    };
    ("cssc") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_CSSC\0")
    };
    ("bf16") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_BF16\0")
    };
    ("i8mm") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_I8MM\0")
    };
    ("fcma") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FCMA\0")
    };
    ("fhm") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FP16\0")
            && $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FHM\0")
    };
    ("jsconv") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_JSCVT\0")
    };
    ("sve-b16b16") => {
        $crate::check!("bf16")
            && $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_SVE_B16B16\0")
    };
    ("frintts") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FRINTTS\0")
    };
    ("lse2") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_LSE2\0")
    };
    // No public Apple capability key is documented for MOPS.
    ("mops") => { false };
    ("faminmax") => { false };
    ("lut") => { false };
    ("rand") => { false };
    ("sve2p1") => { false };
    ("sve") => { false };
    ("sve2") => { false };
    ("sve2-aes") => { false };
    ("sve2-bitperm") => { false };
    ("sve2-sha3") => { false };
    ("sve2-sm4") => { false };
    ("f32mm") => { false };
    ("f64mm") => { false };
    ("aes") => {
        true
    };
    ("dit") => {
        // https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms#Enable-DIT-for-constant-time-cryptographic-operations
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_DIT\0")
    };
    ("sha2") => {
        true
    };
    ("sha3") => {
        // `sha3` target feature implies SHA-512 as well
        $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_sha512\0")
            && $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_sha3\0")
    };
    ("sm4") => {
        false
    };
    ("sb") => {
        // https://support.arm.com/documentation/ddi0487/mb/-Part-B-The-AArch64-Application-Level-Architecture/-Chapter-B2-The-AArch64-Application-Level-Memory-Model/-B2-6-Memory-barriers/-B2-6-3-Speculation-Barrier
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_SB\0")
    };
}

/// Apple helper function for calling `sysctlbyname`.
///
/// <https://developer.apple.com/documentation/kernel/1387446-sysctlbyname>
///
/// # Panics
/// If `name` is not NUL terminated
#[cfg(target_vendor = "apple")]
#[must_use]
pub fn sysctlbyname(name: &[u8]) -> bool {
    sysctlbyname_with(name, |name, value, size| {
        // SAFETY: The helper checks the name is NUL terminated and supplies an
        // aligned, initialized u32 with its size. Null newp and zero newlen make
        // this a read-only query.
        unsafe {
            libc::sysctlbyname(
                name.as_ptr().cast::<i8>(),
                core::ptr::from_mut(value).cast::<libc::c_void>(),
                size,
                core::ptr::null_mut(),
                0,
            )
        }
    })
}

#[cfg(target_vendor = "apple")]
fn sysctlbyname_with(
    name: &[u8],
    read: impl FnOnce(&[u8], &mut u32, &mut usize) -> libc::c_int,
) -> bool {
    assert_eq!(
        name.last().cloned(),
        Some(0),
        "name is not NUL terminated: {:?}",
        name
    );
    let mut value = 0;
    let mut size = size_of::<u32>();
    let rc = read(name, &mut value, &mut size);
    rc == 0 && size == size_of::<u32>() && value != 0
}

#[cfg(all(test, target_vendor = "apple"))]
mod tests {
    extern crate std;

    #[test]
    fn sysctl_results() {
        for (rc, size, value, expected) in [
            (0, 4, 1, true),
            (0, 4, u32::MAX, true),
            (0, 4, 0, false),
            (-1, 4, 1, false),
            (1, 4, 1, false),
            (0, 0, 1, false),
            (0, 3, 1, false),
            (0, 5, 1, false),
            (0, usize::MAX, 1, false),
        ] {
            assert_eq!(
                super::sysctlbyname_with(b"test\0", |name, out, len| {
                    assert_eq!(name, b"test\0");
                    assert_eq!((*out, *len), (0, 4));
                    *out = value;
                    *len = size;
                    rc
                }),
                expected
            );
        }
        for name in [b"".as_slice(), b"not terminated"] {
            assert!(
                std::panic::catch_unwind(|| super::sysctlbyname_with(name, |_, _, _| 0)).is_err()
            );
        }
    }
}

// On other targets, runtime CPU feature detection is unavailable
#[cfg(not(any(target_vendor = "apple", target_os = "linux", target_os = "android",)))]
#[cfg(not(miri))]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {
        false
    };
}
