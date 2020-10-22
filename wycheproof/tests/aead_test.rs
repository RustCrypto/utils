use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead, Payload};
use serde::Deserialize;
use wycheproof::hex_string;

#[derive(Debug, Deserialize)]
pub struct AeadTestSuite {
    #[serde(flatten)]
    pub suite: wycheproof::Suite,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<AeadTestGroup>,
}

#[derive(Debug, Deserialize)]
pub struct AeadTestGroup {
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[serde(rename = "ivSize")]
    pub iv_size: u32,
    #[serde(rename = "keySize")]
    pub key_size: u32,
    #[serde(rename = "tagSize")]
    pub tag_size: u32,
    pub tests: Vec<AeadTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct AeadTestCase {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "hex_string")]
    pub aad: Vec<u8>,
    #[serde(with = "hex_string")]
    pub ct: Vec<u8>,
    #[serde(with = "hex_string")]
    pub iv: Vec<u8>,
    #[serde(with = "hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "hex_string")]
    pub tag: Vec<u8>,
}

const AES_GCM_IV_SIZE: u32 = 12;

#[test]
fn test_aes_gcm() {
    let filename = "testvectors/aes_gcm_test.json";
    println!("wycheproof file '{}'", filename);
    let bytes = wycheproof::wycheproof_data(filename);
    let suite: AeadTestSuite = serde_json::from_slice(&bytes).unwrap();
    assert_eq!("AES-GCM", suite.suite.algorithm);

    for g in &suite.test_groups {
        let key_size = g.key_size / 8;
        if key_size != 16 && key_size != 32 {
            println!("   skipping tests for key_size={}", g.key_size);
            continue;
        }
        if g.iv_size != AES_GCM_IV_SIZE * 8 {
            println!("   skipping tests for iv_size={}", g.iv_size);
            continue;
        }

        for tc in &g.tests {
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );
            let mut combined_ct = Vec::new();
            combined_ct.extend_from_slice(&tc.ct);
            combined_ct.extend_from_slice(&tc.tag);

            // create cipher and do decryption
            let payload = Payload {
                msg: &combined_ct,
                aad: &tc.aad,
            };
            let result = match key_size {
                16 => {
                    let cipher = aes_gcm::Aes128Gcm::new(GenericArray::from_slice(&tc.key));
                    cipher.decrypt(GenericArray::from_slice(&tc.iv), payload)
                }
                32 => {
                    let cipher = aes_gcm::Aes256Gcm::new(GenericArray::from_slice(&tc.key));
                    cipher.decrypt(GenericArray::from_slice(&tc.iv), payload)
                }
                _ => unreachable!(),
            };

            match result {
                Err(e) => {
                    assert_eq!(
                        tc.case.result,
                        wycheproof::CaseResult::Invalid,
                        "unexpected error in test case {}: {}",
                        tc.case.case_id,
                        e
                    );
                }
                Ok(decrypted) => {
                    assert_eq!(
                        tc.case.result,
                        wycheproof::CaseResult::Valid,
                        "decrypted invalid test case {}",
                        tc.case.case_id
                    );
                    assert_eq!(
                        decrypted, tc.msg,
                        "incorrect decryption in test case {}",
                        tc.case.case_id,
                    );
                }
            }
        }
    }
}

const CHA_CHA20_KEY_SIZE: u32 = 32;
const CHA_CHA20_NONCE_SIZE: u32 = 12;

#[test]
fn test_cha_cha20_poly1305() {
    let filename = "testvectors/chacha20_poly1305_test.json";
    println!("wycheproof file '{}'", filename);
    let bytes = wycheproof::wycheproof_data(filename);
    let suite: AeadTestSuite = serde_json::from_slice(&bytes).unwrap();

    for g in &suite.test_groups {
        if (g.key_size / 8) != CHA_CHA20_KEY_SIZE {
            println!(" skipping tests for key_size={}", g.key_size);
            continue;
        }
        if (g.iv_size / 8) != CHA_CHA20_NONCE_SIZE {
            println!(" skipping tests for iv_size={}", g.iv_size);
            continue;
        }
        for tc in &g.tests {
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );
            let mut combined_ct = Vec::new();
            combined_ct.extend_from_slice(&tc.ct);
            combined_ct.extend_from_slice(&tc.tag);

            let key = chacha20poly1305::Key::clone_from_slice(&tc.key);
            let cipher = chacha20poly1305::ChaCha20Poly1305::new(&key);

            let iv = chacha20poly1305::Nonce::from_slice(&tc.iv);
            let result = cipher.decrypt(
                iv,
                Payload {
                    msg: &combined_ct,
                    aad: &tc.aad,
                },
            );

            match result {
                Err(e) => {
                    assert_ne!(
                        tc.case.result,
                        wycheproof::CaseResult::Valid,
                        "#{}, unexpected error: {}",
                        tc.case.case_id,
                        e
                    );
                }
                Ok(decrypted) => {
                    assert_ne!(
                        tc.case.result,
                        wycheproof::CaseResult::Invalid,
                        "#{}, decrypted invalid",
                        tc.case.case_id
                    );
                    assert_eq!(
                        decrypted, tc.msg,
                        "#{}, incorrect decryption",
                        tc.case.case_id
                    );
                }
            }
        }
    }
}
