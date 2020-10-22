use aes::{Aes128, Aes192, Aes256};
use cmac::{Cmac, Mac, NewMac};
use hmac::Hmac;
use serde::Deserialize;
use std::cmp::min;

#[derive(Debug, Deserialize)]
pub struct PrfTestSuite {
    #[serde(flatten)]
    pub suite: wycheproof::Suite,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<PrfTestGroup>,
}

#[derive(Debug, Deserialize)]
pub struct PrfTestGroup {
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[serde(rename = "keySize")]
    pub key_size: u32,
    #[serde(rename = "tagSize")]
    pub tag_size: u32,
    pub tests: Vec<PrfTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct PrfTestCase {
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
    let suite: PrfTestSuite = serde_json::from_slice(&bytes).unwrap();

    for g in &suite.test_groups {
        println!("   key info: key_size={}", g.key_size);
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
            let output_length = (g.tag_size / 8) as usize;
            let valid = tc.case.result == wycheproof::CaseResult::Valid;

            let result = match tc.key.len() {
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
                    assert!(!valid);
                    continue;
                }
            };

            if valid {
                assert_eq!(
                    result, tc.tag,
                    "Computed AES-CMAC and expected for test case {} ({}) do not match",
                    tc.case.case_id, tc.case.comment
                );
            } else {
                assert_ne!(
                    result, tc.tag,
                    "Computed AES-CMAC and invalid expected for test case {} ({}) match",
                    tc.case.case_id, tc.case.comment
                )
            }
        }
    }
}

#[test]
fn test_hmac() {
    test_hmac_with::<Hmac<sha1::Sha1>>("sha1");
    test_hmac_with::<Hmac<sha2::Sha256>>("sha256");
    test_hmac_with::<Hmac<sha2::Sha384>>("sha384");
    test_hmac_with::<Hmac<sha2::Sha512>>("sha512");
}

fn test_hmac_with<T>(hash: &str)
where
    T: hmac::NewMac + hmac::Mac,
{
    let filename = format!("testvectors/hmac_{}_test.json", hash);
    println!("wycheproof file '{}' hash {}", filename, hash);
    let bytes = wycheproof::wycheproof_data(&filename);
    let suite: PrfTestSuite = serde_json::from_slice(&bytes).unwrap();

    for g in &suite.test_groups {
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
            let output_length = (g.tag_size / 8) as usize;
            let valid = tc.case.result == wycheproof::CaseResult::Valid;

            let mut mac = <T>::new_varkey(&tc.key).unwrap();
            mac.update(&tc.msg);
            let result = mac.finalize_reset().into_bytes();
            let res = result[..min(result.len(), output_length)].to_vec();

            if valid {
                assert_eq!(
                    res, tc.tag,
                    "Computed HMAC and expected for test case {} ({}) do not match",
                    tc.case.case_id, tc.case.comment
                );
            } else {
                assert_ne!(
                    res, tc.tag,
                    "Computed HMAC and invalid expected for test case {} ({}) match",
                    tc.case.case_id, tc.case.comment
                )
            }
        }
    }
}
