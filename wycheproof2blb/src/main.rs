//! Tool to convert Wycheproof test vectors to raw hex format
use std::io::Write;

mod aead;
mod aes_siv;
mod ecdsa;
mod ed25519;
mod hkdf;
mod mac;
mod wycheproof;

/// Test information
pub struct TestInfo {
    /// Raw data for the tests.
    pub data: Vec<Vec<u8>>,
    /// Test case description.
    pub desc: String,
}

/// Generator function which takes input parameters:
///  - contents of Wycheproof test data file
///  - algorithm name
///  - key size (in bits) to include
/// and returns the raw contents, together  with a list of test identifiers (one per entry).
type BlbGenerator = fn(&[u8], &str, u32) -> Vec<TestInfo>;

struct Algorithm {
    pub file: &'static str,
    pub generator: BlbGenerator,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wycheproof_dir = args
        .get(1)
        .expect("Provide directory with wycheproof vectors");
    let algorithm = args.get(2).expect("Provide algorithm family");
    let key_size = args
        .get(3)
        .expect("Provide key size in bits, or 0 for all sizes")
        .parse::<u32>()
        .expect("Key size needs to be a number of bits");
    let out_path = args.get(4).expect("Provide path for output blobby file");
    let descriptions_path = args.get(5).expect("Provide path for descriptions file");

    let algo = match algorithm.as_str() {
        "AES-GCM" => Algorithm {
            file: "aes_gcm_test.json",
            generator: aead::aes_gcm_generator,
        },
        "AES-GCM-SIV" => Algorithm {
            file: "aes_gcm_siv_test.json",
            generator: aead::aes_gcm_generator,
        },
        "CHACHA20-POLY1305" => Algorithm {
            file: "chacha20_poly1305_test.json",
            generator: aead::chacha20_poly1305,
        },
        "XCHACHA20-POLY1305" => Algorithm {
            file: "xchacha20_poly1305_test.json",
            generator: aead::xchacha20_poly1305,
        },
        "AES-SIV-CMAC" => Algorithm {
            file: "aes_siv_cmac_test.json",
            generator: aes_siv::generator,
        },
        "AES-CMAC" => Algorithm {
            file: "aes_cmac_test.json",
            generator: mac::generator,
        },
        "HKDF-SHA-1" => Algorithm {
            file: "hkdf_sha1_test.json",
            generator: hkdf::generator,
        },
        "HKDF-SHA-256" => Algorithm {
            file: "hkdf_sha256_test.json",
            generator: hkdf::generator,
        },
        "HKDF-SHA-384" => Algorithm {
            file: "hkdf_sha384_test.json",
            generator: hkdf::generator,
        },
        "HKDF-SHA-512" => Algorithm {
            file: "hkdf_sha512_test.json",
            generator: hkdf::generator,
        },
        "HMACSHA1" => Algorithm {
            file: "hmac_sha1_test.json",
            generator: mac::generator,
        },
        "HMACSHA224" => Algorithm {
            file: "hmac_sha224_test.json",
            generator: mac::generator,
        },
        "HMACSHA256" => Algorithm {
            file: "hmac_sha256_test.json",
            generator: mac::generator,
        },
        "HMACSHA384" => Algorithm {
            file: "hmac_sha384_test.json",
            generator: mac::generator,
        },
        "HMACSHA512" => Algorithm {
            file: "hmac_sha512_test.json",
            generator: mac::generator,
        },
        "EDDSA" => Algorithm {
            file: "eddsa_test.json",
            generator: ed25519::generator,
        },
        "secp256r1" => Algorithm {
            file: "ecdsa_secp256r1_sha256_test.json",
            generator: ecdsa::generator,
        },
        // There's also "ecdsa_secp256r1_sha256_p1363_test.json" with a different signature encoding.
        "secp256k1" => Algorithm {
            file: "ecdsa_secp256k1_sha256_test.json",
            generator: ecdsa::generator,
        },
        _ => panic!("Unrecognized algorithm '{}'", algorithm),
    };

    let data = wycheproof::data(wycheproof_dir, algo.file);

    let infos = (algo.generator)(&data, algorithm, key_size);
    println!("Emitting {} test cases", infos.len());

    let mut txt_file = std::fs::File::create(descriptions_path).unwrap();
    for info in &infos {
        writeln!(&mut txt_file, "{}", info.desc).unwrap();
    }

    let mut out_file = std::fs::File::create(out_path).unwrap();
    let blobs: Vec<Vec<u8>> = infos.into_iter().map(|info| info.data).flatten().collect();
    let (blb_data, _) = blobby::encode_blobs(&blobs);
    out_file.write_all(&blb_data).unwrap();
}
