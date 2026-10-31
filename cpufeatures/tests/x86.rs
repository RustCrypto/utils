//! `x86`/`x86_64` tests

#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

cpufeatures::new!(cpuid, "aes", "sha");

#[test]
fn init() {
    let token: cpuid::InitToken = cpuid::init();
    assert_eq!(token.get(), cpuid::get());
}

#[test]
fn init_get() {
    let (token, val) = cpuid::init_get();
    assert_eq!(val, token.get());
    #[cfg(miri)]
    const {
        assert!(!cpufeatures::__detect_target_features!("sse2"));
    };
}

mod detection {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::CpuidResult;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::CpuidResult;

    const fn registers(bits: u32) -> [CpuidResult; 5] {
        [CpuidResult {
            eax: bits,
            ebx: bits,
            ecx: bits,
            edx: bits,
        }; 5]
    }

    #[test]
    fn cpuid_bounds() {
        for (max, subleaf, expected) in [(0, 0, 1), (1, 0, 2), (7, 0, 3), (7, 1, 4)] {
            let calls = core::cell::RefCell::new(Vec::new());
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, sub: u32| {
                calls.borrow_mut().push((leaf, sub));
                let eax = match (leaf, sub) { (0, 0) => max, (7, 0) => subleaf, _ => u32::MAX };
                CpuidResult { eax, ebx: u32::MAX, ecx: u32::MAX, edx: u32::MAX }
            }, false, false);
            assert_eq!(
                calls.into_inner(),
                [(0, 0), (1, 0), (7, 0), (7, 1)][..expected]
            );
            for (i, available) in [max >= 1, max >= 7, max >= 7 && subleaf >= 1]
                .into_iter()
                .enumerate()
            {
                assert_eq!(cr[i].ebx, if available { u32::MAX } else { 0 });
            }
        }
    }

    #[test]
    fn extended_cpuid_bounds() {
        for max in [0, 0x8000_0000, 0x8000_0001] {
            let calls = core::cell::Cell::new(0);
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, _: u32| {
                calls.set(calls.get() + 1);
                let mut result = registers(0)[0];
                match leaf {
                    0 => (),
                    0x8000_0000 => result.eax = max,
                    0x8000_0001 => { assert!(max >= leaf); result.ecx = u32::MAX; },
                    _ => panic!("unexpected CPUID leaf"),
                }
                result
            }, true, false);
            let available = max >= 0x8000_0001;
            assert_eq!(cr[3].ecx, if available { u32::MAX } else { 0 });
            assert_eq!(calls.get(), if available { 3 } else { 2 });
        }
    }

    #[test]
    fn xsave_cpuid_bounds() {
        for max in [0, 1, 7, 0xc, 0xd] {
            for requested in [false, true] {
                for flags in [0, 1 << 26, 1 << 27, 3 << 26] {
                    let calls = core::cell::Cell::new(0);
                    let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, sub: u32| {
                        let mut result = registers(0)[0];
                        match leaf {
                            0 => result.eax = max,
                            1 => result.ecx = flags,
                            7 => (),
                            0xd => { assert_eq!(sub, 1); calls.set(calls.get() + 1); result.eax = 0xb; },
                            _ => panic!("unexpected CPUID leaf"),
                        }
                        result
                    }, false, requested);
                    let available = requested && max >= 0xd && flags == 3 << 26;
                    assert_eq!(cr[4].eax, if available { 0xb } else { 0 });
                    assert_eq!(calls.get(), u32::from(available));
                }
            }
        }
    }

    #[test]
    fn xgetbv_guard() {
        for (flags, enabled) in [
            (0, false),
            (1 << 26, false),
            (1 << 27, false),
            (3 << 26, true),
        ] {
            let mut cr = registers(0);
            cr[0].ecx = flags;
            for (mask, state, supported) in [
                (0, 0, true),
                (6, 0, false),
                (6, 2, false),
                (6, 4, false),
                (6, 6, true),
                (0xe6, 6, false),
                (0xe6, 0xe2, false),
                (0xe6, 0xe4, false),
                (0xe6, 0xc6, false),
                (0xe6, 0xa6, false),
                (0xe6, 0x66, false),
                (0xe6, 0xe6, true),
                (0xe6, u64::MAX, true),
            ] {
                let reads = core::cell::Cell::new(0);
                assert_eq!(
                    cpufeatures::__xgetbv!(@read cr, mask, {
                        reads.set(reads.get() + 1);
                        state
                    }),
                    enabled && supported
                );
                assert_eq!(reads.get(), u32::from(enabled));
            }
        }
    }

    #[test]
    fn required_bits_and_state() {
        macro_rules! check {
            ($feature:tt, $state:expr; $($i:expr, $reg:ident, $bit:expr);+ $(;)?) => {{
                let cr = registers(u32::MAX);
                let mut sparse = registers(0);
                if $state != 0 {
                    sparse[0].ecx = (1 << 0) | (1 << 9) | (1 << 19) | (1 << 20) | (1 << 26) | (1 << 27) | (1 << 28);
                    sparse[0].edx = (1 << 25) | (1 << 26);
                }
                if $state == 0xe6 {
                    sparse[0].ecx |= (1 << 12) | (1 << 29);
                    sparse[1].ebx = (1 << 5) | (1 << 16);
                }
                $(sparse[$i].$reg |= 1 << $bit;)+
                assert!(cpufeatures::check!(sparse, $feature, |_| true), $feature);
                $(let mut missing = cr;
                missing[$i].$reg &= !(1 << $bit);
                assert!(!cpufeatures::check!(missing, $feature, |_| panic!("XGETBV queried")), $feature);)+
                for state in [0, 2, 4, 6, 0x66, 0xa6, 0xc6, 0xe2, 0xe4, 0xe6] {
                    assert_eq!(cpufeatures::check!(cr, $feature, |mask| state & mask == mask), state & $state == $state, $feature);
                }
                if $state == 0 {
                    assert!(cpufeatures::check!(sparse, $feature, |_| panic!("XGETBV queried")), $feature);
                }
                if $state != 0 {
                    for bit in [0, 9, 12, 19, 20, 26, 27, 28, 29] {
                        if $state != 0xe6 && matches!(bit, 12 | 29) { continue; }
                        let mut missing = cr;
                        missing[0].ecx &= !(1 << bit);
                        assert!(!cpufeatures::check!(missing, $feature, |_| panic!("XGETBV queried")), $feature);
                    }
                    for bit in [25, 26] {
                        let mut missing = cr;
                        missing[0].edx &= !(1 << bit);
                        assert!(!cpufeatures::check!(missing, $feature, |_| panic!("XGETBV queried")), $feature);
                    }
                    if $state == 0xe6 {
                        for bit in [5, 16] {
                            let mut missing = cr;
                            missing[1].ebx &= !(1 << bit);
                            assert!(!cpufeatures::check!(missing, $feature, |_| panic!("XGETBV queried")), $feature);
                        }
                    }
                }
            }};
        }
        check!("ermsb", 0; 1, ebx, 9);
        check!("prfchw", 0; 3, ecx, 8);
        check!("fxsr", 0; 0, edx, 24);
        check!("cmpxchg16b", 0; 0, ecx, 13);
        check!("movbe", 0; 0, ecx, 22);
        check!("lahfsahf", 0; 3, ecx, 0);
        check!("lzcnt", 0; 3, ecx, 5);
        check!("xsave", 0; 0, ecx, 26; 0, ecx, 27);
        check!("xsaveopt", 0; 0, ecx, 26; 0, ecx, 27; 4, eax, 0);
        check!("xsavec", 0; 0, ecx, 26; 0, ecx, 27; 4, eax, 1);
        check!("xsaves", 0; 0, ecx, 26; 0, ecx, 27; 4, eax, 3);
        check!("f16c", 6; 0, ecx, 29);
        check!("avxvnni", 6; 2, eax, 4; 1, ebx, 5);
        check!("avxifma", 6; 2, eax, 23; 1, ebx, 5);
        check!("avxneconvert", 6; 2, edx, 5; 1, ebx, 5);
        check!("avxvnniint8", 6; 2, edx, 4; 1, ebx, 5);
        check!("avxvnniint16", 6; 2, edx, 10; 1, ebx, 5);
        check!("avx512vnni", 0xe6; 1, ecx, 11);
        check!("avx512vp2intersect", 0xe6; 1, edx, 8);
        check!("avx512bf16", 0xe6; 2, eax, 5; 1, ebx, 30);
        check!("avx512fp16", 0xe6; 1, edx, 23; 1, ebx, 30);
        check!("gfni", 0; 1, ecx, 8; 0, edx, 25; 0, edx, 26);
        check!("vaes", 6; 1, ecx, 9; 0, ecx, 25; 1, ebx, 5);
        check!("vpclmulqdq", 6; 1, ecx, 10; 0, ecx, 1);
        check!("sha512", 6; 2, eax, 0; 1, ebx, 5);
        check!("sm3", 6; 2, eax, 1);
        check!("sm4", 6; 2, eax, 2; 1, ebx, 5);
        check!("avx512f", 0xe6; 1, ebx, 16);
        check!("avx512dq", 0xe6; 1, ebx, 17);
        check!("avx512ifma", 0xe6; 1, ebx, 21);
        check!("avx512pf", 0xe6; 1, ebx, 26);
        check!("avx512er", 0xe6; 1, ebx, 27);
        check!("avx512cd", 0xe6; 1, ebx, 28);
        check!("avx512bw", 0xe6; 1, ebx, 30);
        check!("avx512vl", 0xe6; 1, ebx, 31);
        check!("avx512vbmi", 0xe6; 1, ecx, 1; 1, ebx, 30);
        check!("avx512vbmi2", 0xe6; 1, ecx, 6; 1, ebx, 30);
        check!("avx512bitalg", 0xe6; 1, ecx, 12; 1, ebx, 30);
        check!("avx512vpopcntdq", 0xe6; 1, ecx, 14);
    }

    #[cfg(all(
        not(miri),
        not(any(target_env = "sgx", target_os = "none", target_os = "uefi"))
    ))]
    #[test]
    fn cached_detection() {
        macro_rules! check {
            ($($feature:tt),+) => {{
                cpufeatures::new!(combined, $($feature),+);
                assert_eq!(combined::get(), true $(& (cfg!(target_feature = $feature) || cpufeatures::__detect_target_features!($feature)))+);
            } $({
                cpufeatures::new!(detected, $feature);
                assert_eq!(detected::get(), cfg!(target_feature = $feature) || cpufeatures::__detect_target_features!($feature), $feature);
            })+};
        }
        check!("lahfsahf", "ermsb", "prfchw");
        check!("xsave", "xsavec", "xsaveopt", "xsaves");
        check!(
            "avxvnni",
            "avxifma",
            "avxneconvert",
            "avxvnniint8",
            "avxvnniint16"
        );
        macro_rules! standard {
            ($($feature:tt),+) => {$({
                cpufeatures::new!(detected, $feature);
                assert_eq!(detected::get(), std::arch::is_x86_feature_detected!($feature), $feature);
            })+};
        }
        // Std requires AVX register state for XSAVE. The instructions do not.
        if std::arch::is_x86_feature_detected!("avx") {
            standard!("xsave", "xsavec", "xsaveopt", "xsaves");
        }
        standard!("fxsr", "cmpxchg16b", "f16c", "movbe", "lzcnt");
        standard!(
            "gfni",
            "vaes",
            "vpclmulqdq",
            "avx512vnni",
            "avx512vp2intersect",
            "avx512bf16",
            "avx512fp16"
        );
    }
}
