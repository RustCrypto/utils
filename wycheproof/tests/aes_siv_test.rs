use aes_siv::{aead::generic_array::GenericArray, siv::Aes256Siv};
use serde::Deserialize;

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
    #[serde(rename = "keySize")]
    pub key_size: u32,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "wycheproof::hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub aad: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub ct: Vec<u8>,
}

const AES_SIV_KEY_SIZE: usize = 64; // 512 bits

#[test]
fn test_aes_siv() {
    let filename = "testvectors/aes_siv_cmac_test.json";
    println!("wycheproof file '{}'", filename);
    let bytes = wycheproof::wycheproof_data(filename);
    let suite: TestSuite = serde_json::from_slice(&bytes).unwrap();

    for g in &suite.test_groups {
        if (g.key_size / 8) as usize != AES_SIV_KEY_SIZE {
            println!("   skipping tests for key_size={}", g.key_size);
            continue;
        }
        println!("   key info: key_size={}", g.key_size);
        for tc in &g.tests {
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );

            let mut cipher = Aes256Siv::new(*GenericArray::from_slice(&tc.key));
            let got_ct = cipher.encrypt(&[&tc.aad], &tc.msg).unwrap();
            match &tc.case.result {
                wycheproof::CaseResult::Valid => {
                    assert_eq!(got_ct, tc.ct, "{}: incorrect encryption", tc.case.case_id);
                }
                wycheproof::CaseResult::Invalid => {
                    assert_ne!(got_ct, tc.ct, "{}: invalid encryption", tc.case.case_id);
                }
                r => panic!("unknown result type {}", r),
            }

            let pt_result = cipher.decrypt(&[&tc.aad], &tc.ct);
            match tc.case.result {
                wycheproof::CaseResult::Valid => {
                    assert!(
                        pt_result.is_ok(),
                        "{}: unexpected decryption error: {:?}",
                        tc.case.case_id,
                        pt_result
                    );
                    assert_eq!(
                        tc.msg,
                        pt_result.unwrap(),
                        "{}: incorrect decryption",
                        tc.case.case_id
                    );
                }
                wycheproof::CaseResult::Invalid => {
                    assert!(
                        pt_result.is_err(),
                        "{}: decryption error expected",
                        tc.case.case_id
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}
