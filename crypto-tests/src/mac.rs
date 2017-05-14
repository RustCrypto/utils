use crypto_mac::Mac;

pub struct MacTest {
    pub name: &'static str,
    pub key: &'static [u8],
    pub input: &'static [u8],
    pub output: &'static [u8],
}

#[macro_export]
macro_rules! new_mac_tests {
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

pub fn mac_test<M: Mac>(tests: &[MacTest]) {
    for test in tests.iter() {
        let mut hmac = M::new(test.key);
        hmac.input(&test.input[..]);
        assert!(hmac.verify(test.output));
    }

    // incremental test
    for test in tests.iter() {
        let mut hmac = M::new(test.key);
        for i in 0..test.input.len() {
            hmac.input(&test.input[i..i + 1]);
        }
        assert!(hmac.verify(test.output));
    }
}
