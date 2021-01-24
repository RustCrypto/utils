//! Padded Base64 tests

use base64ct::{decode, Error};

#[cfg(feature = "alloc")]
use base64ct::{decode_vec, decoded_len, encode_string, encoded_len};

#[cfg(feature = "alloc")]
struct TestVector {
    raw: &'static [u8],
    b64: &'static str,
}

#[cfg(feature = "alloc")]
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
        raw: b"\xFF\xFF\xFF\xFF\xFF",
        b64: "//////8=",
    },
    TestVector {
        raw: b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
                   \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
        b64: "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=",
    },
];

#[test]
#[cfg(feature = "alloc")]
fn encode_test_vectors() {
    for vector in TEST_VECTORS {
        let out = encode_string(vector.raw, true);
        assert_eq!(encoded_len(vector.raw, true), out.len());
        assert_eq!(vector.b64, &out[..]);
    }
}

#[test]
#[cfg(feature = "alloc")]
fn decode_test_vectors() {
    for vector in TEST_VECTORS {
        let out = decode_vec(vector.b64, true).unwrap();
        assert_eq!(decoded_len(vector.b64, true), out.len());
        assert_eq!(vector.raw, &out[..]);
    }
}

#[test]
#[cfg(feature = "alloc")]
fn encode_and_decode_various_lengths() {
    let data = [b'X'; 64];

    for i in 0..data.len() {
        let encoded = encode_string(&data[..i], true);

        // Make sure it round trips
        let decoded = decode_vec(&encoded, true).unwrap();

        assert_eq!(decoded.as_slice(), &data[..i]);
    }
}

#[test]
fn reject_trailing_whitespace() {
    let input = "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k\n";
    let mut buf = [0u8; 1024];
    assert_eq!(decode(input, &mut buf, true), Err(Error::InvalidEncoding));
}
