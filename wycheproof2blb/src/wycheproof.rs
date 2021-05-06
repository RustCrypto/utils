//! Helpers for retrieving Wycheproof test vectors.

use serde::Deserialize;

/// `Suite` represents the common elements of the top level object in a Wycheproof json
/// file.  Implementations should embed (using `#[serde(flatten)]`) `Suite` in a struct
/// that strongly types the `testGroups` field.
#[derive(Debug, Deserialize)]
pub struct Suite {
    pub algorithm: String,
    #[serde(rename = "generatorVersion")]
    pub generator_version: String,
    #[serde(rename = "numberOfTests")]
    pub number_of_tests: i32,
    pub notes: std::collections::HashMap<String, String>,
}

/// `Group` represents the common elements of a testGroups object in a Wycheproof suite.
/// Implementations should embed (using `#[serde(flatten)]`) Group in a struct that
/// strongly types its list of cases.
#[derive(Debug, Deserialize)]
pub struct Group {
    #[serde(rename = "type")]
    pub group_type: String,
}

/// `Result` represents the possible result values for a Wycheproof test case.
#[derive(Debug, PartialEq, Eq)]
pub enum CaseResult {
    /// Test case is valid, the crypto operation should succeed.
    Valid,
    /// Test case is invalid; the crypto operation should fail.
    Invalid,
    /// Test case is valid, but uses weak parameters; the crypto operation might succeed
    /// or fail depending on how strict the library is.
    Acceptable,
}

impl std::fmt::Display for CaseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CaseResult::Valid => "valid",
                CaseResult::Invalid => "invalid",
                CaseResult::Acceptable => "acceptable",
            }
        )
    }
}

/// `Case` represents the common elements of a tests object in a Wycheproof group.
/// Implementations should embed (using `#[serde(flatten)]`) `Case` in a struct that
/// contains fields specific to the test type.
#[derive(Debug, Deserialize)]
pub struct Case {
    #[serde(rename = "tcId")]
    pub case_id: i32,
    pub comment: String,
    #[serde(with = "case_result")]
    pub result: CaseResult,
    #[serde(default)]
    pub flags: Vec<String>,
}

pub mod hex_string {
    //! Manual JSON deserialization implementation for hex strings.
    use serde::Deserialize;
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        ::hex::decode(&s).map_err(|_e| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"hex data expected")
        })
    }
}

pub mod case_result {
    //! Manual JSON deserialization for a `result` enum.
    use serde::Deserialize;
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<super::CaseResult, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_ref() {
            "valid" => Ok(super::CaseResult::Valid),
            "invalid" => Ok(super::CaseResult::Invalid),
            "acceptable" => Ok(super::CaseResult::Acceptable),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"unexpected result value",
            )),
        }
    }
}

/// Convert a `result` enum to a byte.
pub fn case_result(case: &Case) -> u8 {
    match case.result {
        CaseResult::Invalid => 0u8,
        CaseResult::Valid => 1u8,
        _ => panic!("Unexpected case result {}", case.result),
    }
}

/// Retrieve Wycheproof test vectors from the given filename in a Wycheproof repo.
pub fn data(wycheproof_dir: &str, filename: &str) -> Vec<u8> {
    let path = std::path::Path::new(&wycheproof_dir)
        .join("testvectors")
        .join(filename);
    std::fs::read(&path)
        .unwrap_or_else(|_| panic!("Test vector file {} not found at {:?}", filename, path))
}

/// Build a description for a test case in a suite
pub fn description(suite: &Suite, case: &Case) -> String {
    format!(
        "{} case {} [{}] {}",
        suite.algorithm, case.case_id, case.result, case.comment
    )
}
