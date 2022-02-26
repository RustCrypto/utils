use crate::wycheproof;
use crate::wycheproof::{description, hex_string};
use crate::TestInfo;
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
    #[allow(dead_code)]
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[allow(dead_code)]
    #[serde(rename = "keySize")]
    pub key_size: u32,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "hex_string")]
    pub ikm: Vec<u8>,
    #[serde(with = "hex_string")]
    pub salt: Vec<u8>,
    #[serde(with = "hex_string")]
    pub info: Vec<u8>,
    pub size: usize,
    #[serde(with = "hex_string")]
    pub okm: Vec<u8>,
}

pub fn generator(data: &[u8], algorithm: &str, _key_size: u32) -> Vec<TestInfo> {
    let suite: TestSuite = serde_json::from_slice(data).unwrap();
    assert_eq!(algorithm, suite.suite.algorithm);

    let mut infos = vec![];
    for g in &suite.test_groups {
        for tc in &g.tests {
            if tc.case.result != crate::wycheproof::CaseResult::Valid {
                continue;
            }
            if tc.okm.len() != tc.size {
                eprintln!(
                    "Skipping case {} with size={} != okm.len()={}",
                    tc.case.case_id,
                    tc.size,
                    tc.okm.len()
                );
            }
            infos.push(TestInfo {
                data: vec![
                    tc.ikm.clone(),
                    tc.salt.clone(),
                    tc.info.clone(),
                    tc.okm.clone(),
                ],
                desc: description(&suite.suite, &tc.case),
            });
        }
    }
    infos
}
