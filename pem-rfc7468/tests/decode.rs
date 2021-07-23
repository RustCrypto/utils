//! PEM decoding tests

#[test]
fn pkcs1_example() {
    let pem = include_bytes!("examples/pkcs1.pem");
    let mut buf = [0u8; 2048];
    let (label, decoded) = pem_rfc7468::decode(pem, &mut buf).unwrap();
    assert_eq!(label, "RSA PRIVATE KEY");
    assert_eq!(decoded, include_bytes!("examples/pkcs1.der"));
}

#[test]
fn pkcs8_example() {
    let pem = include_bytes!("examples/pkcs8.pem");
    let mut buf = [0u8; 2048];
    let (label, decoded) = pem_rfc7468::decode(pem, &mut buf).unwrap();
    assert_eq!(label, "PRIVATE KEY");
    assert_eq!(decoded, include_bytes!("examples/pkcs8.der"));
}

#[test]
fn pkcs8_enc_example() {
    let pem = include_bytes!("examples/pkcs8-enc.pem");
    let mut buf = [0u8; 2048];
    let (label, decoded) = pem_rfc7468::decode(pem, &mut buf).unwrap();
    assert_eq!(label, "ENCRYPTED PRIVATE KEY");
    assert_eq!(decoded, include_bytes!("examples/pkcs8-enc.der"));
}

#[test]
#[cfg(feature = "alloc")]
fn pkcs1_example_with_vec() {
    let pem = include_bytes!("examples/pkcs1.pem");
    let (label, decoded) = pem_rfc7468::decode_vec(pem).unwrap();
    assert_eq!(label, "RSA PRIVATE KEY");
    assert_eq!(decoded, include_bytes!("examples/pkcs1.der"));
}

#[test]
#[cfg(feature = "alloc")]
fn pkcs8_enc_example_with_vec() {
    let pem = include_bytes!("examples/pkcs8-enc.pem");
    let (label, decoded) = pem_rfc7468::decode_vec(pem).unwrap();
    assert_eq!(label, "ENCRYPTED PRIVATE KEY");
    assert_eq!(decoded, include_bytes!("examples/pkcs8-enc.der"));
}
