//! Encrypted PKCS#8 private key tests.

#![cfg(feature = "pkcs5")]

use core::convert::TryFrom;
use hex_literal::hex;
use pkcs8::{pkcs5::pbes2, EncryptedPrivateKeyInfo};

/// Ed25519 PKCS#8 encrypted private key (PBES2 + AES-128-CBC + PBKDF2-SHA1) encoded as ASN.1 DER.
///
/// Generated using:
///
/// ```
/// $ openssl pkcs8 -v2 aes256-cbc -v2prf hmacWithSHA1 -topk8 -inform der -in ed25519-priv.der -outform der -out ed25519-encpriv-aes128-sha1.der
/// ```
const ED25519_DER_AES128_SHA1_EXAMPLE: &[u8] =
    include_bytes!("examples/ed25519-encpriv-aes128-sha1.der");

/// Ed25519 PKCS#8 encrypted private key (PBES2 + AES-256-CBC + PBKDF2-SHA256) encoded as ASN.1 DER.
///
/// Generated using:
///
/// ```
/// $ openssl pkcs8 -v2 aes256-cbc -v2prf hmacWithSHA256 -topk8 -inform der -in ed25519-priv.der -outform der -out ed25519-encpriv-aes256-sha256.der
/// ```
const ED25519_DER_AES256_SHA256_EXAMPLE: &[u8] =
    include_bytes!("examples/ed25519-encpriv-aes256-sha256.der");

/// Password used to encrypt the keys.
#[cfg(feature = "encryption")]
const PASSWORD: &[u8] = b"hunter42"; // Bad password; don't actually use outside tests!

#[test]
fn parse_ed25519_der_encpriv_aes128_sha1() {
    let pk = EncryptedPrivateKeyInfo::try_from(ED25519_DER_AES128_SHA1_EXAMPLE).unwrap();

    assert_eq!(
        pk.encryption_algorithm.oid(),
        "1.2.840.113549.1.5.13".parse().unwrap()
    ); // PBES2

    let pbes2_params = pk.encryption_algorithm.pbes2().unwrap();
    let pbkdf2_params = pbes2_params.kdf.pbkdf2().unwrap();

    assert_eq!(pbkdf2_params.salt, hex!("e8765e01e43b6bad"));
    assert_eq!(pbkdf2_params.iteration_count, 2048);
    assert_eq!(pbkdf2_params.key_length, None);
    assert_eq!(pbkdf2_params.prf, pbes2::Pbkdf2Prf::HmacWithSha1);

    match pbes2_params.encryption {
        pbes2::EncryptionScheme::Aes128Cbc { iv } => {
            assert_eq!(iv, &hex!("223080a71bcd2b9a256d876c924979d2"));
        }
        other => panic!("unexpected encryption scheme: {:?}", other),
    }

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/ed25519-encpriv-aes128-sha1.der
    assert_eq!(
        pk.encrypted_data,
        &hex!("4B4D091548EAC381EE7663B21234CD4FF3C9DF664D713394CACCEA7C9B982BD8F29910FABCA4BF7BE0431FAC5C4D657BE997C1F5BF40E2DA465AC1FCC2E30470")
    );
}

#[test]
fn parse_ed25519_der_encpriv_aes256_sha256() {
    let pk = EncryptedPrivateKeyInfo::try_from(ED25519_DER_AES256_SHA256_EXAMPLE).unwrap();

    assert_eq!(
        pk.encryption_algorithm.oid(),
        "1.2.840.113549.1.5.13".parse().unwrap()
    ); // PBES2

    let pbes2_params = pk.encryption_algorithm.pbes2().unwrap();
    let pbkdf2_params = pbes2_params.kdf.pbkdf2().unwrap();

    assert_eq!(pbkdf2_params.salt, hex!("79d982e70df91a88"));
    assert_eq!(pbkdf2_params.iteration_count, 2048);
    assert_eq!(pbkdf2_params.key_length, None);
    assert_eq!(pbkdf2_params.prf, pbes2::Pbkdf2Prf::HmacWithSha256);

    match pbes2_params.encryption {
        pbes2::EncryptionScheme::Aes256Cbc { iv } => {
            assert_eq!(iv, &hex!("b2d02d78b2efd9dff694cf8e0af40925"));
        }
        other => panic!("unexpected encryption scheme: {:?}", other),
    }

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/ed25519-encpriv-aes256-sha256.der
    assert_eq!(
        pk.encrypted_data,
        &hex!("D0CD6C770F4BB87176422305C17401809E226674CE74185D221BFDAA95069890C8882FCE02B05D41BCBF54B035595BCD4154B32593708469B86AACF8815A7B2B")
    );
}

#[cfg(feature = "encryption")]
#[test]
fn decrypt_ed25519_der_encpriv_aes256_sha256() {
    let enc_pk = EncryptedPrivateKeyInfo::try_from(ED25519_DER_AES256_SHA256_EXAMPLE).unwrap();
    let pk = enc_pk.decrypt(PASSWORD).unwrap();
    assert_eq!(pk.as_ref(), include_bytes!("examples/ed25519-priv.der"));
}
