//! `CryptoPAn`
//! Anonymizes IP addresses using the CryptoPAN algorithm tightly based on the
//! GO implementation by Yawning Angel [Repo](https://github.com/Yawning/cryptopan), which is based on the
//! original reference implementation [paper by J. Fan, J. Xu, M. Ammar, and S. Moon.
//! [Paper](https://ieeexplore.ieee.org/abstract/document/1181415)]  
//!   
//! Instantiate using a 256-bit key using `new` or `new_from_slice` and use `encrypt_v4` and `encrypt_v6` to encrypt IP addresses
//! ```
//! use cryptopan::CryptoPAn;
//! use std::net::{Ipv4Addr,Ipv6Addr};
//! use std::str::FromStr;
//! use aes::cipher::KeyInit;
//!   
//! const KEY: [u8; 32] = [21, 34, 23, 141, 51, 164, 207, 128, 19, 10, 91, 22, 73, 144, 125, 16,  
//! 216, 152, 143, 131, 121,121, 101, 39, 98, 87, 76, 45, 42, 132, 34, 2,];
//!   
//! let cp = CryptoPAn::new(&KEY.into());
//! cp.encrypt_v4(Ipv4Addr::from_str("127.0.0.1").unwrap());
//! cp.encrypt_v6(Ipv6Addr::from_str("2001:db8::1").unwrap());
//! ```
use aes::cipher::BlockEncryptMut;
use aes::cipher::{generic_array::GenericArray, InvalidLength, KeyInit};
use bitvec::field::BitField;
use bitvec::prelude::Msb0;
use bitvec::{bitvec, vec::BitVec};
use std::net::{Ipv4Addr, Ipv6Addr};

/// The CryptoPAN anonymizer
pub struct CryptoPAn {
    aes: aes::Aes128,
    pad: BitVec<u8, Msb0>,
}

// Enforce a 256-bit key for `new`
impl aes::cipher::KeySizeUser for CryptoPAn {
    type KeySize = aes::cipher::consts::U32;
}

impl KeyInit for CryptoPAn {
    /// Creates new CryptoPAn using 256-bit key
    fn new(key: &aes::cipher::Key<Self>) -> Self {
        // Create new AES128 encryptor with first 128 bits
        let mut aes = aes::Aes128::new(GenericArray::from_slice(&key[0..16]));

        // Encrypt the second 128 bits
        let mut buf = GenericArray::clone_from_slice(&key[16..32]);
        aes.encrypt_block_mut(&mut buf);

        Self {
            aes,
            pad: BitVec::from_slice(buf.as_slice()),
        }
    }

    /// Creates new CryptoPAn from a generic slice
    fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
        match key.len() {
            32 => Ok(Self::new(key.into())),
            _ => Err(InvalidLength),
        }
    }
}

impl CryptoPAn {
    /// Encrypts an IpV4Addr
    pub fn encrypt_v4(&self, input: Ipv4Addr) -> Ipv4Addr {
        let bits = self.encrypt(BitVec::<u8, Msb0>::from_slice(&input.octets()));
        let bs = bits.as_bitslice();
        Ipv4Addr::new(
            bs[0..8].load::<u8>(),
            bs[8..16].load::<u8>(),
            bs[16..24].load::<u8>(),
            bs[24..32].load::<u8>(),
        )
    }

    /// Encrypts an IpV6Addr
    pub fn encrypt_v6(&self, input: Ipv6Addr) -> Ipv6Addr {
        let bits = self.encrypt(BitVec::<u8, Msb0>::from_slice(&input.octets()));
        let bs = bits.as_bitslice();
        Ipv6Addr::new(
            bs[0..16].load::<u16>(),
            bs[16..32].load::<u16>(),
            bs[32..48].load::<u16>(),
            bs[48..64].load::<u16>(),
            bs[64..80].load::<u16>(),
            bs[80..96].load::<u16>(),
            bs[96..112].load::<u16>(),
            bs[112..128].load::<u16>(),
        )
    }

    /// Encrypts an IP address using the key
    fn encrypt(&self, bits: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let mut encrypted = bitvec!(u8, Msb0; 0; bits.len());
        let mut padding = self.pad.clone();

        // The first bit does not take from the original address
        let mut encpadding = padding.clone();
        let g = GenericArray::from_mut_slice(encpadding.as_raw_mut_slice());
        self.aes.clone().encrypt_block_mut(g);
        let firstpadding: BitVec<u8, Msb0> = BitVec::from_slice(g.as_slice());
        encrypted.set(0, firstpadding[0] ^ bits[0]);

        for n in 1..bits.len() {
            padding.set(n - 1, bits[n - 1]);

            // encrypt padded - this is used as a psuedorandom function
            let mut encpadding = padding.clone();
            let g = GenericArray::from_mut_slice(encpadding.as_raw_mut_slice());
            self.aes.clone().encrypt_block_mut(g);
            let res: BitVec<u8, Msb0> = BitVec::from_slice(g.as_slice());

            // get the first bit of the encrypted value
            encrypted.set(n, res[0] ^ bits[n]);
        }

        encrypted
    }
}
