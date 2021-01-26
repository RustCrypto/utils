//! Base64 tests

/// Generate test suite for a particular Base64 flavor
macro_rules! impl_tests {
    () => {
        use crate::TestVector;
        use base64ct::Error;

        #[test]
        fn encode_test_vectors() {
            let mut buf = [0u8; 1024];

            for vector in TEST_VECTORS {
                let out = encode(vector.raw, &mut buf).unwrap();
                assert_eq!(encoded_len(vector.raw), vector.b64.len());
                assert_eq!(vector.b64, &out[..]);

                #[cfg(feature = "alloc")]
                {
                    let out = encode_string(vector.raw);
                    assert_eq!(vector.b64, &out[..]);
                }
            }
        }

        #[test]
        fn decode_test_vectors() {
            let mut buf = [0u8; 1024];

            for vector in TEST_VECTORS {
                let out = decode(vector.b64, &mut buf).unwrap();
                assert_eq!(vector.raw, &out[..]);

                let n = vector.b64.len();
                buf[..n].copy_from_slice(vector.b64.as_bytes());
                let out = decode_in_place(&mut buf[..n]).unwrap();
                assert_eq!(vector.raw, out);

                #[cfg(feature = "alloc")]
                {
                    let out = decode_vec(vector.b64).unwrap();
                    assert_eq!(vector.raw, &out[..]);
                }
            }
        }

        #[test]
        fn encode_and_decode_various_lengths() {
            let data = [b'X'; 64];
            let mut inbuf = [0u8; 1024];
            let mut outbuf = [0u8; 1024];

            for i in 0..data.len() {
                let encoded = encode(&data[..i], &mut inbuf).unwrap();

                // Make sure it round trips
                let decoded = decode(encoded, &mut outbuf).unwrap();
                assert_eq!(decoded, &data[..i]);

                let elen = encode(&data[..i], &mut inbuf).unwrap().len();
                let buf = &mut inbuf[..elen];
                let decoded = decode_in_place(buf).unwrap();
                assert_eq!(decoded, &data[..i]);

                #[cfg(feature = "alloc")]
                {
                    let encoded = encode_string(&data[..i]);
                    let decoded = decode_vec(&encoded).unwrap();
                    assert_eq!(decoded, &data[..i]);
                }
            }
        }

        #[test]
        fn reject_trailing_whitespace() {
            let input = "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k\n";
            let mut buf = [0u8; 1024];
            assert_eq!(decode(input, &mut buf), Err(Error::InvalidEncoding));
        }
    };
}

struct TestVector {
    raw: &'static [u8],
    b64: &'static str,
}

/// Standard Base64 tests
mod base64 {
    /// Standard Base64 with `=` padding
    mod padded {
        use base64ct::padded::*;

        const TEST_VECTORS: &[TestVector] = &[
            TestVector { raw: b"", b64: "" },
            TestVector {
                raw: b"\0",
                b64: "AA==",
            },
            TestVector {
                raw: b"***",
                b64: "Kioq",
            },
            TestVector {
                raw: b"\x01\x02\x03\x04",
                b64: "AQIDBA==",
            },
            TestVector {
                raw: b"\xAD\xAD\xAD\xAD\xAD",
                b64: "ra2tra0=",
            },
            TestVector {
                raw: b"\xFF\xEF\xFE\xFF\xEF\xFE",
                b64: "/+/+/+/+",
            },
            TestVector {
                raw: b"\xFF\xFF\xFF\xFF\xFF",
                b64: "//////8=",
            },
            TestVector {
                raw: b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
                           \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
                b64: "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=",
            },
            TestVector {
                raw: b"\x00\x10\x83\x10Q\x87 \x92\x8B0\xD3\x8FA\x14\x93QU\x97a\x96\x9Bq\
                       \xD7\x9F\x82\x18\xA3\x92Y\xA7\xA2\x9A\xAB\xB2\xDB\xAF\xC3\x1C\xB3\
                       \xFB\xF0\x00",
                b64: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/AA",
            },
        ];

        impl_tests!();
    }

    /// Standard Base64 *without* padding
    mod unpadded {
        use base64ct::unpadded::*;

        const TEST_VECTORS: &[TestVector] = &[
            TestVector { raw: b"", b64: "" },
            TestVector {
                raw: b"\0",
                b64: "AA",
            },
            TestVector {
                raw: b"***",
                b64: "Kioq",
            },
            TestVector {
                raw: b"\x01\x02\x03\x04",
                b64: "AQIDBA",
            },
            TestVector {
                raw: b"\xAD\xAD\xAD\xAD\xAD",
                b64: "ra2tra0",
            },
            TestVector {
                raw: b"\xFF\xEF\xFE\xFF\xEF\xFE",
                b64: "/+/+/+/+",
            },
            TestVector {
                raw: b"\xFF\xFF\xFF\xFF\xFF",
                b64: "//////8",
            },
            TestVector {
                raw: b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
                       \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
                b64: "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k",
            },
            TestVector {
                raw: b"\x00\x10\x83\x10Q\x87 \x92\x8B0\xD3\x8FA\x14\x93QU\x97a\x96\x9Bq\
                       \xD7\x9F\x82\x18\xA3\x92Y\xA7\xA2\x9A\xAB\xB2\xDB\xAF\xC3\x1C\xB3\
                       \xFB\xF0\x00",
                b64: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/AA",
            },
        ];

        #[test]
        fn unpadded_reject_trailing_equals() {
            let input = "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=";
            let mut buf = [0u8; 1024];
            assert_eq!(decode(input, &mut buf), Err(Error::InvalidEncoding));
        }

        impl_tests!();
    }
}

/// URL-safe Base64 tests
mod base64url {
    /// URL-safe Base64 with `=` padding
    mod padded {
        use base64ct::url::padded::*;

        const TEST_VECTORS: &[TestVector] = &[
            TestVector { raw: b"", b64: "" },
            TestVector {
                raw: b"\0",
                b64: "AA==",
            },
            TestVector {
                raw: b"***",
                b64: "Kioq",
            },
            TestVector {
                raw: b"\x01\x02\x03\x04",
                b64: "AQIDBA==",
            },
            TestVector {
                raw: b"\xAD\xAD\xAD\xAD\xAD",
                b64: "ra2tra0=",
            },
            TestVector {
                raw: b"\xFF\xEF\xFE\xFF\xEF\xFE",
                b64: "_-_-_-_-",
            },
            TestVector {
                raw: b"\xFF\xFF\xFF\xFF\xFF",
                b64: "______8=",
            },
            TestVector {
                raw: b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
                           \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
                b64: "QME_vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=",
            },
            TestVector {
                raw: b"\x00\x10\x83\x10Q\x87 \x92\x8B0\xD3\x8FA\x14\x93QU\x97a\x96\x9Bq\
                       \xD7\x9F\x82\x18\xA3\x92Y\xA7\xA2\x9A\xAB\xB2\xDB\xAF\xC3\x1C\xB3\
                       \xFB\xF0\x00",
                b64: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_AA",
            },
        ];

        impl_tests!();
    }

    /// URL-safe Base64 *without* padding
    mod unpadded {
        use base64ct::url::unpadded::*;

        const TEST_VECTORS: &[TestVector] = &[
            TestVector { raw: b"", b64: "" },
            TestVector {
                raw: b"\0",
                b64: "AA",
            },
            TestVector {
                raw: b"***",
                b64: "Kioq",
            },
            TestVector {
                raw: b"\x01\x02\x03\x04",
                b64: "AQIDBA",
            },
            TestVector {
                raw: b"\xAD\xAD\xAD\xAD\xAD",
                b64: "ra2tra0",
            },
            TestVector {
                raw: b"\xFF\xEF\xFE\xFF\xEF\xFE",
                b64: "_-_-_-_-",
            },
            TestVector {
                raw: b"\xFF\xFF\xFF\xFF\xFF",
                b64: "______8",
            },
            TestVector {
                raw: b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
               \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
                b64: "QME_vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k",
            },
            TestVector {
                raw: b"\x00\x10\x83\x10Q\x87 \x92\x8B0\xD3\x8FA\x14\x93QU\x97a\x96\x9Bq\
               \xD7\x9F\x82\x18\xA3\x92Y\xA7\xA2\x9A\xAB\xB2\xDB\xAF\xC3\x1C\xB3\
               \xFB\xF0\x00",
                b64: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_AA",
            },
        ];

        impl_tests!();

        #[test]
        fn unpadded_reject_trailing_equals() {
            let input = "QME_vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=";
            let mut buf = [0u8; 1024];
            assert_eq!(decode(input, &mut buf), Err(Error::InvalidEncoding));
        }
    }
}
