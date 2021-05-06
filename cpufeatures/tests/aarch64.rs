//! ARM64 tests

#![cfg(target_arch = "aarch64")]

cpufeatures::new!(armcaps_aes_sha2_sha3, "aes", "sha2", "sha3");

#[test]
fn init() {
    let token: armcaps_aes_sha2_sha3::InitToken = armcaps_aes_sha2_sha3::init();
    assert_eq!(token.get(), armcaps_aes_sha2_sha3::get());
}

#[test]
fn init_get() {
    let (token, val) = armcaps_aes_sha2_sha3::init_get();
    assert_eq!(val, token.get());
}
