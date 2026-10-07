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

cpufeatures::new!(
    cpuid_multi;
    AesSha: "aes", "sha";
    Aes: "aes";
    Sse2: "sse2";
    Soft;
);

cpufeatures::new!(cpuid_aes, "aes");
cpufeatures::new!(cpuid_sse2, "sse2");

#[test]
fn multi_init() {
    let token: cpuid_multi::InitToken = cpuid_multi::init();
    assert_eq!(token.get(), cpuid_multi::get());
}

#[test]
fn multi_init_get() {
    let (token, val) = cpuid_multi::init_get();
    assert_eq!(val, token.get());
}

#[test]
fn multi_matches_individual_detection() {
    use cpuid_multi::Features;

    let expected = if cpuid::get() {
        Features::AesSha
    } else if cpuid_aes::get() {
        Features::Aes
    } else if cpuid_sse2::get() {
        Features::Sse2
    } else {
        Features::Soft
    };

    assert_eq!(cpuid_multi::get(), expected);
}

#[test]
fn multi_cached_value_round_trips() {
    // The first call performs detection, subsequent ones decode the cached tag.
    let detected = cpuid_multi::get();

    assert_eq!(detected, cpuid_multi::get());
    assert_eq!(detected, cpuid_multi::init_get().1);
    assert_eq!(detected, cpuid_multi::init().get());
}
