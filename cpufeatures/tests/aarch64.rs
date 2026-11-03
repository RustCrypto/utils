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
    #[cfg(miri)]
    const {
        assert!(!cpufeatures::__detect_target_features!("neon"));
    };
}

#[test]
fn neon_matches_std() {
    cpufeatures::new!(neon, "neon");
    assert_eq!(neon::get(), std::arch::is_aarch64_feature_detected!("neon"));
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn capability_masks() {
    macro_rules! check {
        ($feature:tt, $mask:expr $(, $mask2:expr)?) => {{
            let full: [u64; 2] = [$mask, 0 $(| $mask2)?];
            assert_eq!(cpufeatures::check!(@hwcap2 $feature), full[1] != 0, $feature);
            assert_eq!(cpufeatures::check!(full[0], $feature), full[1] == 0, $feature);
            assert!(cpufeatures::check!(@words full, $feature), $feature);
            assert!(cpufeatures::check!(@words [u64::MAX; 2], $feature), $feature);
            assert!(!cpufeatures::check!(@words full.map(|word| !word), $feature), $feature);
            for word in 0..2 {
                for bit in 0..64 {
                    if full[word] & (1 << bit) != 0 {
                        let mut missing = full;
                        missing[word] &= !(1 << bit);
                        assert!(!cpufeatures::check!(@words missing, $feature), $feature);
                    }
                }
            }
            if full[0] & 3 == 3 && full[0] & (1 << 9) == 0 {
                assert!(!cpufeatures::check!(@words [full[0] | (1 << 9), full[1]], $feature), $feature);
            }
        }};
    }
    check!("neon", 3);
    check!("fcma", 3 | (1 << 14));
    check!("fhm", 3 | (1 << 9) | (1 << 10) | (1 << 23));
    check!("jsconv", 3 | (1 << 13));
    check!("sve-b16b16", 3, (1 << 14) | (1 << 45));
    const SVE: u64 = 3 | (1 << 9) | (1 << 10) | (1 << 22);
    check!("crc", 1 << 7);
    check!("cssc", 0, 1 << 34);
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
}

#[cfg(not(miri))]
#[test]
fn extended_detection() {
    let runtime = cfg!(any(
        target_vendor = "apple",
        target_os = "linux",
        target_os = "android"
    ));
    macro_rules! check {
        ($($feature:tt),+) => {$({
            cpufeatures::new!(detected, $feature);
            assert_eq!(detected::get(), cfg!(target_feature = $feature) || (runtime && std::arch::is_aarch64_feature_detected!($feature)), $feature);
        })+};
    }
    check!(
        "crc",
        "bf16",
        "i8mm",
        "sve",
        "sve2",
        "sve2-aes",
        "sve2-bitperm",
        "sve2-sha3",
        "sve2-sm4",
        "f32mm",
        "f64mm",
        "fcma",
        "fhm",
        "jsconv",
        "lse2",
        "frintts",
        "rand"
    );
}

#[cfg(not(miri))]
#[test]
fn recent_detection() {
    macro_rules! check {
        ($($feature:tt),+) => {{
            cpufeatures::new!(combined, $($feature),+);
            assert_eq!(combined::get(), true $(& (cfg!(target_feature = $feature) || cpufeatures::__detect_target_features!($feature)))+);
        } $({
            cpufeatures::new!(detected, $feature);
            assert_eq!(detected::get(), cfg!(target_feature = $feature) || cpufeatures::__detect_target_features!($feature), $feature);
        })+};
    }
    check!("cssc", "faminmax", "lut", "mops", "sve2p1", "sve-b16b16");
    #[cfg(target_vendor = "apple")]
    assert!(!cpufeatures::aarch64::sysctlbyname(
        b"hw.optional.cpufeatures_missing\0"
    ));
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn optional_sve_components() {
    macro_rules! check {
        ($feature:tt, $bit:expr, $sve_bit:expr $(, $extra:expr)?) => {
            for sve in [false, true] {
                let hwcap = if sve { 3 | (1 << 9) | (1 << 10) | (1 << 22) } else { 3 };
                for bits in [0, 1 << $bit, 1 << $sve_bit, (1 << $bit) | (1 << $sve_bit)] {
                    let hwcap2 = bits $(| $extra)?;
                    assert_eq!(cpufeatures::check!(@words [hwcap, hwcap2], $feature),
                        hwcap2 & (1 << $bit) != 0 && (!sve || hwcap2 & (1 << $sve_bit) != 0), $feature);
                }
            }
        };
    }
    check!("bf16", 14, 12);
    check!("sve-b16b16", 14, 12, 1 << 45);
    check!("i8mm", 13, 9);
}
