use crate::wycheproof;
use crate::wycheproof::{description, hex_string, CaseResult};
use crate::TestInfo;
use serde::Deserialize;

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
    #[serde(with = "hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "hex_string")]
    pub tag: Vec<u8>,
}

pub fn generator(data: &[u8], algorithm: &str, key_size: u32) -> Vec<TestInfo> {
    let suite: TestSuite = serde_json::from_slice(data).unwrap();
    assert_eq!(algorithm, suite.suite.algorithm);

    let mut infos = vec![];
    for g in &suite.test_groups {
        for tc in &g.tests {
            if key_size != 0 && g.key_size != key_size {
                continue;
            }
            if tc.case.result != CaseResult::Valid {
                // TODO: adapt HMAC tests for invalid cases
                continue;
            }
            assert_eq!(tc.key.len() * 8, g.key_size as usize);
            assert_eq!(g.tag_size % 8, 0);
            infos.push(TestInfo {
                data: vec![
                    tc.key.clone(),
                    tc.msg.clone(),
                    // `tc.tag` holds the MAC output truncated to `(g.tag_size / 8)` bytes
                    // (when this is smaller than MAC output size)
                    tc.tag.clone(),
                ],
                desc: description(&suite.suite, &tc.case),
            });
        }
    }
    infos
}
