use aes::{Aes128, Aes192, Aes256};
use cmac::{Cmac, Mac, NewMac};
use serde::Deserialize;
use std::cmp::min;

#[derive(Debug, Deserialize)]
pub struct TestSuite {
    #[serde(flatten)]
    pub suite: wycheproof::Suite,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<TestGroup>,
}

#[derive(Debug, Deserialize)]
pub struct TestGroup {
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[serde(rename = "keySize")]
    pub key_size: u32,
    #[serde(rename = "tagSize")]
    pub tag_size: u32,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
pub struct TestCase {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "wycheproof::hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub tag: Vec<u8>,
}

#[test]
fn test_aes_cmac() {
    let filename = "testvectors/aes_cmac_test.json";
    println!("wycheproof file '{}'", filename);
    let bytes = wycheproof::wycheproof_data(filename);
    let suite: TestSuite = serde_json::from_slice(&bytes).unwrap();

    for g in &suite.test_groups {
        println!(
            "   key info: key_size={}, tag_size={}",
            g.key_size, g.tag_size
        );
        for tc in &g.tests {
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );
            assert_eq!(tc.key.len() * 8, g.key_size as usize);
            assert_eq!(
                g.tag_size % 8,
                0,
                "Requested tag size for test case {} ({}) is not a multiple of 8, but {}",
                tc.case.case_id,
                tc.case.comment,
                g.tag_size
            );
            let output_length = g.tag_size as usize / 8; // in bytes

            let mac = match tc.key.len() {
                16 => {
                    let mut mac = Cmac::<Aes128>::new_varkey(&tc.key).unwrap();
                    mac.update(&tc.msg);
                    let result = mac.finalize_reset().into_bytes();
                    result[..min(result.len(), output_length)].to_vec()
                }
                24 => {
                    let mut mac = Cmac::<Aes192>::new_varkey(&tc.key).unwrap();
                    mac.update(&tc.msg);
                    let result = mac.finalize_reset().into_bytes();
                    result[..min(result.len(), output_length)].to_vec()
                }
                32 => {
                    let mut mac = Cmac::<Aes256>::new_varkey(&tc.key).unwrap();
                    mac.update(&tc.msg);
                    let result = mac.finalize_reset().into_bytes();
                    result[..min(result.len(), output_length)].to_vec()
                }
                _ => {
                    assert_eq!(tc.case.result, wycheproof::CaseResult::Invalid);
                    continue;
                }
            };
            if tc.case.result == wycheproof::CaseResult::Valid {
                assert_eq!(
                    mac, tc.tag,
                    "Could not verify MAC for test case {} ({})",
                    tc.case.case_id, tc.case.comment
                );
            } else {
                assert_ne!(
                    mac, tc.tag,
                    "Verified invalid MAC for test case {} ({})",
                    tc.case.case_id, tc.case.comment
                );
            }
        }
    }
}
