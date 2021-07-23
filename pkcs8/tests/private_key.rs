//! PKCS#8 private key tests

use core::convert::TryFrom;
use hex_literal::hex;
use pkcs8::{PrivateKeyInfo, Version};

#[cfg(any(feature = "pem", feature = "std"))]
use pkcs8::PrivateKeyDocument;

/// Elliptic Curve (P-256) PKCS#8 private key encoded as ASN.1 DER
const EC_P256_DER_EXAMPLE: &[u8] = include_bytes!("examples/p256-priv.der");

/// Ed25519 PKCS#8 v1 private key encoded as ASN.1 DER
const ED25519_DER_V1_EXAMPLE: &[u8] = include_bytes!("examples/ed25519-priv-pkcs8v1.der");

/// Ed25519 PKCS#8 v2 private key + public key encoded as ASN.1 DER
const ED25519_DER_V2_EXAMPLE: &[u8] = include_bytes!("examples/ed25519-priv-pkcs8v2.der");

/// RSA-2048 PKCS#8 private key encoded as ASN.1 DER
const RSA_2048_DER_EXAMPLE: &[u8] = include_bytes!("examples/rsa2048-priv.der");

/// Elliptic Curve (P-256) PKCS#8 private key encoded as PEM
#[cfg(feature = "pem")]
const EC_P256_PEM_EXAMPLE: &str = include_str!("examples/p256-priv.pem");

/// Ed25519 PKCS#8 private key encoded as PEM
#[cfg(feature = "pem")]
const ED25519_PEM_V1_EXAMPLE: &str = include_str!("examples/ed25519-priv-pkcs8v1.pem");

/// RSA-2048 PKCS#8 private key encoded as PEM
#[cfg(feature = "pem")]
const RSA_2048_PEM_EXAMPLE: &str = include_str!("examples/rsa2048-priv.pem");

#[test]
fn decode_ec_p256_der() {
    let pk = PrivateKeyInfo::try_from(EC_P256_DER_EXAMPLE).unwrap();

    assert_eq!(pk.version(), Version::V1);
    assert_eq!(pk.algorithm.oid, "1.2.840.10045.2.1".parse().unwrap());

    assert_eq!(
        pk.algorithm.parameters.unwrap().oid().unwrap(),
        "1.2.840.10045.3.1.7".parse().unwrap()
    );

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/p256-priv.der
    assert_eq!(pk.private_key, &hex!("306B020101042069624171561A63340DE0E7D869F2A05492558E1A04868B6A9F854A866788188DA144034200041CACFFB55F2F2CEFD89D89EB374B2681152452802DEEA09916068137D839CF7FC481A44492304D7EF66AC117BEFE83A8D08F155F2B52F9F618DD447029048E0F")[..]);
}

#[test]
fn decode_ed25519_der_v1() {
    let pk = PrivateKeyInfo::try_from(ED25519_DER_V1_EXAMPLE).unwrap();
    assert_eq!(pk.version(), Version::V1);
    assert_eq!(pk.algorithm.oid, "1.3.101.112".parse().unwrap());
    assert_eq!(pk.algorithm.parameters, None);

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/ed25519-priv.der
    assert_eq!(
        pk.private_key,
        &hex!("042017ED9C73E9DB649EC189A612831C5FC570238207C1AA9DFBD2C53E3FF5E5EA85")[..]
    );
}

#[test]
fn decode_ed25519_der_v2() {
    const PRIV_KEY: [u8; 34] =
        hex!("04203A133DABADA2AA9CE54B0961CC3F1576B0943DC86EBF72A56E052C43F30FA3A5");
    const PUB_KEY: [u8; 32] =
        hex!("A3A7EAE3A8373830BC47E1167BC50E1DB551999651E0E2DC587623438EAC3F31");

    let pk = PrivateKeyInfo::try_from(ED25519_DER_V2_EXAMPLE).unwrap();
    assert_eq!(pk.version(), Version::V2);
    assert_eq!(pk.algorithm.oid, "1.3.101.112".parse().unwrap());
    assert_eq!(pk.algorithm.parameters, None);
    assert_eq!(pk.private_key, PRIV_KEY);
    assert_eq!(pk.public_key, Some(&PUB_KEY[..]));
}

#[test]
fn decode_rsa_2048_der() {
    let pk = PrivateKeyInfo::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    assert_eq!(pk.version(), Version::V1);
    assert_eq!(pk.algorithm.oid, "1.2.840.113549.1.1.1".parse().unwrap());
    assert!(pk.algorithm.parameters.unwrap().is_null());

    // Extracted with:
    // $ openssl asn1parse -inform der -in tests/examples/rsa2048-priv.der
    assert_eq!(pk.private_key, &hex!("308204A30201000282010100B6C42C515F10A6AAF282C63EDBE24243A170F3FA2633BD4833637F47CA4F6F36E03A5D29EFC3191AC80F390D874B39E30F414FCEC1FCA0ED81E547EDC2CD382C76F61C9018973DB9FA537972A7C701F6B77E0982DFC15FC01927EE5E7CD94B4F599FF07013A7C8281BDF22DCBC9AD7CABB7C4311C982F58EDB7213AD4558B332266D743AED8192D1884CADB8B14739A8DADA66DC970806D9C7AC450CB13D0D7C575FB198534FC61BC41BC0F0574E0E0130C7BBBFBDFDC9F6A6E2E3E2AFF1CBEAC89BA57884528D55CFB08327A1E8C89F4E003CF2888E933241D9D695BCBBACDC90B44E3E095FA37058EA25B13F5E295CBEAC6DE838AB8C50AF61E298975B872F0203010001028201007ECC8362C0EDB0741164215E22F74AB9D91BA06900700CF63690E5114D8EE6BDCFBB2E3F9614692A677A083F168A5E52E5968E6407B9D97C6E0E4064F82DA0B758A14F17B9B7D41F5F48E28D6551704F56E69E7AA9FA630FC76428C06D25E455DCFC55B7AC2B4F76643FDED3FE15FF78ABB27E65ACC4AAD0BDF6DB27EF60A6910C5C4A085ED43275AB19C1D997A32C6EFFCE7DF2D1935F6E601EEDE161A12B5CC27CA21F81D2C99C3D1EA08E90E3053AB09BEFA724DEF0D0C3A3C1E9740C0D9F76126A149EC0AA7D8078205484254D951DB07C4CF91FB6454C096588FD5924DBABEB359CA2025268D004F9D66EB3D6F7ADC1139BAD40F16DDE639E11647376C102818100DCC061242D4E92AFAEE72AC513CA65B9F77036F9BD7E0E6E61461A7EF7654225EC153C7E5C31A6157A6E5A13FF6E178E8758C1CB33D9D6BBE3179EF18998E422ECDCBED78F4ECFDBE5F4FCD8AEC2C9D0DC86473CA9BD16D9D238D21FB5DDEFBEB143CA61D0BD6AA8D91F33A097790E9640DBC91085DC5F26343BA3138F6B2D6702818100D3F314757E40E954836F92BE24236AF2F0DA04A34653C180AF67E960086D93FDE65CB23EFD9D09374762F5981E361849AF68CDD75394FF6A4E06EB69B209E4228DB2DFA70E40F7F9750A528176647B788D0E5777A2CB8B22E3CD267FF70B4F3B02D3AAFB0E18C590A564B03188B0AA5FC48156B07622214243BD1227EFA7F2F902818100CE68B7AC1B0D100D636E55488753C5C09843FDB390E2705DF7689457C9BD8D9765E30978617E2EFC8048F4C324206DB86087B654E97BB3D464E7EE3F8CD83FE10436F7DF18E9A963C4E64911D67EDE34042F2E26E3D3A1AD346ADAD6B9B7F67708CB094E62DEE9FF4D5D6669AF988AF2255D1CE8ED317C6A7D8691DA354D12DB02818025F6E5944220286B4DFBBF4235C0EE5843D2198091895120D6CA7B200B826D3ECE738E2E00498FAC0A2A6CA969C7F0C3CA1AB0BC40297132BE7538D7BEDF4CB0EFC6B98EF7DBA54F56AA99AABCE534C49C27947D4678C51C63C78C7CE1687231B4C8EB587AE6EF0480CBAF4FC0173CFD587A7E67AF515FB9B9DE75111839722902818031995406D406207CADEAEA35B38D040C5F8A9A1AE0827E9ED06B153D83B6821935B4B36A82BE9D56C791B58C27271A5793D53A1D657C08997960B1433E5171987F452F144A7C72306D63E1D3FFC0B71B75AB08F2E45A482E988451CBE478E12EB228D07456C924B66F6CED048D853F533E31A68614F1C3CE6D8EC9983CE72AF7")[..]);
}

#[test]
#[cfg(feature = "pem")]
fn decode_ec_p256_pem() {
    let pkcs8_doc: PrivateKeyDocument = EC_P256_PEM_EXAMPLE.parse().unwrap();
    assert_eq!(pkcs8_doc.as_ref(), EC_P256_DER_EXAMPLE);

    // Ensure `PrivateKeyDocument` parses successfully
    let pk_info = PrivateKeyInfo::try_from(EC_P256_DER_EXAMPLE).unwrap();
    assert_eq!(pkcs8_doc.private_key_info().algorithm, pk_info.algorithm);
}

#[test]
#[cfg(feature = "pem")]
fn decode_ed25519_pem() {
    let pkcs8_doc: PrivateKeyDocument = ED25519_PEM_V1_EXAMPLE.parse().unwrap();
    assert_eq!(pkcs8_doc.as_ref(), ED25519_DER_V1_EXAMPLE);

    // Ensure `PrivateKeyDocument` parses successfully
    let pk_info = PrivateKeyInfo::try_from(ED25519_DER_V1_EXAMPLE).unwrap();
    assert_eq!(pkcs8_doc.private_key_info().algorithm, pk_info.algorithm);
}

#[test]
#[cfg(feature = "pem")]
fn decode_rsa_2048_pem() {
    let pkcs8_doc: PrivateKeyDocument = RSA_2048_PEM_EXAMPLE.parse().unwrap();
    assert_eq!(pkcs8_doc.as_ref(), RSA_2048_DER_EXAMPLE);

    // Ensure `PrivateKeyDocument` parses successfully
    let pk_info = PrivateKeyInfo::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    assert_eq!(pkcs8_doc.private_key_info().algorithm, pk_info.algorithm);
}

#[test]
#[cfg(feature = "alloc")]
fn encode_ec_p256_der() {
    let pk = PrivateKeyInfo::try_from(EC_P256_DER_EXAMPLE).unwrap();
    let pk_encoded = pk.to_der();
    assert_eq!(EC_P256_DER_EXAMPLE, pk_encoded.as_ref());
}

#[test]
#[cfg(feature = "alloc")]
fn encode_ed25519_der_v1() {
    let pk = PrivateKeyInfo::try_from(ED25519_DER_V1_EXAMPLE).unwrap();
    assert_eq!(ED25519_DER_V1_EXAMPLE, pk.to_der().as_ref());
}

#[test]
#[cfg(feature = "alloc")]
fn encode_ed25519_der_v2() {
    let pk = PrivateKeyInfo::try_from(ED25519_DER_V2_EXAMPLE).unwrap();
    assert_eq!(ED25519_DER_V2_EXAMPLE, pk.to_der().as_ref());
}

#[test]
#[cfg(feature = "alloc")]
fn encode_rsa_2048_der() {
    let pk = PrivateKeyInfo::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    assert_eq!(RSA_2048_DER_EXAMPLE, pk.to_der().as_ref());
}

#[test]
#[cfg(feature = "pem")]
fn encode_ec_p256_pem() {
    let pk = PrivateKeyInfo::try_from(EC_P256_DER_EXAMPLE).unwrap();
    assert_eq!(EC_P256_PEM_EXAMPLE, &*pk.to_pem());
}

#[test]
#[cfg(feature = "pem")]
fn encode_ed25519_pem() {
    let pk = PrivateKeyInfo::try_from(ED25519_DER_V1_EXAMPLE).unwrap();
    assert_eq!(ED25519_PEM_V1_EXAMPLE, &*pk.to_pem());
}

#[test]
#[cfg(feature = "pem")]
fn encode_rsa_2048_pem() {
    let pk = PrivateKeyInfo::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    assert_eq!(RSA_2048_PEM_EXAMPLE, &*pk.to_pem());
}

#[test]
#[cfg(feature = "std")]
fn read_der_file() {
    let pkcs8_doc = PrivateKeyDocument::read_der_file("tests/examples/p256-priv.der").unwrap();
    assert_eq!(pkcs8_doc.as_ref(), EC_P256_DER_EXAMPLE);
}

#[test]
#[cfg(all(feature = "pem", feature = "std"))]
fn read_pem_file() {
    let pkcs8_doc = PrivateKeyDocument::read_pem_file("tests/examples/p256-priv.pem").unwrap();
    assert_eq!(pkcs8_doc.as_ref(), EC_P256_DER_EXAMPLE);
}
