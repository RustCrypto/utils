//! ARM64 CPU feature detection support.
//!
//! Unfortunately ARM instructions to detect CPU features cannot be called from
//! unprivileged userspace code, so this implementation relies on OS-specific
//! APIs for feature detection.

include!(concat!(env!("OUT_DIR"), "/feature_cfg.rs"));

// Use features enabled at compile time or detect them at runtime.
#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr) => {
        $crate::__feature_cfg!([$($tf),+] [$($tf,)+] => $body)
    };
    (@check [$($tf:tt),+] => $body:expr) => {{
        #[cfg(not(all($(target_feature=$tf,)*)))]
        {
            $crate::__unless_target_features!(@runtime $body)
        }

        #[cfg(all($(target_feature=$tf,)*))]
        true
    }};
    (@runtime $body:expr) => {{
        #[cfg(not(miri))]
        $body

        #[cfg(miri)]
        false
    }};
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
        let detected = $($crate::check!(@words hwcaps, $tf) & )+ true;
        #[cfg(target_os = "android")]
        let detected = detected
            && ((true $(&& matches!($tf, "neon" | "crc" | "aes" | "sha2"))+)
                || !$crate::aarch64::is_exynos9810());
        detected
    }};
}

/// Read `AT_HWCAP` with `getauxval`.
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

/// Older Exynos 9810 kernels report features that some cores lack.
#[cfg(all(target_os = "android", not(miri)))]
pub fn is_exynos9810() -> bool {
    let mut arch = [0_u8; libc::PROP_VALUE_MAX as usize];
    // SAFETY: The property name ends in NUL and the buffer holds PROP_VALUE_MAX bytes.
    let len = unsafe { libc::__system_property_get(c"ro.arch".as_ptr(), arch.as_mut_ptr().cast()) };
    len > 0 && arch.starts_with(b"exynos9810")
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
            // Linux has no capability bit for FEAT_PAuth_LR.
            (@hwcap2 "pauth-lr") => { false };
            (@words $hwcaps:expr, "pauth-lr") => {{
                let _ = $hwcaps;
                false
            }};
            ($hwcaps:expr, "pauth-lr") => {{
                let _ = $hwcaps;
                false
            }};
            $(
                (@hwcap2 $name) => { false $(|| $crate::aarch64::hwcaps::$hwcap2 != 0)? };
                ($hwcaps:expr, $name) => { $crate::check!(@words [$hwcaps, 0], $name) };
                (@words $hwcaps:expr, $name) => {{
                    use $crate::aarch64::hwcaps as caps;
                    let [hwcap, hwcap2] = $hwcaps;
                    let mask = caps::$hwcap;
                    let mask2 = 0 $(| caps::$hwcap2)?;
                    hwcap & mask == mask
                        && hwcap2 & mask2 == mask2
                        // When scalar FP16 is present, Rust also requires vector FP16.
                        && (mask & 3 != 3 || hwcap & (1 << 9) == 0 || hwcap & (1 << 10) != 0)
                        // If SVE is available, require its BF16 or I8MM flag too.
                        && (hwcap & caps::SVE != caps::SVE
                            || ((mask2 & caps::BF16 == 0 || hwcap2 & caps::SVE_BF16 != 0)
                                && (mask2 & caps::I8MM == 0 || hwcap2 & caps::SVE_I8MM != 0)))
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
    ("bti",    NONE, BTI),
    ("dpb",    DPB),
    ("dpb2",   DPB, DPB2),
    ("ecv",    NONE, ECV),
    ("flagm",  FLAGM),
    ("flagm2", FLAGM, FLAGM2),
    ("hbc",    NONE, HBC),
    ("mte",    NONE, MTE),
    ("paca",   PAUTH),
    ("pacg",   PAUTH),
    ("rcpc",   RCPC),
    ("rcpc2",  RCPC2),
    ("rcpc3",  RCPC2, RCPC3),
    ("rdm",    RDM),
    ("ssbs",   SSBS),
    ("wfxt",   NONE, WFXT),
    ("dotprod", DOTPROD),
    ("bf16",   NEON, BF16),
    ("i8mm",   NEON, I8MM),
    ("fcma",   FCMA),
    ("fp16",   FP16),
    ("fhm",    FHM),
    ("jsconv", JSCVT),
    ("lse",    LSE),
    ("lse128", LSE, LSE128),
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
    ("fp8",    NEON, FP8),
    ("fp8fma", NEON, FP8FMA),
    ("fp8dot4", NEON, FP8DOT4),
    ("fp8dot2", NEON, FP8DOT2),
    ("fpmr",   NONE, FPMR),
    ("sme",    FP16, SME),
    ("sme-i16i64", FP16, SME_I16I64),
    ("sme-f64f64", FP16, SME_F64F64),
    ("sme-fa64", SVE, SME_FA64),
    ("sme2",   FP16, SME2),
    ("sme2p1", FP16, SME2P1),
    ("sme-b16b16", FP16, SME_B16B16),
    ("sme-f16f16", FP16, SME_F16F16),
    ("sme-lutv2", FP16, SME_LUTV2),
    ("sme-f8f32", FP16, SME_F8F32),
    ("sme-f8f16", FP16, SME_F8F16),
    ("ssve-fp8fma", FP16, SSVE_FP8FMA),
    ("ssve-fp8dot4", FP16, SSVE_FP8DOT4),
    ("ssve-fp8dot2", FP16, SSVE_FP8DOT2),
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
    pub const DOTPROD: c_ulong = NEON | libc::HWCAP_ASIMDDP;
    pub const LSE: c_ulong = libc::HWCAP_ATOMICS;
    pub const DPB: c_ulong = 1 << 16;
    pub const FLAGM: c_ulong = 1 << 27;
    pub const PAUTH: c_ulong = (1 << 30) | (1 << 31);
    pub const RCPC: c_ulong = 1 << 15;
    pub const RCPC2: c_ulong = RCPC | (1 << 26);
    pub const RDM: c_ulong = NEON | (1 << 12);
    pub const SSBS: c_ulong = 1 << 28;
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
    pub const BTI: c_ulong = 1 << 17;
    pub const DPB2: c_ulong = 1;
    pub const ECV: c_ulong = 1 << 19;
    pub const FLAGM2: c_ulong = 1 << 7;
    pub const HBC: c_ulong = 1 << 44;
    pub const LSE128: c_ulong = 1 << 47;
    pub const FPMR: c_ulong = 1 << 48;
    pub const MTE: c_ulong = 1 << 18;
    pub const RCPC3: c_ulong = 1 << 46;
    pub const WFXT: c_ulong = 1 << 31;
    // FEAT_FP8 requires conversions and both FP8 formats.
    pub const FP8: c_ulong = BF16 | FPMR | LUT | FAMINMAX | (1 << 51) | (1 << 55) | (1 << 56);
    pub const FP8FMA: c_ulong = FP8 | (1 << 52);
    pub const FP8DOT4: c_ulong = FP8FMA | (1 << 53);
    pub const FP8DOT2: c_ulong = FP8DOT4 | (1 << 54);
    pub const SME: c_ulong = BF16 | (1 << 23);
    pub const SME_I16I64: c_ulong = SME | (1 << 24);
    pub const SME_F64F64: c_ulong = SME | (1 << 25);
    pub const SME_FA64: c_ulong = SME | SVE2 | (1 << 30);
    pub const SME2: c_ulong = SME | (1 << 37);
    pub const SME2P1: c_ulong = SME2 | (1 << 38);
    pub const SME_B16B16: c_ulong = SME2 | SVE_B16B16 | (1 << 41);
    pub const SME_F16F16: c_ulong = SME2 | (1 << 42);
    pub const SME_LUTV2: c_ulong = SME2 | (1 << 57);
    pub const SME_F8F32: c_ulong = SME2 | FP8 | (1 << 59);
    pub const SME_F8F16: c_ulong = SME_F8F32 | (1 << 58);
    pub const SSVE_FP8FMA: c_ulong = SME2 | FP8 | (1 << 60);
    pub const SSVE_FP8DOT4: c_ulong = SSVE_FP8FMA | (1 << 61);
    pub const SSVE_FP8DOT2: c_ulong = SSVE_FP8DOT4 | (1 << 62);
}

// Apple ARM64 provides NEON, AES, SHA-1, and SHA-256.
// Read sysctl for optional features.
//
// See discussion on this issue for more information:
// <https://github.com/RustCrypto/utils/issues/378>
#[cfg(target_vendor = "apple")]
#[macro_export]
#[doc(hidden)]
macro_rules! check {
    (@feat $name:literal) => {
        $crate::aarch64::sysctlbyname(concat!("hw.optional.arm.FEAT_", $name, "\0").as_bytes())
    };
    ("bti") => {
        $crate::check!(@feat "BTI")
    };
    ("dpb") => {
        $crate::check!(@feat "DPB")
    };
    ("dpb2") => {
        $crate::check!("dpb") && $crate::check!(@feat "DPB2")
    };
    ("ecv") => {
        $crate::check!(@feat "ECV")
    };
    ("flagm") => {
        $crate::check!(@feat "FlagM")
    };
    ("flagm2") => {
        $crate::check!("flagm") && $crate::check!(@feat "FlagM2")
    };
    ("hbc") => {
        $crate::check!(@feat "HBC")
    };
    ("mte") => {
        $crate::check!(@feat "MTE") && $crate::check!(@feat "MTE2")
    };
    ("paca") => {
        $crate::check!(@feat "PAuth")
    };
    ("pacg") => {
        $crate::check!("paca")
    };
    ("pauth-lr") => {
        $crate::check!("paca") && $crate::check!(@feat "PAuth_LR")
    };
    ("rcpc") => {
        $crate::check!(@feat "LRCPC")
    };
    ("rcpc2") => {
        $crate::check!("rcpc") && $crate::check!(@feat "LRCPC2")
    };
    ("rcpc3") => { false };
    ("rdm") => {
        $crate::check!(@feat "RDM")
    };
    ("ssbs") => {
        $crate::check!(@feat "SSBS")
    };
    ("wfxt") => {
        $crate::check!(@feat "WFxT")
    };
    ("lse128") => { false };
    ("fp8") => {
        $crate::check!("bf16")
            && $crate::check!("lut")
            && $crate::check!("faminmax")
            && $crate::check!("fpmr")
            && $crate::check!(@feat "FP8")
    };
    ("fp8fma") => { false };
    ("fp8dot4") => { false };
    ("fp8dot2") => { false };
    ("fpmr") => {
        $crate::check!(@feat "FPMR")
    };
    ("sme") => {
        $crate::check!("bf16")
            && $crate::check!("fp16")
            && $crate::check!(@feat "SME")
    };
    ("sme-i16i64") => {
        $crate::check!("sme") && $crate::check!(@feat "SME_I16I64")
    };
    ("sme-f64f64") => {
        $crate::check!("sme") && $crate::check!(@feat "SME_F64F64")
    };
    ("sme-fa64") => { false };
    ("sme2") => {
        $crate::check!("sme") && $crate::check!(@feat "SME2")
    };
    ("sme2p1") => {
        $crate::check!("sme2") && $crate::check!(@feat "SME2p1")
    };
    ("sme-b16b16") => {
        $crate::check!("sme2")
            && $crate::check!("sve-b16b16")
            && $crate::check!(@feat "SME_B16B16")
    };
    ("sme-f16f16") => {
        $crate::check!("sme2") && $crate::check!(@feat "SME_F16F16")
    };
    ("sme-lutv2") => {
        $crate::check!("sme2") && $crate::check!(@feat "SME_LUTv2")
    };
    ("sme-f8f32") => {
        $crate::check!("sme2")
            && $crate::check!("fp8")
            && $crate::check!(@feat "SME_F8F32")
    };
    ("sme-f8f16") => {
        $crate::check!("sme-f8f32") && $crate::check!(@feat "SME_F8F16")
    };
    ("ssve-fp8fma") => { false };
    ("ssve-fp8dot4") => { false };
    ("ssve-fp8dot2") => { false };

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
    ("dotprod") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_DotProd\0")
    };
    ("bf16") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_BF16\0")
    };
    ("i8mm") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_I8MM\0")
    };
    ("fcma") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FCMA\0")
            || $crate::aarch64::sysctlbyname(b"hw.optional.armv8_3_compnum\0")
    };
    ("fp16") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FP16\0")
            || $crate::aarch64::sysctlbyname(b"hw.optional.neon_fp16\0")
    };
    ("fhm") => {
        $crate::check!("fp16")
            && ($crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FHM\0")
                || $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_fhm\0"))
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
    ("lse") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_LSE\0")
            || $crate::aarch64::sysctlbyname(b"hw.optional.armv8_1_atomics\0")
    };
    ("lse2") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_LSE2\0")
    };
    // No public Apple capability key is documented for MOPS.
    ("mops") => { false };
    ("faminmax") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_FAMINMAX\0")
    };
    ("lut") => {
        $crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_LUT\0")
    };
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
        // Rust's `sha3` feature requires SHA-512.
        ($crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_SHA512\0")
            || $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_sha512\0"))
            && ($crate::aarch64::sysctlbyname(b"hw.optional.arm.FEAT_SHA3\0")
                || $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_sha3\0"))
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

    #[cfg(all(target_os = "macos", not(miri)))]
    #[test]
    fn sysctl_reads_existing_and_missing_keys() {
        assert!(!super::sysctlbyname(b"hw.optional.cpufeatures_missing\0"));

        // Query a CPU count so the test needs no optional CPU features.
        let name = c"hw.ncpu";
        let mut count = 0u32;
        let mut size = size_of::<u32>();
        // SAFETY: The name is NUL terminated, and count and size describe a
        // writable buffer. Null newp and zero newlen make this read-only.
        let rc = unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                core::ptr::from_mut(&mut count).cast(),
                &raw mut size,
                core::ptr::null_mut(),
                0,
            )
        };
        assert_eq!(
            super::sysctlbyname(name.to_bytes_with_nul()),
            rc == 0 && size == size_of::<u32>() && count > 0
        );
    }

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
