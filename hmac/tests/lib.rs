// TODO: move HMAC tests to the hash meta-crate
extern crate hmac;
extern crate md_5 as md5;

use hmac::{Mac, MacResult, Hmac};

pub struct MacTest {
    pub name: &'static str,
    pub key: &'static [u8],
    pub input: &'static [u8],
    pub output: &'static [u8],
}

macro_rules! new_tests {
    ( $( $name:expr ),*  ) => {
        [$(
            MacTest {
                name: $name,
                key: include_bytes!(concat!("data/", $name, ".key.bin")),
                input: include_bytes!(concat!("data/", $name, ".input.bin")),
                output: include_bytes!(concat!("data/", $name, ".output.bin")),
            },
        )*]
    };
}

#[test]
fn hmac_md5() {
    // Test vectors from: http://tools.ietf.org/html/rfc2104
    // Plus wiki test
    let tests = new_tests!("1", "2", "3", "4");
    for test in tests.iter() {
        let mut hmac = Hmac::<md5::Md5>::new(test.key);
        hmac.input(&test.input[..]);
        let result = hmac.result();
        let expected = MacResult::new_from_slice(test.output);
        println!("result: {:?}", result.code());
        println!("expected: {:?}", expected.code());
        assert!(result == expected);
    }

    // incremental test
    for test in tests.iter() {
        let mut hmac = Hmac::<md5::Md5>::new(test.key);
        for i in 0..test.input.len() {
            hmac.input(&test.input[i..i + 1]);
        }
        let result = hmac.result();
        let expected = MacResult::new_from_slice(test.output);
        assert!(result == expected);
    }
}
