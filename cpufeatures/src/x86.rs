//! x86/x86-64 CPU feature detection support.
//!
//! Portable, `no_std`-friendly implementation that relies on the x86 `CPUID`
//! instruction for feature detection.

/// Evaluate the given `$body` expression any of the supplied target features
/// are not enabled. Otherwise returns true.
///
/// The `$body` expression is not evaluated on SGX targets, and returns false
/// on these targets unless *all* supplied target features are enabled.
#[cfg(not(miri))]
#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {{
        #[cfg(not(all($(target_feature=$tf,)*)))]
        {
            #[cfg(not(any(target_env = "sgx", target_os = "none", target_os = "uefi")))]
            $body

            // CPUID is not available on SGX. Freestanding and UEFI targets
            // do not support SIMD features with default compilation flags.
            #[cfg(any(target_env = "sgx", target_os = "none", target_os = "uefi"))]
            false
        }

        #[cfg(all($(target_feature=$tf,)*))]
        true
    }};
}

/// Use CPUID to detect the presence of all supplied target features.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    (@cpuid $cpuid:expr, $extended:expr, $xstate:expr) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::CpuidResult;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::CpuidResult;
        let cpuid = $cpuid;
        let zero = CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 };
        let max = cpuid(0, 0).eax;
        let basic = if max >= 1 { cpuid(1, 0) } else { zero };
        let leaf7 = if max >= 7 { cpuid(7, 0) } else { zero };
        let subleaf = if max >= 7 && leaf7.eax >= 1 { cpuid(7, 1) } else { zero };
        let extended = if $extended && cpuid(0x8000_0000, 0).eax >= 0x8000_0001 { cpuid(0x8000_0001, 0) } else { zero };
        let xstate = if $xstate && max >= 0xd && basic.ecx & (3 << 26) == 3 << 26 { cpuid(0xd, 1) } else { zero };
        [basic, leaf7, subleaf, extended, xstate]
    }};
    ($($tf:tt),+) => {{
        #[cfg(miri)]
        { false }
        #[cfg(not(miri))]
        {
            #[cfg(target_arch = "x86")]
            use core::arch::x86 as arch;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64 as arch;
            let extended = false $(|| matches!($tf, "lzcnt" | "lahfsahf" | "prfchw"))+;
            let xstate = false $(|| matches!($tf, "xsavec" | "xsaveopt" | "xsaves"))+;
            // SAFETY: CPUID provides the real CPU flags used to guard XGETBV below.
            unsafe {
                let cr = $crate::__detect_target_features!(@cpuid arch::__cpuid_count, extended, xstate);
                $($crate::check!(cr, $tf) & )+ true
            }
        }
    }};
}

/// Check that the OS supports the required SIMD registers.
///
/// # Safety
/// The supplied CPUID flags must describe the current CPU and OS.
/// Safe callers cannot supply flags to the hardware query:
///
/// ```compile_fail,E0133
/// # #[cfg(target_arch = "x86")] use core::arch::x86::CpuidResult;
/// # #[cfg(target_arch = "x86_64")] use core::arch::x86_64::CpuidResult;
/// let cr = [CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 }; 5];
/// cpufeatures::__xgetbv!(cr, 6);
/// ```
///
/// The feature check also requires an unsafe context when given flags:
///
/// ```compile_fail,E0133
/// # #[cfg(target_arch = "x86")] use core::arch::x86::CpuidResult;
/// # #[cfg(target_arch = "x86_64")] use core::arch::x86_64::CpuidResult;
/// let cr = [CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 }; 5];
/// cpufeatures::check!(cr, "avx");
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __xgetbv {
    ($cr:expr, $mask:expr) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86 as arch;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64 as arch;

        $crate::__xgetbv!(@read $cr, $mask, arch::_xgetbv(arch::_XCR_XFEATURE_ENABLED_MASK))
    }};
    (@read $cr:expr, $mask:expr, $xcr0:expr) => {{
        // Check XSAVE and OSXSAVE before reading XCR0.
        let xmask = 0b11 << 26;
        let xsave = $cr[0].ecx & xmask == xmask;
        if xsave {
            let xcr0 = $xcr0;
            (xcr0 & $mask) == $mask
        } else {
            false
        }
    }};
}

macro_rules! __expand_check_macro {
    ($(($name:tt, $reg_cap:tt $(, $i:expr, $reg:ident, $offset:expr)*)),* $(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                ($cr:expr, $name) => {{
                    let cr = $cr;
                    $crate::check!(cr, $name, |mask| $crate::__xgetbv!(cr, mask))
                }};
                ($cr:expr, $name, $os:expr) => {{
                    let cr = $cr;
                    // Check Rust's requirements for AVX and AVX-512.
                    let avx = (1 << 0) | (1 << 9) | (1 << 19) | (1 << 20)
                        | (1 << 26) | (1 << 27) | (1 << 28);
                    let sse = (1 << 25) | (1 << 26);
                    let registers = match $reg_cap {
                        "xsave" => cr[0].ecx & (3 << 26) == 3 << 26,
                        "sse2" => cr[0].edx & sse == sse,
                        "ymm" | "zmm" => cr[0].ecx & avx == avx && cr[0].edx & sse == sse,
                        _ => true,
                    };
                    let mask = match $reg_cap { "ymm" => 0b110, "zmm" => 0b1110_0110, _ => 0 };
                    registers
                        $(& (cr[$i].$reg & (1 << $offset) != 0))*
                        && ($reg_cap != "zmm" || (cr[0].ecx & ((1 << 12) | (1 << 29)) == ((1 << 12) | (1 << 29))
                            && cr[1].ebx & ((1 << 5) | (1 << 16)) == ((1 << 5) | (1 << 16))))
                        && (mask == 0 || ($os)(mask))
                }};
            )*
        }
    };
}

__expand_check_macro! {
    ("sse3", "", 0, ecx, 0),
    ("pclmulqdq", "", 0, ecx, 1),
    ("ssse3", "", 0, ecx, 9),
    ("fma", "ymm", 0, ecx, 12, 0, ecx, 28),
    ("cmpxchg16b", "", 0, ecx, 13),
    ("sse4.1", "", 0, ecx, 19),
    ("sse4.2", "", 0, ecx, 20),
    ("movbe", "", 0, ecx, 22),
    ("popcnt", "", 0, ecx, 23),
    ("aes", "", 0, ecx, 25),
    ("xsave", "xsave"),
    ("avx", "ymm", 0, ecx, 28),
    ("f16c", "ymm", 0, ecx, 29),
    ("rdrand", "", 0, ecx, 30),

    ("mmx", "", 0, edx, 23),
    ("fxsr", "", 0, edx, 24),
    ("sse", "", 0, edx, 25),
    ("sse2", "", 0, edx, 26),

    ("sgx", "", 1, ebx, 2),
    ("bmi1", "", 1, ebx, 3),
    ("bmi2", "", 1, ebx, 8),
    ("ermsb", "", 1, ebx, 9),
    ("avx2", "ymm", 1, ebx, 5, 0, ecx, 28),
    ("avx512f", "zmm", 1, ebx, 16),
    ("avx512dq", "zmm", 1, ebx, 17),
    ("rdseed", "", 1, ebx, 18),
    ("adx", "", 1, ebx, 19),
    ("avx512ifma", "zmm", 1, ebx, 21),
    ("avx512pf", "zmm", 1, ebx, 26),
    ("avx512er", "zmm", 1, ebx, 27),
    ("avx512cd", "zmm", 1, ebx, 28),
    ("sha", "", 1, ebx, 29),
    ("avx512bw", "zmm", 1, ebx, 30),
    ("avx512vl", "zmm", 1, ebx, 31),
    ("avx512vnni", "zmm", 1, ecx, 11),
    ("avx512vp2intersect", "zmm", 1, edx, 8),
    ("avx512fp16", "zmm", 1, edx, 23, 1, ebx, 30),
    ("avx512vbmi", "zmm", 1, ecx, 1, 1, ebx, 30),
    ("avx512vbmi2", "zmm", 1, ecx, 6, 1, ebx, 30),
    ("gfni", "sse2", 1, ecx, 8),
    ("vaes", "ymm", 1, ecx, 9, 0, ecx, 25, 1, ebx, 5),
    ("vpclmulqdq", "ymm", 1, ecx, 10, 0, ecx, 1),
    ("avx512bitalg", "zmm", 1, ecx, 12, 1, ebx, 30),
    ("avx512vpopcntdq", "zmm", 1, ecx, 14),

    ("sha512", "ymm", 2, eax, 0, 1, ebx, 5),
    ("sm3", "ymm", 2, eax, 1),
    ("sm4", "ymm", 2, eax, 2, 1, ebx, 5),
    ("avxvnni", "ymm", 2, eax, 4, 1, ebx, 5),
    ("avx512bf16", "zmm", 2, eax, 5, 1, ebx, 30),
    ("avxifma", "ymm", 2, eax, 23, 1, ebx, 5),
    ("avxvnniint8", "ymm", 2, edx, 4, 1, ebx, 5),
    ("avxneconvert", "ymm", 2, edx, 5, 1, ebx, 5),
    ("avxvnniint16", "ymm", 2, edx, 10, 1, ebx, 5),

    ("lahfsahf", "", 3, ecx, 0),
    ("lzcnt", "", 3, ecx, 5),
    ("prfchw", "", 3, ecx, 8),

    ("xsaveopt", "xsave", 4, eax, 0),
    ("xsavec", "xsave", 4, eax, 1),
    ("xsaves", "xsave", 4, eax, 3),
}
