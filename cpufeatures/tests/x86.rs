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
}

#[test]
fn sse2_and_popcnt() {
    cpufeatures::new!(sse2, "sse2");
    cpufeatures::new!(popcnt, "popcnt");
    cpufeatures::new!(both, "sse2", "popcnt");

    let sse2_enabled = cfg!(target_feature = "sse2");
    let popcnt_enabled = cfg!(target_feature = "popcnt");
    if cfg!(any(
        miri,
        target_env = "sgx",
        target_os = "none",
        target_os = "uefi"
    )) {
        assert_eq!(sse2::get(), sse2_enabled);
        assert_eq!(popcnt::get(), popcnt_enabled);
    } else {
        assert_eq!(sse2::get(), std::arch::is_x86_feature_detected!("sse2"));
        assert_eq!(popcnt::get(), std::arch::is_x86_feature_detected!("popcnt"));
    }
    let detected = both::get();
    let singles = sse2::get() && popcnt::get();
    // A mixed query can fall back to checking both features at runtime.
    if sse2_enabled == popcnt_enabled || cfg!(miri) {
        assert_eq!(detected, singles);
    } else {
        assert!(!detected || singles);
    }
}

mod detection {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::CpuidResult;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::CpuidResult;

    const ZERO: CpuidResult = CpuidResult {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
    };
    const YMM: u64 = 0x06;
    const ZMM: u64 = 0xe6;

    fn vector_registers(state: u64) -> [CpuidResult; 7] {
        let mut cr = [ZERO; 7];
        if state != 0 {
            cr[0].ecx =
                (1 << 0) | (1 << 9) | (1 << 19) | (1 << 20) | (1 << 26) | (1 << 27) | (1 << 28);
            cr[0].edx = (1 << 25) | (1 << 26);
        }
        if state == ZMM {
            cr[0].ecx |= (1 << 12) | (1 << 29);
            cr[1].ebx = (1 << 5) | (1 << 16);
        }
        cr
    }

    fn cpu_info(max_leaf: u32, vendor: [u8; 12]) -> CpuidResult {
        CpuidResult {
            eax: max_leaf,
            ebx: u32::from_le_bytes(vendor[..4].try_into().expect("four vendor bytes")),
            edx: u32::from_le_bytes(vendor[4..8].try_into().expect("four vendor bytes")),
            ecx: u32::from_le_bytes(vendor[8..].try_into().expect("four vendor bytes")),
        }
    }

    #[test]
    fn cpuid_bounds() {
        for (max, subleaf, bmi, movrs) in [
            (0, 0, true, true),
            (1, 0, true, true),
            (7, 0, true, true),
            (7, 1, true, true),
            (7, 1, false, true),
            (7, 1, true, false),
            (7, 1, false, false),
        ] {
            let leaf7 = max >= 7 && (bmi || movrs);
            let leaf7_1 = leaf7 && subleaf >= 1 && movrs;
            let leaves = (if bmi {
                cpufeatures::check!(@leaves "bmi1")
            } else {
                0
            }) | (if movrs {
                cpufeatures::check!(@leaves "movrs")
            } else {
                0
            });
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, sub: u32| {
                match (leaf, sub) {
                    (0, 0) => CpuidResult { eax: max, ..ZERO },
                    (1, 0) if max >= 1 => CpuidResult { ecx: 1 << 22, ..ZERO },
                    (7, 0) if leaf7 => CpuidResult { eax: subleaf, ebx: 1 << 3, ..ZERO },
                    (7, 1) if leaf7_1 => CpuidResult { eax: 1 << 31, ..ZERO },
                    _ => panic!("unexpected CPUID leaf {leaf:#x}:{sub}"),
                }
            }, leaves);
            assert_eq!(cpufeatures::check!(cr, "movbe", |_| false), max >= 1);
            if bmi {
                assert_eq!(cpufeatures::check!(cr, "bmi1", |_| false), leaf7);
            }
            assert_eq!(cpufeatures::check!(cr, "movrs", |_| false), leaf7_1);
        }
    }

    #[test]
    fn extended_cpuid_bounds() {
        const AMD_FEATURES: u32 = (1 << 6) | (1 << 11) | (1 << 16) | (1 << 21);
        const LZCNT: u32 = 1 << 5;
        for (vendor, amd_bits) in [
            (*b"AuthenticAMD", AMD_FEATURES),
            (*b"HygonGenuine", AMD_FEATURES),
            (*b"GenuineIntel", 0),
        ] {
            for (requested, max) in [
                (false, 0x8000_0001),
                (true, 0x8000_0000),
                (true, 0x8000_0001),
            ] {
                let available = requested && max >= 0x8000_0001;
                let leaves = if requested {
                    cpufeatures::check!(@leaves "lzcnt")
                } else {
                    0
                };
                let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, sub: u32| {
                    match (leaf, sub) {
                        (0, 0) => cpu_info(0, vendor),
                        (0x8000_0000, 0) if requested => CpuidResult { eax: max, ..ZERO },
                        (0x8000_0001, 0) if available => CpuidResult { ecx: AMD_FEATURES | LZCNT, ..ZERO },
                        _ => panic!("unexpected CPUID leaf {leaf:#x}:{sub}"),
                    }
                }, leaves);
                assert_eq!(cr[3].ecx, if available { amd_bits | LZCNT } else { 0 });
            }
        }
    }

    #[test]
    fn xsave_cpuid_bounds() {
        for (max, requested, flags, available) in [
            (0xc, true, 3 << 26, false),
            (0xd, false, 3 << 26, false),
            (0xd, true, 1 << 26, false),
            (0xd, true, 1 << 27, false),
            (0xd, true, 3 << 26, true),
        ] {
            let leaves = if requested {
                cpufeatures::check!(@leaves "xsaveopt")
            } else {
                0
            };
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, sub: u32| {
                match (leaf, sub) {
                    (0, 0) => CpuidResult { eax: max, ..ZERO },
                    (1, 0) => CpuidResult { ecx: flags, ..ZERO },
                    (0xd, 1) if available => CpuidResult { eax: 1, ..ZERO },
                    _ => panic!("unexpected CPUID leaf {leaf:#x}:{sub}"),
                }
            }, leaves);
            assert_eq!(cpufeatures::check!(cr, "xsaveopt", |_| false), available);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn cmpxchg16b_rejects_intel_ld0() {
        for (vendor, signature, supported) in [
            (*b"GenuineIntel", 0x0f34, false),
            (*b"GenuineIntel", 0x0f41, true),
            (*b"AuthenticAMD", 0x0f34, true),
        ] {
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf, subleaf| {
                match (leaf, subleaf) {
                    (0, 0) => cpu_info(1, vendor),
                    (1, 0) => CpuidResult { eax: signature, ebx: 0, ecx: 1 << 13, edx: 0 },
                    _ => panic!("unexpected CPUID leaf"),
                }
            }, 0);
            assert_eq!(cpufeatures::check!(cr, "cmpxchg16b", |_| false), supported);
        }
    }

    #[test]
    fn bmi_rejects_affected_skylake() {
        for (vendor, signature, avx, supported) in [
            (*b"GenuineIntel", 0x406e3, false, false),
            (*b"GenuineIntel", 0x506e3, false, false),
            (*b"GenuineIntel", 0x506e3, true, true),
            (*b"GenuineIntel", 0x90672, false, true),
            (*b"AuthenticAMD", 0x506e3, false, true),
        ] {
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf, subleaf| {
                match (leaf, subleaf) {
                    (0, 0) => cpu_info(7, vendor),
                    (1, 0) => CpuidResult { eax: signature, ebx: 0, ecx: u32::from(avx) << 28, edx: 0 },
                    (7, 0) => CpuidResult { eax: 0, ebx: (1 << 3) | (1 << 8), ecx: 0, edx: 0 },
                    _ => panic!("unexpected CPUID leaf"),
                }
            }, cpufeatures::check!(@leaves "bmi1") | cpufeatures::check!(@leaves "bmi2"));
            assert_eq!(cpufeatures::check!(cr, "bmi1", |_| false), supported);
            assert_eq!(cpufeatures::check!(cr, "bmi2", |_| false), supported);
        }
    }

    #[test]
    fn xgetbv_guard() {
        let mut cr = [ZERO; 7];
        let read = || -> u64 { panic!("XGETBV queried") };
        for flags in [0, 1 << 26, 1 << 27] {
            cr[0].ecx = flags;
            assert!(!cpufeatures::__xgetbv!(@read cr, YMM, read()));
        }
        cr[0].ecx = 3 << 26;
        assert!(cpufeatures::__xgetbv!(@read cr, YMM, ZMM));
        assert!(cpufeatures::__xgetbv!(@read cr, ZMM, ZMM));
        for bit in [1, 2, 5, 6, 7] {
            assert!(!cpufeatures::__xgetbv!(@read cr, ZMM, ZMM & !(1 << bit)));
        }
    }

    #[test]
    fn bundle_reads_xcr0_once() {
        let mut cr = vector_registers(ZMM);
        cr[0].ecx |= 1 << 25; // AES
        let calls = core::cell::Cell::new(0);
        let read = |mask| {
            assert_eq!(mask, ZMM);
            calls.set(calls.get() + 1);
            true
        };

        assert!(cpufeatures::__detect_target_features!(@check cr, read; "sse2"));
        assert!(cpufeatures::__detect_target_features!(@check cr, read; "avx512f", "avx2"));
        assert_eq!(calls.get(), 1);
        assert!(!cpufeatures::__detect_target_features!(@check cr, |_| false; "avx2", "avx512f"));

        cr[0].ecx &= !(1 << 25);
        assert!(!cpufeatures::__detect_target_features!(@check cr, read; "avx2", "avx512f", "aes"));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn rtm_respects_forced_abort() {
        let mut cr = [ZERO; 7];
        assert!(!cpufeatures::check!(cr, "rtm", |_| false));
        cr[1].ebx = 1 << 11;
        assert!(cpufeatures::check!(cr, "rtm", |_| false));
        cr[1].edx = 1 << 11;
        assert!(!cpufeatures::check!(cr, "rtm", |_| false));
    }

    #[test]
    fn required_bits_and_state() {
        macro_rules! check {
            ($feature:tt, $state:expr; $($i:expr, $reg:ident, $bit:expr);+ $(;)?) => {{
                let mut cr = vector_registers($state);
                $(cr[$i].$reg |= 1 << $bit;)+
                assert!(
                    cpufeatures::check!(cr, $feature, |mask| {
                        assert_ne!($state, 0, "{} read XCR0", $feature);
                        assert_eq!(mask, $state, $feature);
                        true
                    }),
                    $feature
                );
                $(
                    let mut missing = cr;
                    missing[$i].$reg &= !(1 << $bit);
                    assert!(
                        !cpufeatures::check!(missing, $feature, |_| panic!("XGETBV queried")),
                        "{} without bit {} in {} of CPUID result {}",
                        $feature, $bit, stringify!($reg), $i
                    );
                )+
                if $state != 0 {
                    assert!(!cpufeatures::check!(cr, $feature, |_| false), $feature);
                }
            }};
        }
        check!("ermsb", 0; 1, ebx, 9);
        check!("prfchw", 0; 3, ecx, 8);
        check!("fxsr", 0; 0, edx, 24);
        #[cfg(target_arch = "x86_64")]
        check!("cmpxchg16b", 0; 0, ecx, 13);
        check!("x87", 0; 0, edx, 0);
        check!("clflushopt", 0; 1, ebx, 23);
        check!("movdiri", 0; 1, ecx, 27);
        check!("movdir64b", 0; 1, ecx, 28);
        check!("movrs", 0; 2, eax, 31);
        check!("sse4a", 0; 3, ecx, 6; 0, ecx, 0; 0, edx, 25; 0, edx, 26);
        check!("xop", 6; 3, ecx, 11; 3, ecx, 16; 3, ecx, 6);
        check!("fma4", 6; 3, ecx, 16; 3, ecx, 6);
        check!("tbm", 0; 3, ecx, 21);
        check!("kl", 0; 1, ecx, 23; 5, ebx, 0; 0, edx, 25; 0, edx, 26);
        check!("widekl", 0; 1, ecx, 23; 5, ebx, 0; 5, ebx, 2; 0, edx, 25; 0, edx, 26);
        check!("aes", 0; 0, ecx, 25; 0, edx, 25; 0, edx, 26);
        check!("pclmulqdq", 0; 0, ecx, 1; 0, edx, 25; 0, edx, 26);
        check!("sha", 0; 1, ebx, 29; 0, edx, 25; 0, edx, 26);
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
        // Check the shared vector prerequisites once for each register class.
        check!("avx", YMM; 0, ecx, 0; 0, ecx, 9; 0, ecx, 19; 0, ecx, 20;
            0, ecx, 26; 0, ecx, 27; 0, ecx, 28; 0, edx, 25; 0, edx, 26);
        check!("avx512f", ZMM; 0, ecx, 12; 0, ecx, 29; 1, ebx, 5; 1, ebx, 16);
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

    #[test]
    fn key_locker_and_avx10_cpuid_bounds() {
        for (max, kl_flag, avx10_flag) in [
            (0x18, true, true),
            (0x19, true, true),
            (0x23, true, true),
            (0x24, true, false),
            (0x24, false, true),
            (0x24, true, true),
        ] {
            let key_locker = max >= 0x19 && kl_flag;
            let avx10 = max >= 0x24 && avx10_flag;
            let cr = cpufeatures::__detect_target_features!(@cpuid |leaf: u32, sub: u32| {
                match (leaf, sub) {
                    (0, 0) => CpuidResult { eax: max, ..ZERO },
                    (1, 0) => ZERO,
                    (7, 0) => CpuidResult { eax: 1, ecx: u32::from(kl_flag) << 23, ..ZERO },
                    (7, 1) => CpuidResult { edx: u32::from(avx10_flag) << 19, ..ZERO },
                    (0x19, 0) if key_locker => CpuidResult { ebx: 5, ..ZERO },
                    (0x24, 0) if avx10 => CpuidResult { ebx: (1 << 18) | 2, ..ZERO },
                    _ => panic!("unexpected CPUID leaf {leaf:#x}:{sub}"),
                }
            }, cpufeatures::check!(@leaves "widekl") | cpufeatures::check!(@leaves "avx10.2"));
            assert_eq!(cr[5].ebx, if key_locker { 5 } else { 0 });
            assert_eq!(cr[6].ebx, if avx10 { (1 << 18) | 2 } else { 0 });
        }
    }

    #[test]
    fn avx10_requires_the_version_width_and_legacy_features() {
        let mut cr = vector_registers(ZMM);
        cr[1].ebx |= (1 << 17) | (1 << 21) | (1 << 28) | (1 << 30) | (1 << 31);
        cr[1].ecx = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 12) | (1 << 14);
        cr[1].edx = 1 << 23;
        cr[2].eax = (1 << 4) | (1 << 5);
        cr[2].edx = (1 << 4) | (1 << 10) | (1 << 19);
        for (ebx, avx10_1, avx10_2) in [
            (0, false, false),
            (1, false, false),
            (1 | (1 << 18), true, false),
            (2, false, false),
            (2 | (1 << 18), true, true),
        ] {
            cr[6].ebx = ebx;
            assert_eq!(
                cpufeatures::check!(cr, "avx10.1", |mask| mask == ZMM),
                avx10_1
            );
            assert_eq!(
                cpufeatures::check!(cr, "avx10.2", |mask| mask == ZMM),
                avx10_2
            );
        }
        cr[1].ecx &= !(1 << 12); // AVX512BITALG
        assert!(!cpufeatures::check!(cr, "avx10.1", |_| true));
        assert!(!cpufeatures::check!(cr, "avx10.2", |_| true));
    }

    #[test]
    fn apx_requires_64_bit_mode_and_os_state() {
        let mut cr = [ZERO; 7];
        cr[0].ecx = 3 << 26;
        cr[2].edx = 1 << 21;
        assert_eq!(
            cpufeatures::check!(cr, "apxf", |mask| mask == 1 << 19),
            cfg!(target_arch = "x86_64")
        );
        assert!(!cpufeatures::check!(cr, "apxf", |_| false));
        cr[0].ecx &= !(1 << 27);
        assert!(!cpufeatures::check!(cr, "apxf", |_| panic!(
            "XGETBV queried"
        )));
    }

    #[test]
    fn compiler_optional_features() {
        // Check each name alone so an earlier runtime-only name cannot hide it.
        macro_rules! check {
            ($($feature:tt),+ $(,)?) => {$({
                #[allow(unexpected_cfgs)]
                let enabled = cfg!(target_feature = $feature);
                #[forbid(unexpected_cfgs)]
                {
                    cpufeatures::new!(detected, $feature);
                    let actual = detected::get();
                    if cfg!(miri) || enabled {
                        assert_eq!(actual, enabled, $feature);
                    }
                }
            })+};
        }
        #[cfg(target_arch = "x86_64")]
        check!("apxf");
        check!(
            "avx10.1",
            "avx10.2",
            "avx512er",
            "avx512pf",
            "clflushopt",
            "fma4",
            "kl",
            "movdir64b",
            "movdiri",
            "movrs",
            "widekl",
        );
        #[forbid(unexpected_cfgs)]
        {
            cpufeatures::new!(mixed, "sse2", "avx512pf");
            let (token, detected) = mixed::init_get();
            assert_eq!(token.get(), detected);
        }
    }

    #[cfg(target_arch = "x86")]
    #[test]
    #[forbid(unexpected_cfgs)]
    fn rejects_64_bit_only_features() {
        cpufeatures::new!(cx16, "cmpxchg16b");
        cpufeatures::new!(apx, "apxf");
        assert!(!cx16::get());
        assert!(!apx::get());

        let mut cr = [ZERO; 7];
        cr[0].ecx = 1 << 13;
        assert!(!cpufeatures::check!(cr, "cmpxchg16b", |_| true));
    }
}
