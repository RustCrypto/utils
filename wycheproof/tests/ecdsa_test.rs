use elliptic_curve::sec1::EncodedPoint;
use generic_array::typenum::Unsigned;
use p256::ecdsa::{signature::Verifier, Signature};
use serde::Deserialize;
use signature::Signature as _;
use std::collections::HashSet;
use wycheproof::hex_string;

#[derive(Debug, Deserialize)]
struct TestSuite {
    #[serde(flatten)]
    pub suite: wycheproof::Suite,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<TestGroup>,
}

#[derive(Debug, Deserialize)]
struct TestGroup {
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[serde(rename = "keyDer")]
    pub key_der: String,
    #[serde(rename = "keyPem")]
    pub key_pem: String,
    pub sha: String,
    pub key: TestKey,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
struct TestKey {
    curve: String,
    #[serde(rename = "type")]
    key_type: String,
    #[serde(with = "hex_string")]
    wx: Vec<u8>,
    #[serde(with = "hex_string")]
    wy: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "hex_string")]
    pub sig: Vec<u8>,
}

#[derive(Debug)]
enum SigEncoding {
    Der,
    IeeeP1363,
}

#[test]
fn test_ecdsa() {
    struct TestVector {
        filename: &'static str,
        encoding: SigEncoding,
        skip_tests: HashSet<i32>,
    }
    let vectors = vec![
        TestVector {
            filename: "ecdsa_test.json",
            encoding: SigEncoding::Der,
            // Test 4 uses ASN.1 long form encoding of sequence length, which is not DER.
            // TODO: remove when upstream fix released
            // 4: non-minimal SEQUENCE length
            // 18, 19, 39: SEQUENCE too short
            skip_tests: vec![4, 18, 19, 39].into_iter().collect(),
        },
        TestVector {
            filename: "ecdsa_secp256r1_sha256_p1363_test.json",
            encoding: SigEncoding::IeeeP1363,
            // TODO: remove when upstream fix released
            // 119: SEQUENCE too short
            skip_tests: vec![119].into_iter().collect(),
        },
        /* TODO: more ECDSA curves
                TestVector {
                    filename: "ecdsa_secp384r1_sha512_p1363_test.json",
                    encoding: SigEncoding::IeeeP1363,
                    skip_tests: HashSet::new(),
                },
                TestVector {
                    filename: "ecdsa_secp521r1_sha512_p1363_test.json",
                    encoding: SigEncoding::IeeeP1363,
                    skip_tests: HashSet::new(),
                },
        */
    ];
    for v in vectors {
        wycheproof_test(v.filename, v.encoding, v.skip_tests)
    }
}

fn wycheproof_test(filename: &str, encoding: SigEncoding, skip_tests: HashSet<i32>) {
    println!(
        "wycheproof file 'testvectors/{}', encoding '{:?}'",
        filename, encoding
    );
    let bytes = wycheproof::wycheproof_data(&format!("testvectors/{}", filename));
    let suite: TestSuite = serde_json::from_slice(&bytes).unwrap();
    let mut skipped_hashes = HashSet::new();
    let mut skipped_curves = HashSet::new();
    for g in &suite.test_groups {
        if g.key.curve != "secp256r1" {
            if !skipped_curves.contains(&g.key.curve) {
                println!("skipping tests for unsupported curve {}", g.key.curve);
                skipped_curves.insert(g.key.curve.clone());
            }
            continue;
        }
        if g.sha != "SHA-256" {
            if !skipped_hashes.contains(&g.sha) {
                println!("skipping tests for unsupported hash {}", g.sha);
                skipped_hashes.insert(g.sha.clone());
            }
            continue;
        }
        println!(
            "   key info: {}, {}, {:?}, {}, {}",
            g.sha,
            g.key.curve,
            encoding,
            hex::encode(&g.key.wx),
            hex::encode(&g.key.wy),
        );
        let x = element_from_padded_slice::<p256::NistP256>(&g.key.wx);
        let y = element_from_padded_slice::<p256::NistP256>(&g.key.wy);
        let pt = EncodedPoint::from_affine_coordinates(&x, &y, /* compress= */ false);
        let verify_key = p256::ecdsa::VerifyKey::from_encoded_point(&pt).unwrap();

        for tc in &g.tests {
            if skip_tests.contains(&tc.case.case_id) {
                println!(
                    "     SKIP case {} [{}] {}: sig={}",
                    tc.case.case_id,
                    tc.case.result,
                    tc.case.comment,
                    hex::encode(&tc.sig)
                );
                continue;
            }
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );
            let sig_result = match encoding {
                SigEncoding::Der => Signature::from_asn1(&tc.sig),
                SigEncoding::IeeeP1363 => Signature::from_bytes(&tc.sig),
            };
            let signature = match sig_result {
                Ok(s) => s,
                Err(_) => {
                    assert_ne!(tc.case.result, wycheproof::CaseResult::Valid);
                    continue;
                }
            };

            let result = verify_key.verify(&tc.msg, &signature);
            match tc.case.result {
                wycheproof::CaseResult::Valid => assert!(
                    result.is_ok(),
                    "failed in test case {} with result {:?}",
                    tc.case.case_id,
                    result
                ),
                wycheproof::CaseResult::Invalid => assert!(
                    result.is_err(),
                    "unexpected success in test case {}",
                    tc.case.case_id
                ),
                wycheproof::CaseResult::Acceptable => {}
            }
        }
    }
}

// Build a field element but allow for too-short input (left pad with zeros)
// or too-long input (check excess leftmost bytes are zeros).
fn element_from_padded_slice<C: elliptic_curve::Curve>(
    data: &[u8],
) -> elliptic_curve::FieldBytes<C> {
    let point_len = C::FieldSize::to_usize();
    if data.len() >= point_len {
        let offset = data.len() - point_len;
        for v in data.iter().take(offset) {
            assert_eq!(*v, 0, "EcdsaVerifier: point too large");
        }
        elliptic_curve::FieldBytes::<C>::from_slice(&data[offset..]).clone()
    } else {
        let mut data_copy = vec![0; point_len];
        data_copy[(point_len - data.len())..].copy_from_slice(data);
        elliptic_curve::FieldBytes::<C>::clone_from_slice(&data_copy)
    }
}
