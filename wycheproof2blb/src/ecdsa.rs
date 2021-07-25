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

pub fn generator(data: &[u8], algorithm: &str, _key_size: u32) -> Vec<TestInfo> {
    let suite: TestSuite = serde_json::from_slice(data).unwrap();

    let mut infos = vec![];
    for g in &suite.test_groups {
        assert_eq!(g.key.curve, algorithm);
        assert_eq!(g.sha, "SHA-256");
        for tc in &g.tests {
            if tc.case.result == crate::wycheproof::CaseResult::Acceptable {
                // TODO: figure out what to do with test cases that pass but which have weak params
                continue;
            }
            infos.push(TestInfo {
                data: vec![
                    g.key.wx.clone(),
                    g.key.wy.clone(),
                    tc.msg.clone(),
                    tc.sig.clone(),
                    vec![case_result(&tc.case)],
                ],
                desc: description(&suite.suite, &tc.case),
            });
        }
    }
    infos
}
