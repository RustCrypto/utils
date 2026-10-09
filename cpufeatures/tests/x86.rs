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

#[cfg(not(miri))]
mod capability_masks {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::CpuidResult;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::CpuidResult;

    const FMA: u32 = 1 << 12;
    const F16C: u32 = 1 << 29;
    const ALL_BITS: [CpuidResult; 3] = [CpuidResult {
        eax: u32::MAX,
        ebx: u32::MAX,
        ecx: u32::MAX,
        edx: u32::MAX,
    }; 3];

    macro_rules! avx512_prerequisites {
        ($(($name:ident, $feature:tt)),+ $(,)?) => {
            $(
                #[test]
                fn $name() {
                    for flags in [0, FMA, F16C, FMA | F16C] {
                        let mut registers = ALL_BITS;
                        registers[0].ecx = (registers[0].ecx & !(FMA | F16C)) | flags;
                        assert_eq!(
                            cpufeatures::check!(@cpuid registers, $feature),
                            flags == FMA | F16C,
                        );
                    }
                }
            )+
        };
    }

    avx512_prerequisites! {
        (avx512f, "avx512f"),
        (avx512dq, "avx512dq"),
        (avx512ifma, "avx512ifma"),
        (avx512pf, "avx512pf"),
        (avx512er, "avx512er"),
        (avx512cd, "avx512cd"),
        (avx512bw, "avx512bw"),
        (avx512vl, "avx512vl"),
        (avx512vbmi, "avx512vbmi"),
        (avx512vbmi2, "avx512vbmi2"),
        (avx512bitalg, "avx512bitalg"),
        (avx512vpopcntdq, "avx512vpopcntdq"),
    }

    #[test]
    fn other_zmm_features_do_not_require_fma_or_f16c() {
        let mut registers = ALL_BITS;
        registers[0].ecx &= !(FMA | F16C);
        assert!(cpufeatures::check!(@cpuid registers, "gfni"));
        assert!(cpufeatures::check!(@cpuid registers, "vaes"));
        assert!(cpufeatures::check!(@cpuid registers, "vpclmulqdq"));
    }

    #[test]
    fn check_requires_cpu_bits() {
        let mut registers = ALL_BITS;
        assert!(cpufeatures::check!(registers, "sha"));
        registers[1].ebx &= !(1 << 29);
        assert!(!cpufeatures::check!(registers, "sha"));
    }

    #[test]
    fn avx512_requires_xsave_and_osxsave() {
        for bit in [26, 27] {
            let mut registers = ALL_BITS;
            registers[0].ecx &= !(1 << bit);
            assert!(!cpufeatures::check!(registers, "avx512f"));
        }
    }
}
