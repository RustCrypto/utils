//! Encrypted PKCS#8 private key tests.

#![cfg(feature = "pkcs5")]

use core::convert::TryFrom;
use hex_literal::hex;
use pkcs8::EncryptedPrivateKeyInfo;

/// Ed25519 PKCS#8 encrypted private key encoded as ASN.1 DER.
///
/// Generated using:
///
/// ```
/// $ openssl pkcs8 -v2 aes-256-cbc -v2prf hmacWithSHA256 -topk8 -inform der -in ed25519-priv.der -outform der -out ed25519-priv-enc-v2.der
/// ```
const ED25519_DER_EXAMPLE: &[u8] = include_bytes!("examples/ed25519-priv-enc-v2.der");

/// Ed25519 PKCS#8 encrypted private key encoded as PEM.
///
/// Generated using:
///
/// ```
/// $ openssl pkcs8 -v2 aes-256-cbc -v2prf hmacWithSHA256 -topk8 -in ed25519-priv.pem -out ed25519-priv-enc-v2.pem
/// ```
#[cfg(feature = "pem")]
#[allow(dead_code)] // TODO(tarcieri): support for PEM-encoded `EncryptedPrivateKeyInfo`
const ED25519_PEM_EXAMPLE: &str = include_str!("examples/ed25519-priv-enc-v2.pem");

/// Password used to encrypt the keys.
#[allow(dead_code)] // TODO(tarcieri): decryption support
const PASSWORD: &[u8] = b"hunter42"; // Bad password; don't actually use outside tests!

#[test]
fn parse_ed25519_der_encrypted() {
    let pk = EncryptedPrivateKeyInfo::try_from(ED25519_DER_EXAMPLE).unwrap();

    assert_eq!(
        pk.encryption_algorithm.oid,
        "1.2.840.113549.1.5.13".parse().unwrap()
    ); // PBES2

    // TODO(tarcieri): parse/extract params
    let params = pk.encryption_algorithm.parameters_any().unwrap();
    assert_eq!(params.tag(), der::Tag::Sequence);

    // Extracted with:
    // $  openssl asn1parse -in tests/examples/ed25519-priv-enc-v2.der -inform der
    assert_eq!(
        pk.encrypted_data,
        &hex!("D0CD6C770F4BB87176422305C17401809E226674CE74185D221BFDAA95069890C8882FCE02B05D41BCBF54B035595BCD4154B32593708469B86AACF8815A7B2B")
    );
}
