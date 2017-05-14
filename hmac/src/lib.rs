#![no_std]
extern crate generic_array;
extern crate digest;
extern crate crypto_mac;

pub use crypto_mac::{Mac, MacResult};
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
