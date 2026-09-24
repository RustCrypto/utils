//! `x86`/`x86_64` tests

#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

cpufeatures::new!(cpuid, "aes", "sha");
cpufeatures::new!(cpuid_avx10, "avx10.1", "avx10.2");

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
fn avx10_probe_does_not_fault() {
    let (token, val) = cpuid_avx10::init_get();
    assert_eq!(val, token.get());
    let _ = cpuid_avx10::get();
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx10_logic {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::CpuidResult;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::CpuidResult;

    fn leaf1() -> CpuidResult {
        CpuidResult {
            eax: 0,
            ebx: 0,
            ecx: 0b11 << 26,
            edx: 0,
        }
    }

    fn blank() -> CpuidResult {
        CpuidResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        }
    }

    #[test]
    fn version_zero_is_disabled() {
        let cr = [
            leaf1(),
            blank(),
            CpuidResult {
                eax: 0,
                ebx: 0,
                ecx: 0,
                edx: 1 << 19,
            },
            CpuidResult {
                eax: 0x24,
                ebx: 0,
                ecx: 0,
                edx: 0,
            },
            CpuidResult {
                eax: 0,
                ebx: (1 << 17) | (1 << 18),
                ecx: 0,
                edx: 0,
            },
        ];
        assert!(!cpufeatures::check!(cr, "avx10.1"));
        assert!(!cpufeatures::check!(cr, "avx10.2"));
    }

    #[test]
    fn max_leaf_guard_holds() {
        let cr = [
            leaf1(),
            blank(),
            CpuidResult {
                eax: 0,
                ebx: 0,
                ecx: 0,
                edx: 1 << 19,
            },
            CpuidResult {
                eax: 0x16,
                ebx: 0,
                ecx: 0,
                edx: 0,
            },
            CpuidResult {
                eax: 0,
                ebx: 1 | (1 << 17) | (1 << 18),
                ecx: 0,
                edx: 0,
            },
        ];
        assert!(!cpufeatures::check!(cr, "avx10.1"));
        assert!(!cpufeatures::check!(cr, "avx10.2"));
    }

    #[test]
    fn presence_bit_required() {
        let cr = [
            leaf1(),
            blank(),
            blank(),
            CpuidResult {
                eax: 0x24,
                ebx: 0,
                ecx: 0,
                edx: 0,
            },
            CpuidResult {
                eax: 0,
                ebx: 1 | (1 << 17) | (1 << 18),
                ecx: 0,
                edx: 0,
            },
        ];
        assert!(!cpufeatures::check!(cr, "avx10.1"));
        assert!(!cpufeatures::check!(cr, "avx10.2"));
    }

    #[test]
    fn narrow_vector_without_512_bit_reports_false() {
        let cr = [
            leaf1(),
            blank(),
            CpuidResult {
                eax: 0,
                ebx: 0,
                ecx: 0,
                edx: 1 << 19,
            },
            CpuidResult {
                eax: 0x24,
                ebx: 0,
                ecx: 0,
                edx: 0,
            },
            CpuidResult {
                eax: 0,
                ebx: 1 | (1 << 17),
                ecx: 0,
                edx: 0,
            },
        ];
        assert!(!cpufeatures::check!(cr, "avx10.1"));
        assert!(!cpufeatures::check!(cr, "avx10.2"));
    }

    #[test]
    fn vnni_trio_required_for_avx10_2() {
        let cr = [
            leaf1(),
            blank(),
            CpuidResult {
                eax: 0,
                ebx: 0,
                ecx: 0,
                edx: 1 << 19,
            },
            CpuidResult {
                eax: 0x24,
                ebx: 0,
                ecx: 0,
                edx: 0,
            },
            CpuidResult {
                eax: 0,
                ebx: 2 | (1 << 18),
                ecx: 0,
                edx: 0,
            },
        ];
        assert!(!cpufeatures::check!(cr, "avx10.2"));
    }
}
