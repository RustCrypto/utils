//! Encrypted PKCS#8 private key tests.

#![cfg(feature = "pkcs5")]

use core::convert::TryFrom;
use hex_literal::hex;
use pkcs8::EncryptedPrivateKeyInfo;

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
#[allow(dead_code)] // TODO(tarcieri): decryption support
const PASSWORD: &[u8] = b"hunter42"; // Bad password; don't actually use outside tests!

#[test]
fn parse_ed25519_der_encrypted_aes128_sha1() {
    let pk = EncryptedPrivateKeyInfo::try_from(ED25519_DER_AES128_SHA1_EXAMPLE).unwrap();

    assert_eq!(
        pk.encryption_algorithm.oid,
        "1.2.840.113549.1.5.13".parse().unwrap()
    ); // PBES2

    // TODO(tarcieri): parse/extract params
    let params = pk.encryption_algorithm.parameters_any().unwrap();
    assert_eq!(params.tag(), der::Tag::Sequence);

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/ed25519-encpriv-aes128-sha1.der
    assert_eq!(
        pk.encrypted_data,
        &hex!("4B4D091548EAC381EE7663B21234CD4FF3C9DF664D713394CACCEA7C9B982BD8F29910FABCA4BF7BE0431FAC5C4D657BE997C1F5BF40E2DA465AC1FCC2E30470")
    );
}

#[test]
fn parse_ed25519_der_encrypted_aes256_sha256() {
    let pk = EncryptedPrivateKeyInfo::try_from(ED25519_DER_AES256_SHA256_EXAMPLE).unwrap();

    assert_eq!(
        pk.encryption_algorithm.oid,
        "1.2.840.113549.1.5.13".parse().unwrap()
    ); // PBES2

    // TODO(tarcieri): parse/extract params
    let params = pk.encryption_algorithm.parameters_any().unwrap();
    assert_eq!(params.tag(), der::Tag::Sequence);

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/ed25519-encpriv-aes256-sha256.der
    assert_eq!(
        pk.encrypted_data,
        &hex!("D0CD6C770F4BB87176422305C17401809E226674CE74185D221BFDAA95069890C8882FCE02B05D41BCBF54B035595BCD4154B32593708469B86AACF8815A7B2B")
    );
}
