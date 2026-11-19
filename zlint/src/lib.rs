//! Rust API for invoking the `zlint` X.509 certificate linter.

#![allow(clippy::std_instead_of_alloc)]

use serde::{
    Deserialize, Deserializer,
    de::{Error, MapAccess, Visitor},
};
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, Read, Write},
    process::{Command, Stdio},
};
use tempfile::tempdir;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    NotApplicable,
    NotEffective,
    Pass,
    Notice,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Status {
    fn is_successful(self) -> bool {
        !matches!(self, Status::Warn | Status::Error | Status::Fatal)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LintStatus {
    status: Status,
    details: Option<String>,
}

impl LintStatus {
    fn is_successful(&self) -> bool {
        self.status.is_successful()
    }
}

impl<'de> Deserialize<'de> for LintStatus {
    fn deserialize<D>(deserializer: D) -> Result<LintStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVisitor;

        impl<'de> Visitor<'de> for StatusVisitor {
            type Value = LintStatus;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ZLint result object")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut status_output = None;
                let mut details = None;

                while let Some((key, value)) = access.next_entry::<&str, &str>()? {
                    if key == "result" {
                        status_output = Some(match value {
                            "NA" => Status::NotApplicable,
                            "NE" => Status::NotEffective,
                            "pass" => Status::Pass,
                            "notice" => Status::Notice,
                            "fatal" => Status::Fatal,
                            "error" => Status::Error,
                            "warn" => Status::Warn,
                            "info" => Status::Info,
                            other => {
                                return Err(M::Error::custom(format!(
                                    "unsupported value: {other}",
                                )));
                            }
                        });
                    }
                    if key == "details" {
                        details = Some(value.to_string());
                    }
                }

                if let Some(status) = status_output {
                    Ok(LintStatus { status, details })
                } else {
                    Err(M::Error::custom("no 'result' field found"))
                }
            }
        }

        deserializer.deserialize_map(StatusVisitor)
    }
}

#[derive(Debug, Deserialize)]
struct LintResult(HashMap<String, LintStatus>);

impl LintResult {
    fn check_lints(&self, ignored: &[&str]) -> bool {
        let mut failed = HashMap::<String, LintStatus>::new();

        for (key, value) in &self.0 {
            if !value.is_successful() && !ignored.contains(&key.as_str()) {
                failed.insert(String::from(key), value.clone());
            }
        }

        eprintln!("failed lints: {failed:?}");

        failed.is_empty()
    }
}

const ZLINT_CONFIG: &str = "
[AppleRootStorePolicyConfig]

[CABFBaselineRequirementsConfig]

[CABFEVGuidelinesConfig]

[CommunityConfig]

[MozillaRootStorePolicyConfig]

[RFC5280Config]

[RFC5480Config]

[RFC5891Config]

[e_rsa_fermat_factorization]
Rounds = 100
";

/// Run the `zlint` executable on a PEM-encoded certificate.
///
/// # Panics
///
/// Panics if `ZLint` cannot run, its output cannot be read, or a lint reports
/// `warn`, `error`, or `fatal` and its name is absent from `ignored`.
#[track_caller]
pub fn check_certificate(pem: &[u8], ignored: &[&str]) {
    let tmp_dir = tempdir().expect("create tempdir");
    let config_path = tmp_dir.path().join("zlint_config.toml");
    let cert_path = tmp_dir.path().join("zlint_cert.pem");

    {
        let mut config_file = File::create(&config_path).expect("create config file");
        config_file
            .write_all(ZLINT_CONFIG.as_bytes())
            .expect("Create config file");
    }
    {
        let mut cert_file = File::create(&cert_path).expect("create pem file");
        cert_file.write_all(pem).expect("Create pem file");
    }

    let mut child = Command::new("zlint")
        .arg("-pretty")
        .arg("-config")
        .arg(&config_path)
        .arg(&cert_path)
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| match e.kind() {
            io::ErrorKind::NotFound => {
                panic!("error running 'zlint': command not found. Is it installed?")
            }
            _ => panic!("error running 'zlint': {e:?}"),
        });

    let mut stdout = child.stdout.take().expect("couldn't get stdout");

    let output_buf = {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).expect("read zlint output");
        buf
    };

    let exit_status = child.wait().expect("get zlint status");

    assert!(exit_status.success(), "zlint failed");

    let output: LintResult = serde_json::from_slice(&output_buf).expect("parse zlint output");

    assert!(output.check_lints(ignored));
}

#[test]
fn parse_zlint_output() {
    let demo_output = br#"
          {
            "e_algorithm_identifier_improper_encoding": {"result": "pass"},
            "e_basic_constraints_not_critical": {"result": "NA", "details": "foo"}
          }
        "#;

    let output: LintResult = serde_json::from_slice(demo_output).expect("parse output");

    assert_eq!(
        output.0.get("e_algorithm_identifier_improper_encoding"),
        Some(&LintStatus {
            status: Status::Pass,
            details: None
        })
    );
}

#[test]
fn fatal_lint_fails_unless_ignored() {
    let output: LintResult =
        serde_json::from_str(r#"{"example_lint":{"result":"fatal"}}"#).unwrap();

    assert!(!output.check_lints(&[]));
    assert!(output.check_lints(&["example_lint"]));
}
