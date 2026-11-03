//! x86/x86-64 CPU feature detection support.
//!
//! Portable, `no_std`-friendly implementation that relies on the x86 `CPUID`
//! instruction for feature detection.

/// Evaluate the given `$body` expression any of the supplied target features
/// are not enabled. Otherwise returns true.
///
/// The `$body` expression is not evaluated on SGX targets, and returns false
/// on these targets unless *all* supplied target features are enabled.
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

#[cfg(target_arch = "x86")]
use core::arch::x86::CpuidResult;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::CpuidResult;

const ZERO_CPUID: CpuidResult = CpuidResult {
    eax: 0,
    ebx: 0,
    ecx: 0,
    edx: 0,
};

/// Collect the CPUID leaves used for feature detection: leaf 1, and leaf 7
/// subleaves 0 and 1.
///
/// Querying a leaf beyond the CPU's maximum supported basic leaf does not
/// return zero: the CPU instead re-returns the highest supported leaf's data
/// (Intel SDM Vol. 2A, "CPUID"), which `check!` would otherwise misinterpret
/// as feature bits. The CPUID sources are parameterized so this guard can be
/// exercised with synthetic data in tests.
#[doc(hidden)]
pub fn __collect_leaves(
    cpuid: impl Fn(u32) -> CpuidResult,
    cpuid_count: impl Fn(u32, u32) -> CpuidResult,
) -> [CpuidResult; 3] {
    let max_leaf = cpuid(0).eax;

    let leaf1 = if max_leaf >= 1 { cpuid(1) } else { ZERO_CPUID };
    let leaf7_0 = if max_leaf >= 7 {
        cpuid_count(7, 0)
    } else {
        ZERO_CPUID
    };
    // `max_leaf >= 7` is redundant here (leaf7_0 is zeroed otherwise, so its
    // eax cannot reach 1) but is kept to state the precondition explicitly.
    let leaf7_1 = if max_leaf >= 7 && leaf7_0.eax >= 1 {
        cpuid_count(7, 1)
    } else {
        ZERO_CPUID
    };

    [leaf1, leaf7_0, leaf7_1]
}

/// Use CPUID to detect the presence of all supplied target features.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::{__cpuid, __cpuid_count};
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::{__cpuid, __cpuid_count};

        let cr = $crate::x86::__collect_leaves(
            |leaf| unsafe { __cpuid(leaf) },
            |leaf, sub_leaf| unsafe { __cpuid_count(leaf, sub_leaf) },
        );

        $($crate::check!(cr, $tf) & )+ true
    }};
}

/// Check the XSAVE state a feature's encodings require.
///
/// Register bits are listed here:
/// <https://wiki.osdev.org/CPU_Registers_x86#Extended_Control_Registers>
///
/// There is deliberately no catch-all arm: an XSAVE tag outside the four
/// below is a compile error rather than a silently skipped check.
#[macro_export]
#[doc(hidden)]
macro_rules! __xsave_state {
    ($cr:expr, "") => {
        true
    };
    // Bit 1
    ($cr:expr, "xmm") => {
        $crate::__xgetbv!($cr, 0b10)
    };
    // Bits 1 and 2
    ($cr:expr, "ymm") => {
        $crate::__xgetbv!($cr, 0b110)
    };
    // Bits 1, 2, 5, 6, and 7
    ($cr:expr, "zmm") => {
        $crate::__xgetbv!($cr, 0b1110_0110)
    };
}

/// Check that OS supports required SIMD registers
#[macro_export]
#[doc(hidden)]
macro_rules! __xgetbv {
    ($cr:expr, $mask:expr) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86 as arch;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64 as arch;

        // Check bits 26 and 27
        let xmask = 0b11 << 26;
        let xsave = $cr[0].ecx & xmask == xmask;
        if xsave {
            // SAFETY: `xsave` above confirms CPUID.1:ECX bits 26 and 27
            // (XSAVE and OSXSAVE) are both set, so XGETBV is supported by
            // the CPU and enabled by the OS. ECX = 0 is always a valid
            // XCR index.
            let xcr0 = unsafe { arch::_xgetbv(arch::_XCR_XFEATURE_ENABLED_MASK) };
            (xcr0 & $mask) == $mask
        } else {
            false
        }
    }};
}

/// Accept only the four known XSAVE tags. Used to validate the table below at
/// its definition, so a mistyped tag is an error here rather than in whichever
/// downstream crate first happens to check that feature.
macro_rules! __assert_xsave_tag {
    ("") => {
        ()
    };
    ("xmm") => {
        ()
    };
    ("ymm") => {
        ()
    };
    ("zmm") => {
        ()
    };
}

macro_rules! __expand_check_macro {
    ($(($name:tt, $reg_cap:tt $(, $i:expr, $reg:ident, $offset:expr)*)),* $(,)?) => {
        $(
            const _: () = __assert_xsave_tag!($reg_cap);
        )*

        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                ($cr:expr, $name) => {{
                    // Register bits are listed here:
                    // https://wiki.osdev.org/CPU_Registers_x86#Extended_Control_Registers
                    $crate::__xsave_state!($cr, $reg_cap)
                    $(
                        & ($cr[$i].$reg & (1 << $offset) != 0)
                    )*
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
    ("sse4.1", "", 0, ecx, 19),
    ("sse4.2", "", 0, ecx, 20),
    ("popcnt", "", 0, ecx, 23),
    ("aes", "", 0, ecx, 25),
    ("avx", "ymm", 0, ecx, 28),
    ("rdrand", "", 0, ecx, 30),

    ("mmx", "", 0, edx, 23),
    ("sse", "", 0, edx, 25),
    ("sse2", "", 0, edx, 26),

    ("sgx", "", 1, ebx, 2),
    ("bmi1", "", 1, ebx, 3),
    ("bmi2", "", 1, ebx, 8),
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
    ("avx512vbmi", "zmm", 1, ecx, 1),
    ("avx512vbmi2", "zmm", 1, ecx, 6),
    // The 512-bit/EVEX-512 forms of GFNI/VAES/VPCLMULQDQ additionally
    // require `avx512f`, which consumers dispatching that form must check
    // separately; the bits below only certify the legacy-SSE (gfni) or
    // VEX-128/256 (vaes, vpclmulqdq) forms, which need at most `ymm` state.
    ("gfni", "", 1, ecx, 8),
    ("vaes", "ymm", 1, ecx, 9, 0, ecx, 28),
    ("vpclmulqdq", "ymm", 1, ecx, 10, 0, ecx, 28),
    ("avx512bitalg", "zmm", 1, ecx, 12),
    ("avx512vpopcntdq", "zmm", 1, ecx, 14),

    ("sha512", "ymm", 2, eax, 0),
    ("sm3", "ymm", 2, eax, 1),
    ("sm4", "ymm", 2, eax, 2),
}

#[cfg(test)]
mod tests {
    use super::CpuidResult;
    use super::{__collect_leaves, ZERO_CPUID};

    fn leaf(eax: u32, ebx: u32, ecx: u32) -> CpuidResult {
        CpuidResult {
            eax,
            ebx,
            ecx,
            edx: 0,
        }
    }

    fn r(eax: u32, ecx: u32) -> CpuidResult {
        CpuidResult {
            eax,
            ebx: 0,
            ecx,
            edx: 0,
        }
    }

    fn is_zero(c: &CpuidResult) -> bool {
        c.eax == 0 && c.ebx == 0 && c.ecx == 0 && c.edx == 0
    }

    /// A CPU reporting no basic leaves must not have leaf 1 read.
    #[test]
    fn leaf1_zeroed_when_max_leaf_is_zero() {
        let cr = __collect_leaves(|_| r(0, 0xdead_beef), |_, _| r(0, 0xdead_beef));
        assert!(is_zero(&cr[0]));
    }

    /// Below leaf 7 the CPU aliases to its highest leaf; those bits must
    /// not reach `check!` as leaf 7 feature bits.
    #[test]
    fn leaf7_zeroed_when_max_leaf_below_7() {
        let cr = __collect_leaves(
            |leaf| {
                if leaf == 0 {
                    r(1, 0)
                } else {
                    r(0, 0xffff_ffff)
                }
            },
            |_, _| r(0, 0xffff_ffff),
        );
        assert!(is_zero(&cr[1]), "leaf 7 subleaf 0 must be zeroed");
        assert!(is_zero(&cr[2]), "leaf 7 subleaf 1 must be zeroed");
    }

    /// Subleaf 1 is only valid when subleaf 0 reports it.
    #[test]
    fn subleaf1_zeroed_when_subleaf0_reports_none() {
        let cr = __collect_leaves(
            |_| r(7, 0),
            |leaf, sub| {
                assert!(!(leaf == 7 && sub == 1), "subleaf 1 must not be queried");
                r(0, 0x1234)
            },
        );
        assert_eq!(cr[1].ecx, 0x1234);
        assert!(is_zero(&cr[2]));
    }

    /// Subleaf 1 is read through when subleaf 0 reports it supported.
    #[test]
    fn subleaf1_read_when_supported() {
        let cr = __collect_leaves(
            |_| r(7, 0),
            |leaf, sub| match (leaf, sub) {
                (7, 0) => r(1, 0xaaaa),
                (7, 1) => r(0, 0xbbbb),
                _ => ZERO_CPUID,
            },
        );
        assert_eq!(cr[1].ecx, 0xaaaa);
        assert_eq!(cr[2].ecx, 0xbbbb);
    }

    /// Leaf 1 is still read at the boundary, and is read from leaf 1.
    #[test]
    fn leaf1_read_when_max_leaf_is_exactly_1() {
        let cr = __collect_leaves(
            |l| match l {
                0 => r(1, 0),
                1 => r(0, 0xc0de),
                _ => r(0, 0xbad0),
            },
            |_, _| r(0xffff_ffff, 0xffff_ffff),
        );
        assert_eq!(cr[0].ecx, 0xc0de, "leaf 1 must be read when max_leaf == 1");
        assert!(is_zero(&cr[1]));
        assert!(is_zero(&cr[2]));
    }

    /// The leaf 7 guard boundary is 7, not 6.
    #[test]
    fn leaf7_zeroed_when_max_leaf_is_exactly_6() {
        let cr = __collect_leaves(
            |l| if l == 0 { r(6, 0) } else { r(0, 0xffff_ffff) },
            |_, _| r(0xffff_ffff, 0xffff_ffff),
        );
        assert!(is_zero(&cr[1]), "leaf 7 subleaf 0 must be zeroed");
        assert!(is_zero(&cr[2]), "leaf 7 subleaf 1 must be zeroed");
    }

    /// With XSAVE/OSXSAVE clear, only the tags requiring no extended state
    /// can report present. `gfni` is now one of them; `vaes`, `vpclmulqdq`
    /// and `avx512f` are not.
    #[test]
    fn xsave_state_requirements_per_tag() {
        // Leaf 1 ECX bits 26, 27 (XSAVE, OSXSAVE) deliberately clear, so
        // `__xgetbv!` short-circuits to false without executing XGETBV.
        // Every relevant CPUID feature bit is set, so the only thing that
        // can distinguish these is the XSAVE tag.
        let cr = [
            leaf(0, 0, 0),
            leaf(0, 1 << 16, (1 << 8) | (1 << 9) | (1 << 10)),
            ZERO_CPUID,
        ];
        assert!(check!(cr, "gfni"), "gfni must not require XSAVE state");
        assert!(!check!(cr, "vaes"), "vaes must require ymm state");
        assert!(
            !check!(cr, "vpclmulqdq"),
            "vpclmulqdq must require ymm state"
        );
        assert!(
            !check!(cr, "avx512f"),
            "avx512f must still require zmm state"
        );
    }
}
