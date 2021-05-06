use crate::wycheproof;
use crate::wycheproof::{case_result, description, hex_string};
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
    #[serde(with = "hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "hex_string")]
    pub aad: Vec<u8>,
    #[serde(with = "hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "hex_string")]
    pub ct: Vec<u8>,
}

pub fn generator(data: &[u8], algorithm: &str, key_size: u32) -> Vec<TestInfo> {
    let suite: TestSuite = serde_json::from_slice(data).unwrap();
    assert_eq!(algorithm, suite.suite.algorithm);

    let mut infos = vec![];
    for g in &suite.test_groups {
        if key_size != 0 && g.key_size != key_size {
            continue;
        }
        for tc in &g.tests {
            infos.push(TestInfo {
                data: vec![
                    tc.key.clone(),
                    tc.aad.clone(),
                    tc.msg.clone(),
                    tc.ct.clone(),
                    vec![case_result(&tc.case)],
                ],
                desc: description(&suite.suite, &tc.case),
            });
        }
    }
    infos
}
