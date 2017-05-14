//! Generic implementation of Hash-based Message Authentication Code (HMAC).
//! 
//! To use it you'll need a cryptographic hash function implementation from
//! RustCrypto project. You can either import specific crate (e.g. `sha2`), or
//! meta-crate `crypto-hashes` which reexport all related crates.
//! 
//! # Usage
//! Let us demonstrate how to use HMAC using SHA256 as an example.
//! 
//! To get the authentication code:
//! 
//! ```rust,ignore
//! extern crate sha2;
//! extern crate hmac;
//! 
//! use hmac::{Hmac, Mac};
//! use sha2::Sha256;
//! 
//! // Create `Mac` trait implementation, namely HMAC-SHA256
//! let mac = Hmac::<Sha256>::new(b"my secret and secure key");
//! mac.input(b"input message");
//! 
//! // `result` has type `MacResult` which is a thin wrapper around array of
//! // bytes for providing constant time equality check
//! let result = mac.result();
//! // To get &[u8] use `code` method, but be carefull, since incorrect use
//! // of the code value may permit timing attacks which defeat the security
//! // provided by the `MacResult`.
//! let code_bytes = resul.code();
//! ```
//! 
//! To verify the message:
//! 
//! ```rust,ignore
//! let mac = Hmac::<Sha256>::new(b"my secret and secure key");
//! 
//! mac.input(b"input message");
//! 
//! let is_code_correct = mac.verify(code_bytes); 
//! ```

#![no_std]
extern crate generic_array;
extern crate digest;
extern crate crypto_mac;

pub use crypto_mac::Mac;
pub use crypto_mac::MacResult;
use digest::{Input, FixedOutput};
use generic_array::{ArrayLength, GenericArray};

const IPAD: u8 = 0x36;
const OPAD: u8 = 0x5c;

/// The `Hmac` struct represents an HMAC using a given hash function `D`.
pub struct Hmac<D>
    where D: Input + FixedOutput + Default,
          <D as Input>::BlockSize: ArrayLength<u8>
{
    digest: D,
    key: GenericArray<u8, D::BlockSize>,
}

/// The key that Hmac processes must be the same as the block size of the
/// underlying Digest. If the provided key is smaller than that, we just pad it
/// with zeros. If its larger, we hash it and then pad it with zeros.
fn expand_key<D>(key: &[u8]) -> GenericArray<u8, D::BlockSize>
    where D: Input + FixedOutput + Default,
          <D as Input>::BlockSize: ArrayLength<u8>
{
    let mut exp_key = GenericArray::default();
    
    if key.len() <= exp_key.len() {
        exp_key[..key.len()].copy_from_slice(key);
    } else {
        let mut digest = D::default();
        digest.digest(key);
        let output = digest.fixed_result();
        exp_key[..output.len()].copy_from_slice(output.as_slice());
    }
    exp_key
}

impl <D> Hmac<D>
    where D: Input + FixedOutput + Default,
          <D as Input>::BlockSize: ArrayLength<u8>
{
    fn derive_key(&self, mask: u8) -> GenericArray<u8, D::BlockSize> {
        let mut key = self.key.clone();
        for elem in key.iter_mut() {
            *elem ^= mask;
        }
        key
    }
}

impl <D> Mac for Hmac<D>
    where D: Input + FixedOutput + Default,
          <D as Input>::BlockSize: ArrayLength<u8>,
          <D as FixedOutput>::OutputSize: ArrayLength<u8>
{
    type OutputSize = D::OutputSize;

    fn new(key: &[u8]) -> Hmac<D> {
        let mut hmac = Hmac {
            digest: D::default(),
            key: expand_key::<D>(key),
        };
        let i_key_pad = hmac.derive_key(IPAD);
        hmac.digest.digest(&i_key_pad);
        hmac
    }

    fn input(&mut self, data: &[u8]) {
        self.digest.digest(data);
    }

    fn result(self) -> MacResult<D::OutputSize> {
        let o_key_pad = self.derive_key(OPAD);
        let output = self.digest.fixed_result();
        let mut digest = D::default();
        digest.digest(&o_key_pad);
        digest.digest(&output);
        MacResult::new(digest.fixed_result())
    }
}
