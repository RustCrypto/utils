use digest::{Digest, Input, VariableOutput};

pub struct Test {
    pub name: &'static str,
    pub input: &'static [u8],
    pub output: &'static [u8],
}

#[macro_export]
macro_rules! new_tests {
    ( $( $name:expr ),*  ) => {
        [$(
            Test {
                name: $name,
                input: include_bytes!(concat!("data/", $name, ".input.bin")),
                output: include_bytes!(concat!("data/", $name, ".output.bin")),
            },
        )*]
    };
    ( $( $name:expr ),+, ) => (new_tests!($($name),+))
}

pub fn main_test<D: Digest + Default>(tests: &[Test]) {
    // Test that it works when accepting the message all at once
    for t in tests.iter() {
        let mut sh = D::default();
        sh.input(t.input);

        let out = sh.result();

        assert_eq!(out[..], t.output[..]);
    }

    // Test that it works when accepting the message in pieces
    for t in tests.iter() {
        let mut sh = D::default();
        let len = t.input.len();
        let mut left = len;
        while left > 0 {
            let take = (left + 1) / 2;
            sh.input(&t.input[len - left..take + len - left]);
            left = left - take;
        }

        let out = sh.result();

        assert_eq!(out[..], t.output[..]);
    }
}

pub fn variable_test<D: Input + VariableOutput + Default>(tests: &[Test]) {
    let mut buf = [0u8; 1024];
    // Test that it works when accepting the message all at once
    for t in tests.iter() {
        let mut sh = D::default();
        sh.digest(t.input);

        let out = sh.variable_result(&mut buf[..t.output.len()]).unwrap();

        assert_eq!(out[..], t.output[..]);
    }

    // Test that it works when accepting the message in pieces
    for t in tests.iter() {
        let mut sh = D::default();
        let len = t.input.len();
        let mut left = len;
        while left > 0 {
            let take = (left + 1) / 2;
            sh.digest(&t.input[len - left..take + len - left]);
            left = left - take;
        }

        let out = sh.variable_result(&mut buf[..t.output.len()]).unwrap();

        assert_eq!(out[..], t.output[..]);
    }
}


pub fn one_million_a<D: Digest + Default>(expected: &[u8]) {
    let mut sh = D::default();
    for _ in 0..50000 {
        sh.input(&[b'a'; 10]);
    }
    sh.input(&[b'a'; 500000]);
    let out = sh.result();
    assert_eq!(out[..], expected[..]);
}


#[macro_export]
macro_rules! bench_digest {
    ($name:ident, $engine:path, $bs:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut d = <$engine>::default();
            let data = [0; $bs];

            b.iter(|| {
                d.input(&data);
            });

            b.bytes = $bs;
        }
    };

    ($engine:path) => {
        extern crate test;
        extern crate digest;

        use test::Bencher;
        use digest::Digest;

        bench_digest!(bench1_16, $engine, 1<<4);
        bench_digest!(bench2_64, $engine, 1<<6);
        bench_digest!(bench3_256, $engine, 1<<8);
        bench_digest!(bench4_1k, $engine, 1<<10);
        bench_digest!(bench5_8k, $engine, 1<<13);
        bench_digest!(bench6_64k, $engine, 1<<16);
    }
}
