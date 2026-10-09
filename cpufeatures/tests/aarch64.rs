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

#[cfg(all(not(miri), any(target_os = "linux", target_os = "android")))]
mod capability_masks {
    fn requires_both(detect: impl Fn(u64) -> bool, first: u64, second: u64) {
        for extra in [0, libc::HWCAP_FP | libc::HWCAP_ASIMD] {
            assert!(!detect(extra));
            assert!(!detect(extra | first));
            assert!(!detect(extra | second));
            assert!(detect(extra | first | second));
        }
    }

    #[test]
    fn aes_requires_aes_and_pmull() {
        requires_both(
            |caps| cpufeatures::check!(caps, "aes"),
            libc::HWCAP_AES,
            libc::HWCAP_PMULL,
        );
    }

    #[test]
    fn sha3_requires_sha3_and_sha512() {
        requires_both(
            |caps| cpufeatures::check!(caps, "sha3"),
            libc::HWCAP_SHA3,
            libc::HWCAP_SHA512,
        );
    }

    #[test]
    fn sm4_requires_sm3_and_sm4() {
        requires_both(
            |caps| cpufeatures::check!(caps, "sm4"),
            libc::HWCAP_SM3,
            libc::HWCAP_SM4,
        );
    }
}
