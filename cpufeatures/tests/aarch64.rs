//! ARM64 tests

#![cfg(target_arch = "aarch64")]

cpufeatures::new!(armcaps, "aes", "sha2", "sha3", "sm4");

#[test]
fn init() {
    let token: armcaps::InitToken = armcaps::init();
    assert_eq!(token.get(), armcaps::get());
}

#[test]
fn init_get() {
    let (token, val) = armcaps::init_get();
    assert_eq!(val, token.get());
}

#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
#[test]
fn neon_and_crc() {
    cpufeatures::new!(neon, "neon");
    cpufeatures::new!(crc, "crc");
    cpufeatures::new!(both, "neon", "crc");
    let neon_enabled = cfg!(target_feature = "neon");
    let crc_enabled = cfg!(target_feature = "crc");
    if cfg!(miri) {
        assert_eq!(neon::get(), neon_enabled);
        assert_eq!(crc::get(), crc_enabled);
    } else {
        assert_eq!(neon::get(), std::arch::is_aarch64_feature_detected!("neon"));
        assert_eq!(crc::get(), std::arch::is_aarch64_feature_detected!("crc"));
    }
    let (token, detected) = both::init_get();
    assert_eq!(detected, token.get());
    // In a mixed build, the whole query can fall back to runtime detection.
    if neon_enabled == crc_enabled || cfg!(miri) {
        assert_eq!(detected, neon::get() && crc::get());
    } else {
        assert!(!detected || (neon::get() && crc::get()));
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn capability_masks() {
    macro_rules! check {
        ($feature:tt, $mask:expr $(, $mask2:expr)?) => {{
            let full: [u64; 2] = [$mask, 0 $(| $mask2)?];
            assert_eq!(cpufeatures::check!(@hwcap2 $feature), full[1] != 0, $feature);
            assert!(cpufeatures::check!(@words full, $feature), $feature);
            for (word, mut bits) in full.into_iter().enumerate() {
                while bits != 0 {
                    let bit = bits.trailing_zeros();
                    let mut missing = full;
                    missing[word] &= !(1 << bit);
                    assert!(
                        !cpufeatures::check!(@words missing, $feature),
                        "{} without bit {bit} in capability word {word}",
                        $feature
                    );
                    bits &= bits - 1;
                }
            }
        }};
    }
    check!("neon", 3);
    check!("dotprod", 3 | (1 << 20));
    check!("fcma", 3 | (1 << 14));
    check!("fp16", 3 | (1 << 9) | (1 << 10));
    check!("fhm", 3 | (1 << 9) | (1 << 10) | (1 << 23));
    check!("jsconv", 3 | (1 << 13));
    check!("sve-b16b16", 3, (1 << 14) | (1 << 45));
    const SVE: u64 = 3 | (1 << 9) | (1 << 10) | (1 << 22);
    check!("crc", 1 << 7);
    check!("cssc", 0, 1 << 34);
    check!("lse", 1 << 8);
    check!("lse2", 1 << 25);
    check!("mops", 0, 1 << 43);
    check!("bf16", 3, 1 << 14);
    check!("i8mm", 3, 1 << 13);
    check!("sve", SVE);
    check!("sve2", SVE, 1 << 1);
    check!(
        "sve2-aes",
        SVE | (1 << 3) | (1 << 4),
        (1 << 1) | (1 << 2) | (1 << 3)
    );
    check!("sve2-bitperm", SVE, (1 << 1) | (1 << 4));
    check!(
        "sve2-sha3",
        SVE | (1 << 5) | (1 << 6) | (1 << 17) | (1 << 21),
        (1 << 1) | (1 << 5)
    );
    check!("sve2-sm4", SVE | (1 << 18) | (1 << 19), (1 << 1) | (1 << 6));
    check!("f32mm", SVE, 1 << 10);
    check!("f64mm", SVE, 1 << 11);
    check!("frintts", 3, 1 << 8);
    check!("faminmax", 3, 1 << 50);
    check!("lut", 3, 1 << 49);
    check!("rand", 0, 1 << 16);
    check!("sve2p1", SVE, (1 << 1) | (1 << 36));
    check!("aes", 3 | (1 << 3) | (1 << 4));
    check!("sha2", 3 | (1 << 5) | (1 << 6));
    check!("sha3", 3 | (1 << 5) | (1 << 6) | (1 << 17) | (1 << 21));
    check!("sm4", 3 | (1 << 18) | (1 << 19));
    check!("bti", 0, 1 << 17);
    check!("dpb", 1 << 16);
    check!("dpb2", 1 << 16, 1);
    check!("ecv", 0, 1 << 19);
    check!("flagm", 1 << 27);
    check!("flagm2", 1 << 27, 1 << 7);
    check!("hbc", 0, 1 << 44);
    check!("lse128", 1 << 8, 1 << 47);
    check!("mte", 0, 1 << 18);
    check!("paca", (1 << 30) | (1 << 31));
    check!("pacg", (1 << 30) | (1 << 31));
    check!("rcpc", 1 << 15);
    check!("rcpc2", (1 << 15) | (1 << 26));
    check!("rcpc3", (1 << 15) | (1 << 26), 1 << 46);
    check!("rdm", 3 | (1 << 12));
    check!("ssbs", 1 << 28);
    check!("wfxt", 0, 1 << 31);

    const FP8: u64 = (1 << 14) | (0b1111 << 48) | (3 << 55);
    check!("fp8", 3, FP8);
    check!("fp8fma", 3, FP8 | (1 << 52));
    check!("fp8dot4", 3, FP8 | (3 << 52));
    check!("fp8dot2", 3, FP8 | (7 << 52));
    check!("fpmr", 0, 1 << 48);

    const FP16: u64 = 3 | (1 << 9) | (1 << 10);
    const SME: u64 = (1 << 14) | (1 << 23);
    const SME2: u64 = SME | (1 << 37);
    check!("sme", FP16, SME);
    check!("sme-i16i64", FP16, SME | (1 << 24));
    check!("sme-f64f64", FP16, SME | (1 << 25));
    check!("sme-fa64", SVE, SME | (1 << 1) | (1 << 30) | (1 << 12));
    check!("sme2", FP16, SME2);
    check!("sme2p1", FP16, SME2 | (1 << 38));
    check!("sme-b16b16", FP16, SME2 | (1 << 41) | (1 << 45));
    check!("sme-f16f16", FP16, SME2 | (1 << 42));
    check!("sme-lutv2", FP16, SME2 | (1 << 57));
    check!("sme-f8f32", FP16, SME2 | FP8 | (1 << 59));
    check!("sme-f8f16", FP16, SME2 | FP8 | (3 << 58));
    check!("ssve-fp8fma", FP16, SME2 | FP8 | (1 << 60));
    check!("ssve-fp8dot4", FP16, SME2 | FP8 | (3 << 60));
    check!("ssve-fp8dot2", FP16, SME2 | FP8 | (7 << 60));
    assert!(!cpufeatures::check!(@words [u64::MAX; 2], "pauth-lr"));
    assert!(!cpufeatures::check!(@words [3 | (1 << 9), 0], "neon"));
}

#[test]
fn fp8_query_with_strict_lints() {
    #[allow(unexpected_cfgs)]
    let fpmr_enabled = cfg!(target_feature = "fpmr");
    #[forbid(unexpected_cfgs)]
    {
        cpufeatures::new!(fpmr, "fpmr");
        cpufeatures::new!(fp8, "fp8");
        cpufeatures::new!(both, "fp8", "fpmr");

        let fp8_enabled = cfg!(target_feature = "fp8");
        let detected = both::get();
        if cfg!(miri) {
            assert_eq!(fpmr::get(), fpmr_enabled);
            assert_eq!(detected, fp8_enabled && fpmr_enabled);
        } else if fp8_enabled == fpmr_enabled {
            assert_eq!(detected, fp8::get() && fpmr::get());
        } else {
            assert!(!detected || (fp8::get() && fpmr::get()));
        }
    }
}

#[cfg(all(not(miri), target_vendor = "apple"))]
#[test]
fn apple_extension_keys() {
    use cpufeatures::aarch64::sysctlbyname;

    macro_rules! check {
        ($feature:tt, $reported:expr) => {{
            cpufeatures::new!(feature, $feature);
            #[allow(unexpected_cfgs)]
            let enabled = cfg!(target_feature = $feature);
            assert_eq!(feature::get(), enabled || $reported, $feature);
        }};
    }

    check!("faminmax", sysctlbyname(b"hw.optional.arm.FEAT_FAMINMAX\0"));
    check!("lut", sysctlbyname(b"hw.optional.arm.FEAT_LUT\0"));
    check!("fpmr", sysctlbyname(b"hw.optional.arm.FEAT_FPMR\0"));
    check!(
        "fcma",
        sysctlbyname(b"hw.optional.arm.FEAT_FCMA\0")
            || sysctlbyname(b"hw.optional.armv8_3_compnum\0")
    );
    check!(
        "lse",
        sysctlbyname(b"hw.optional.arm.FEAT_LSE\0")
            || sysctlbyname(b"hw.optional.armv8_1_atomics\0")
    );
    let fp16 =
        sysctlbyname(b"hw.optional.arm.FEAT_FP16\0") || sysctlbyname(b"hw.optional.neon_fp16\0");
    check!("fp16", fp16);
    check!(
        "fhm",
        fp16 && (sysctlbyname(b"hw.optional.arm.FEAT_FHM\0")
            || sysctlbyname(b"hw.optional.armv8_2_fhm\0"))
    );
    check!(
        "sha3",
        (sysctlbyname(b"hw.optional.arm.FEAT_SHA512\0")
            || sysctlbyname(b"hw.optional.armv8_2_sha512\0"))
            && (sysctlbyname(b"hw.optional.arm.FEAT_SHA3\0")
                || sysctlbyname(b"hw.optional.armv8_2_sha3\0"))
    );
    let sme = fp16
        && sysctlbyname(b"hw.optional.arm.FEAT_BF16\0")
        && sysctlbyname(b"hw.optional.arm.FEAT_SME\0");
    check!("sme", sme);
    check!("sme2", sme && sysctlbyname(b"hw.optional.arm.FEAT_SME2\0"));
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn optional_sve_components() {
    macro_rules! check {
        ($feature:tt, $neon_bit:expr, $sve_bit:expr $(, $extra:expr)?) => {{
            const NEON: u64 = 3;
            const SVE: u64 = NEON | (1 << 9) | (1 << 10) | (1 << 22);
            let neon_bit = 1 << $neon_bit;
            let sve_bit = 1 << $sve_bit;
            for (hwcap, bits, supported) in [
                (NEON, neon_bit, true),
                (NEON, sve_bit, false),
                (SVE, neon_bit, false),
                (SVE, sve_bit, false),
                (SVE, neon_bit | sve_bit, true),
            ] {
                assert_eq!(
                    cpufeatures::check!(@words [hwcap, bits $(| $extra)?], $feature),
                    supported,
                    "{} with HWCAP={hwcap:#x}, HWCAP2 bits={bits:#x}",
                    $feature
                );
            }
        }};
    }
    check!("bf16", 14, 12);
    check!("sve-b16b16", 14, 12, 1 << 45);
    check!("i8mm", 13, 9);
}
