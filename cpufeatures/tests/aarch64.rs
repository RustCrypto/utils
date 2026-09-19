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

cpufeatures::new!(
    armcaps_multi;
    Sha3Aes: "sha3", "aes";
    Sha2Aes: "sha2", "aes";
    Aes: "aes";
    Soft;
);

cpufeatures::new!(armcaps_sha3_aes, "sha3", "aes");
cpufeatures::new!(armcaps_sha2_aes, "sha2", "aes");
cpufeatures::new!(armcaps_aes, "aes");

#[test]
fn multi_init() {
    let token: armcaps_multi::InitToken = armcaps_multi::init();
    assert_eq!(token.get(), armcaps_multi::get());
}

#[test]
fn multi_init_get() {
    let (token, val) = armcaps_multi::init_get();
    assert_eq!(val, token.get());
}

#[test]
fn multi_matches_individual_detection() {
    use armcaps_multi::Features;

    let expected = if armcaps_sha3_aes::get() {
        Features::Sha3Aes
    } else if armcaps_sha2_aes::get() {
        Features::Sha2Aes
    } else if armcaps_aes::get() {
        Features::Aes
    } else {
        Features::Soft
    };

    assert_eq!(armcaps_multi::get(), expected);
}

#[test]
fn multi_cached_value_round_trips() {
    // The first call performs detection, subsequent ones decode the cached tag.
    let detected = armcaps_multi::get();

    assert_eq!(detected, armcaps_multi::get());
    assert_eq!(detected, armcaps_multi::init_get().1);
    assert_eq!(detected, armcaps_multi::init().get());
}
