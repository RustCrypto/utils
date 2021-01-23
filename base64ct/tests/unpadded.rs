//! Unpadded Base64 tests

use base64ct::{decode, decode_in_place_unpadded, decoded_len, encode, encoded_len, Error};

#[cfg(feature = "alloc")]
use base64ct::{decode_vec, encode_string};

struct TestVector {
    raw: &'static [u8],
    b64: &'static str,
}

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
        raw: b"\xFF\xFF\xFF\xFF\xFF",
        b64: "//////8",
    },
    TestVector {
        raw: b"\x40\xC1\x3F\xBD\x05\x4C\x72\x2A\xA3\xC2\xF2\x11\x73\xC0\x69\xEA\
               \x49\x7D\x35\x29\x6B\xCC\x24\x65\xF6\xF9\xD0\x41\x08\x7B\xD7\xA9",
        b64: "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k",
    },
];

#[test]
fn encode_test_vectors() {
    let mut buf = [0u8; 1024];

    for vector in TEST_VECTORS {
        let out = encode(vector.raw, &mut buf, false).unwrap();
        assert_eq!(encoded_len(vector.raw, false), vector.b64.len());
        assert_eq!(vector.b64, &out[..]);

        #[cfg(feature = "alloc")]
        {
            let out = encode_string(vector.raw, false);
            assert_eq!(vector.b64, &out[..]);
        }
    }
}

#[test]
fn decode_test_vectors() {
    let mut buf = [0u8; 1024];

    for vector in TEST_VECTORS {
        let out = decode(vector.b64, &mut buf, false).unwrap();
        assert_eq!(decoded_len(vector.b64, false), out.len());
        assert_eq!(vector.raw, &out[..]);

        let n = vector.b64.len();
        buf[..n].copy_from_slice(vector.b64.as_bytes());
        let out = decode_in_place_unpadded(&mut buf[..n]).unwrap();
        assert_eq!(vector.raw, out);

        #[cfg(feature = "alloc")]
        {
            let out = decode_vec(vector.b64, false).unwrap();
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
        let encoded = encode(&data[..i], &mut inbuf, false).unwrap();

        // Make sure it round trips
        let decoded = decode(encoded, &mut outbuf, false).unwrap();
        assert_eq!(decoded, &data[..i]);

        let elen = encode(&data[..i], &mut inbuf, false).unwrap().len();
        let buf = &mut inbuf[..elen];
        let decoded = decode_in_place_unpadded(buf).unwrap();
        assert_eq!(decoded, &data[..i]);

        #[cfg(feature = "alloc")]
        {
            let encoded = encode_string(&data[..i], false);
            let decoded = decode_vec(&encoded, false).unwrap();
            assert_eq!(decoded, &data[..i]);
        }
    }
}

#[test]
fn reject_trailing_equals() {
    let input = "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k=";
    let mut buf = [0u8; 1024];
    assert_eq!(decode(input, &mut buf, false), Err(Error::InvalidEncoding));
}

#[test]
fn reject_trailing_whitespace() {
    let input = "QME/vQVMciqjwvIRc8Bp6kl9NSlrzCRl9vnQQQh716k\n";
    let mut buf = [0u8; 1024];
    assert_eq!(decode(input, &mut buf, false), Err(Error::InvalidEncoding));
}
