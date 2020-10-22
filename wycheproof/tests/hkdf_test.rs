use hkdf::Hkdf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HkdfTestSuite {
    #[serde(flatten)]
    pub suite: wycheproof::Suite,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<HkdfTestGroup>,
}

#[derive(Debug, Deserialize)]
struct HkdfTestGroup {
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[serde(rename = "keySize")]
    pub key_size: u32,
    pub tests: Vec<HkdfTestCase>,
}

#[derive(Debug, Deserialize)]
struct HkdfTestCase {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "wycheproof::hex_string")]
    pub ikm: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub salt: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub info: Vec<u8>,
    pub size: usize,
    #[serde(with = "wycheproof::hex_string")]
    pub okm: Vec<u8>,
}

#[test]
fn test_hkdf() {
    test_hkdf_with::<sha1::Sha1>("sha1");
    test_hkdf_with::<sha2::Sha256>("sha256");
    test_hkdf_with::<sha2::Sha384>("sha384");
    test_hkdf_with::<sha2::Sha512>("sha512");
}

fn test_hkdf_with<D>(hash: &str)
where
    D: digest::Update + digest::BlockInput + digest::FixedOutput + digest::Reset + Default + Clone,
    D::BlockSize: generic_array::ArrayLength<u8>,
    D::OutputSize: generic_array::ArrayLength<u8>,
{
    let filename = format!("testvectors/hkdf_{}_test.json", hash);
    println!("wycheproof file '{}' hash {}", filename, hash);
    let bytes = wycheproof::wycheproof_data(&filename);
    let suite: HkdfTestSuite = serde_json::from_slice(&bytes).unwrap();

    for g in &suite.test_groups {
        println!("   key info: key_size={}", g.key_size);
        for tc in &g.tests {
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );
            assert_eq!(tc.ikm.len() * 8, g.key_size as usize);
            let valid = tc.case.result == wycheproof::CaseResult::Valid;

            let prk = Hkdf::<D>::new(Some(&tc.salt), &tc.ikm);

            let mut okm = vec![0; tc.size];
            if prk.expand(&tc.info, &mut okm).is_err() {
                assert!(
                    !valid,
                    "Could not compute HKDF {:?} PRF for test case {} ({})",
                    hash, tc.case.case_id, tc.case.comment
                );
                continue;
            }
            if valid {
                assert_eq!(
                    okm, tc.okm,
                    "Compute HKDF {:?} PRF and expected for test case {} ({}) do not match",
                    hash, tc.case.case_id, tc.case.comment
                );
            } else {
                assert_ne!(
                    okm, tc.okm,
                    "Compute HKDF {:?} PRF and invalid expected for test case {} ({}) match",
                    hash, tc.case.case_id, tc.case.comment
                );
            }
        }
    }
}
