//! x86/x86_64 tests

#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

cpufeatures::new!(cpuid_aes_sha, "aes", "sha");

#[test]
fn init() {
    let token: cpuid_aes_sha::InitToken = cpuid_aes_sha::init();
    assert_eq!(token.get(), cpuid_aes_sha::get());
}

#[test]
fn init_get() {
    let (token, val) = cpuid_aes_sha::init_get();
    assert_eq!(val, token.get());
}
