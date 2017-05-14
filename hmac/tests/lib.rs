#![no_std]
#[macro_use]
extern crate crypto_tests;
extern crate hmac;
extern crate md_5 as md5;
extern crate sha2;

use crypto_tests::mac::{mac_test, MacTest};
use hmac::Hmac;

#[test]
fn hmac_md5() {
    // Test vectors from RFC 2104, plus wiki test
    let tests = new_mac_tests!("md5/1", "md5/2", "md5/3", "md5/4");
    mac_test::<Hmac<md5::Md5>>(&tests);
}


#[test]
fn hmac_sha224() {
    // Test vectors from RFC 4231
    let tests = new_mac_tests!(
        "sha224/1", "sha224/2", "sha224/3", "sha224/4",
        "sha224/5", "sha224/6", "sha224/7"
    );
    mac_test::<Hmac<sha2::Sha224>>(&tests);
}

#[test]
fn hmac_sha256() {
    // Test vectors from RFC 4231
    let tests = new_mac_tests!(
        "sha256/1", "sha256/2", "sha256/3", "sha256/4",
        "sha256/5", "sha256/6", "sha256/7"
    );
    mac_test::<Hmac<sha2::Sha256>>(&tests);
}

#[test]
fn hmac_sha384() {
    // Test vectors from RFC 4231
    let tests = new_mac_tests!(
        "sha384/1", "sha384/2", "sha384/3", "sha384/4",
        "sha384/5", "sha384/6", "sha384/7"
    );
    mac_test::<Hmac<sha2::Sha384>>(&tests);
}

#[test]
fn hmac_sha512() {
    // Test vectors from RFC 4231
    let tests = new_mac_tests!(
        "sha512/1", "sha512/2", "sha512/3", "sha512/4",
        "sha512/5", "sha512/6", "sha512/7"
    );
    mac_test::<Hmac<sha2::Sha512>>(&tests);
}
