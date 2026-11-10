//! x86/x86-64 CPU feature detection support.
//!
//! Portable, `no_std`-friendly implementation that relies on the x86 `CPUID`
//! instruction for feature detection.

include!(concat!(env!("OUT_DIR"), "/feature_cfg.rs"));

/// Use features enabled at compile time or detect them at runtime.
#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {{
        if cfg!(target_arch = "x86") && (false $(|| matches!($tf, "cmpxchg16b" | "apxf"))+) {
            false
        } else {
            $crate::__feature_cfg!([$($tf),+] [$($tf,)+] => $body)
        }
    }};
    (@check [$($tf:tt),+] => $body:expr) => {{
        #[cfg(not(all($(target_feature=$tf,)*)))]
        {
            $crate::__unless_target_features!(@runtime $body)
        }

        #[cfg(all($(target_feature=$tf,)*))]
        true
    }};
    (@runtime $body:expr) => {{
        #[cfg(not(any(miri, target_env = "sgx", target_os = "none", target_os = "uefi")))]
        $body

        #[cfg(any(miri, target_env = "sgx", target_os = "none", target_os = "uefi"))]
        false
    }};
}

/// Use CPUID to detect the presence of all supplied target features.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    (@cpuid $cpuid:expr, $leaves:expr) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::CpuidResult;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::CpuidResult;
        let cpuid = $cpuid;
        let leaves = $leaves;
        let zero = CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 };
        let info = cpuid(0, 0);
        let vendor = (info.ebx.to_le_bytes(), info.edx.to_le_bytes(), info.ecx.to_le_bytes());
        let intel = vendor == (*b"Genu", *b"ineI", *b"ntel");
        let max = info.eax;
        let mut basic = if max >= 1 { cpuid(1, 0) } else { zero };
        // Intel erratum R85: Pentium 4 LD0 reports CX16 without supporting it.
        if intel && basic.eax == 0x0f34 {
            basic.ecx &= !(1 << 13);
        }
        let mut leaf7 = if leaves & ((1 << 1) | (1 << 2) | (1 << 5) | (1 << 6)) != 0 && max >= 7 {
            cpuid(7, 0)
        } else { zero };
        // Intel erratum SKL052: Some Skylake Pentium and Celeron CPUs report BMI in error.
        if intel && matches!(basic.eax >> 4, 0x406e | 0x506e) && basic.ecx & (1 << 28) == 0 {
            leaf7.ebx &= !((1 << 3) | (1 << 8));
        }
        let subleaf = if leaves & ((1 << 2) | (1 << 6)) != 0 && leaf7.eax >= 1 {
            cpuid(7, 1)
        } else { zero };
        let extended = if leaves & (1 << 3) != 0 && cpuid(0x8000_0000, 0).eax >= 0x8000_0001 {
            let mut extended = cpuid(0x8000_0001, 0);
            if vendor != (*b"Auth", *b"enti", *b"cAMD")
                && vendor != (*b"Hygo", *b"nGen", *b"uine")
            {
                extended.ecx &= !((1 << 6) | (1 << 11) | (1 << 16) | (1 << 21));
            }
            extended
        } else { zero };
        let xstate = if leaves & (1 << 4) != 0 && max >= 0xd && basic.ecx & (3 << 26) == 3 << 26 {
            cpuid(0xd, 1)
        } else { zero };
        let keylocker = if leaves & (1 << 5) != 0 && max >= 0x19 && leaf7.ecx & (1 << 23) != 0 {
            cpuid(0x19, 0)
        } else { zero };
        let avx10 = if leaves & (1 << 6) != 0 && max >= 0x24 && subleaf.edx & (1 << 19) != 0 {
            cpuid(0x24, 0)
        } else { zero };
        [basic, leaf7, subleaf, extended, xstate, keylocker, avx10]
    }};
    (@check $cr:ident, $read:expr; $($tf:tt),+) => {{
        // Collect the required register bits before reading XCR0.
        let mut required = 0;
        let supported = true $(&& $crate::check!($cr, $tf, |mask| {
            required |= mask;
            true
        }))+;
        supported && (required == 0 || ($read)(required))
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
            let leaves = 0 $(| $crate::check!(@leaves $tf))+;
            // SAFETY: CPUID provides the real CPU flags used to guard XGETBV below.
            unsafe {
                let cr = $crate::__detect_target_features!(@cpuid arch::__cpuid_count, leaves);
                $crate::__detect_target_features!(
                    @check cr, |mask| $crate::__xgetbv!(cr, mask); $($tf),+
                )
            }
        }
    }};
}

/// Check that the OS supports the required register state.
///
/// # Safety
/// The supplied CPUID flags must describe the current CPU and OS.
/// The hardware query requires an unsafe context.
///
/// ```compile_fail,E0133
/// # #[cfg(target_arch = "x86")] use core::arch::x86::CpuidResult;
/// # #[cfg(target_arch = "x86_64")] use core::arch::x86_64::CpuidResult;
/// let cr = [CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 }; 7];
/// cpufeatures::__xgetbv!(cr, 6);
/// ```
///
/// The feature check also requires an unsafe context when the caller supplies flags.
///
/// ```compile_fail,E0133
/// # #[cfg(target_arch = "x86")] use core::arch::x86::CpuidResult;
/// # #[cfg(target_arch = "x86_64")] use core::arch::x86_64::CpuidResult;
/// let cr = [CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 }; 7];
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
            (@version "avx10.1", $cr:ident) => { $crate::check!(@avx10 $cr, 1) };
            (@version "avx10.2", $cr:ident) => { $crate::check!(@avx10 $cr, 2) };
            (@version $feature:tt, $cr:ident) => { true };
            (@avx10 $cr:ident, $version:expr) => {{
                // Rust's AVX10 levels imply these AVX-512 features in addition to AVX512F.
                let ebx = (1 << 17) | (1 << 21) | (1 << 28) | (1 << 30) | (1 << 31);
                let ecx = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 12) | (1 << 14);
                $cr[6].ebx & 0xff >= $version
                    && $cr[1].ebx & ebx == ebx
                    && $cr[1].ecx & ecx == ecx
                    && $cr[1].edx & (1 << 23) != 0 // AVX512FP16
                    && $cr[2].eax & (1 << 5) != 0 // AVX512BF16
            }};
            $(
                (@leaves $name) => { 0 $(| (1 << $i))* };
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
                    let avx512_ecx = (1 << 12) | (1 << 29);
                    let avx512_ebx = (1 << 5) | (1 << 16);
                    let registers = match $reg_cap {
                        "x86_64" => cfg!(target_arch = "x86_64"),
                        "xsave" => cr[0].ecx & (3 << 26) == 3 << 26,
                        "apx" => cfg!(target_arch = "x86_64") && cr[0].ecx & (3 << 26) == 3 << 26,
                        "sse2" => cr[0].edx & sse == sse,
                        // Microcode can force every RTM transaction to abort.
                        "rtm" => cr[1].edx & (1 << 11) == 0,
                        "ymm" | "zmm" => cr[0].ecx & avx == avx && cr[0].edx & sse == sse,
                        _ => true,
                    };
                    let mask = match $reg_cap {
                        "ymm" => 0b110,
                        "zmm" => 0b1110_0110,
                        "apx" => 1 << 19,
                        _ => 0,
                    };
                    registers
                        $(& (cr[$i].$reg & (1 << $offset) != 0))*
                        && ($reg_cap != "zmm"
                            || (cr[0].ecx & avx512_ecx == avx512_ecx
                                && cr[1].ebx & avx512_ebx == avx512_ebx))
                        && $crate::check!(@version $name, cr)
                        && (mask == 0 || ($os)(mask))
                }};
            )*
        }
    };
}

__expand_check_macro! {
    ("sse3", "", 0, ecx, 0),
    ("pclmulqdq", "sse2", 0, ecx, 1),
    ("ssse3", "", 0, ecx, 9),
    ("fma", "ymm", 0, ecx, 12, 0, ecx, 28),
    ("cmpxchg16b", "x86_64", 0, ecx, 13),
    ("sse4.1", "", 0, ecx, 19),
    ("sse4.2", "", 0, ecx, 20),
    ("movbe", "", 0, ecx, 22),
    ("popcnt", "", 0, ecx, 23),
    ("aes", "sse2", 0, ecx, 25),
    ("xsave", "xsave"),
    ("avx", "ymm", 0, ecx, 28),
    ("f16c", "ymm", 0, ecx, 29),
    ("rdrand", "", 0, ecx, 30),

    ("x87", "", 0, edx, 0),
    ("mmx", "", 0, edx, 23),
    ("fxsr", "", 0, edx, 24),
    ("sse", "", 0, edx, 25),
    ("sse2", "", 0, edx, 26),

    ("sgx", "", 1, ebx, 2),
    ("bmi1", "", 1, ebx, 3),
    ("avx2", "ymm", 1, ebx, 5, 0, ecx, 28),
    ("bmi2", "", 1, ebx, 8),
    ("ermsb", "", 1, ebx, 9),
    ("rtm", "rtm", 1, ebx, 11),
    ("clflushopt", "", 1, ebx, 23),
    ("avx512f", "zmm", 1, ebx, 16),
    ("avx512dq", "zmm", 1, ebx, 17),
    ("rdseed", "", 1, ebx, 18),
    ("adx", "", 1, ebx, 19),
    ("avx512ifma", "zmm", 1, ebx, 21),
    ("avx512pf", "zmm", 1, ebx, 26),
    ("avx512er", "zmm", 1, ebx, 27),
    ("avx512cd", "zmm", 1, ebx, 28),
    ("sha", "sse2", 1, ebx, 29),
    ("avx512bw", "zmm", 1, ebx, 30),
    ("avx512vl", "zmm", 1, ebx, 31),
    ("avx512vbmi", "zmm", 1, ecx, 1, 1, ebx, 30),
    ("avx512vbmi2", "zmm", 1, ecx, 6, 1, ebx, 30),
    ("gfni", "sse2", 1, ecx, 8),
    ("vaes", "ymm", 1, ecx, 9, 0, ecx, 25, 1, ebx, 5),
    ("vpclmulqdq", "ymm", 1, ecx, 10, 0, ecx, 1),
    ("avx512vnni", "zmm", 1, ecx, 11),
    ("avx512bitalg", "zmm", 1, ecx, 12, 1, ebx, 30),
    ("avx512vpopcntdq", "zmm", 1, ecx, 14),
    ("movdiri", "", 1, ecx, 27),
    ("movdir64b", "", 1, ecx, 28),
    ("avx512vp2intersect", "zmm", 1, edx, 8),
    ("avx512fp16", "zmm", 1, edx, 23, 1, ebx, 30),

    ("sha512", "ymm", 2, eax, 0, 1, ebx, 5),
    ("sm3", "ymm", 2, eax, 1),
    ("sm4", "ymm", 2, eax, 2, 1, ebx, 5),
    ("avxvnni", "ymm", 2, eax, 4, 1, ebx, 5),
    ("avx512bf16", "zmm", 2, eax, 5, 1, ebx, 30),
    ("avxifma", "ymm", 2, eax, 23, 1, ebx, 5),
    ("movrs", "", 2, eax, 31),
    ("avxvnniint8", "ymm", 2, edx, 4, 1, ebx, 5),
    ("avxneconvert", "ymm", 2, edx, 5, 1, ebx, 5),
    ("avxvnniint16", "ymm", 2, edx, 10, 1, ebx, 5),
    ("apxf", "apx", 2, edx, 21),

    ("lahfsahf", "", 3, ecx, 0),
    ("lzcnt", "", 3, ecx, 5),
    ("sse4a", "sse2", 3, ecx, 6, 0, ecx, 0),
    ("prfchw", "", 3, ecx, 8),
    ("xop", "ymm", 3, ecx, 11, 3, ecx, 16, 3, ecx, 6),
    ("fma4", "ymm", 3, ecx, 16, 3, ecx, 6),
    ("tbm", "", 3, ecx, 21),

    ("xsaveopt", "xsave", 4, eax, 0),
    ("xsavec", "xsave", 4, eax, 1),
    ("xsaves", "xsave", 4, eax, 3),

    ("kl", "sse2", 1, ecx, 23, 5, ebx, 0),
    ("widekl", "sse2", 1, ecx, 23, 5, ebx, 0, 5, ebx, 2),

    // Reject guests that report AVX10 without 512-bit vectors.
    ("avx10.1", "zmm", 2, edx, 19, 6, ebx, 18),
    ("avx10.2", "zmm", 2, edx, 19, 6, ebx, 18, 2, eax, 4, 2, edx, 4, 2, edx, 10),
}
