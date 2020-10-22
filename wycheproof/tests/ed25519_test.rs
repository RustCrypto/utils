use ed25519_dalek::Signer;
use serde::Deserialize;
use signature::{Signature, Verifier};

#[derive(Debug, Deserialize)]
struct TestSuiteEd25519 {
    #[serde(flatten)]
    pub suite: wycheproof::Suite,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<TestGroupEd25519>,
}

#[derive(Debug, Deserialize)]
struct TestGroupEd25519 {
    #[serde(flatten)]
    pub group: wycheproof::Group,
    #[serde(rename = "keyDer")]
    pub key_der: String,
    #[serde(rename = "keyPem")]
    pub key_pem: String,
    pub key: TestKeyEd25519,
    pub tests: Vec<TestCaseEd25519>,
}

#[derive(Debug, Deserialize)]
struct TestKeyEd25519 {
    #[serde(with = "wycheproof::hex_string")]
    sk: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pk: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct TestCaseEd25519 {
    #[serde(flatten)]
    pub case: wycheproof::Case,
    #[serde(with = "wycheproof::hex_string")]
    pub msg: Vec<u8>,
    #[serde(with = "wycheproof::hex_string")]
    pub sig: Vec<u8>,
}

#[test]
fn test_ed25519() {
    let filename = "testvectors/eddsa_test.json";
    println!("wycheproof file '{}'", filename);
    let bytes = wycheproof::wycheproof_data(filename);
    let suite: TestSuiteEd25519 = serde_json::from_slice(&bytes).unwrap();
    for g in &suite.test_groups {
        println!(
            "   key info: sk={}, pk={}",
            hex::encode(&g.key.sk),
            hex::encode(&g.key.pk)
        );

        let secret_key = ed25519_dalek::SecretKey::from_bytes(&g.key.sk).unwrap();
        let public_key: ed25519_dalek::PublicKey = (&secret_key).into();
        let keypair = ed25519_dalek::Keypair {
            secret: secret_key,
            public: public_key,
        };

        let public_key = ed25519_dalek::PublicKey::from_bytes(&g.key.pk).unwrap();
        assert_eq!(public_key, keypair.public);

        for tc in &g.tests {
            println!(
                "     case {} [{}] {}",
                tc.case.case_id, tc.case.result, tc.case.comment
            );

            let got = keypair.sign(&tc.msg).as_bytes().to_vec();
            if tc.case.result == wycheproof::CaseResult::Valid {
                // Ed25519 is deterministic.
                assert_eq!(
                    tc.sig,
                    got,
                    "sign failed in test case {}: invalid signature generated {}",
                    tc.case.case_id,
                    hex::encode(&got)
                );
            }

            let s = match ed25519_dalek::Signature::from_bytes(&tc.sig) {
                Ok(s) => s,
                Err(_) => {
                    assert_eq!(tc.case.result, wycheproof::CaseResult::Invalid);
                    continue;
                }
            };
            let result = public_key.verify(&tc.msg, &s);
            match tc.case.result {
                wycheproof::CaseResult::Valid => assert!(
                    result.is_ok(),
                    "verify failed in test case {}: valid signature rejected with {:?}",
                    tc.case.case_id,
                    result
                ),
                wycheproof::CaseResult::Invalid => assert!(
                    result.is_err(),
                    "verify failed in test case {}: invalid signature is accepted",
                    tc.case.case_id
                ),
                wycheproof::CaseResult::Acceptable => unimplemented!(),
            }
        }
    }
}
