//! `CryptoPAn`
//! IP-address anonymization algorithm that preserves prefixes
//! Tightly based on the GO implementation
//! Author: Yawning Angel <yawning at schwanenlied dot me>
//! [Github Link](./https://github.com/Yawning/cryptopan/blob/master/cryptopan.go)
//! Based upon the algorithm described in:
//! Package cryptopan implements the Crypto-PAn prefix-preserving IP address sanitization algorithm as specified by J. Fan, J. Xu, M. Ammar, and S. Moon.
use aes::cipher::BlockEncryptMut;
use aes::cipher::{generic_array::GenericArray, KeyInit};
use bitvec::field::BitField;
use bitvec::prelude::Msb0;
use bitvec::{bitvec, vec::BitVec};
use std::net::{
    IpAddr,
    IpAddr::{V4, V6},
    Ipv4Addr, Ipv6Addr,
};

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

pub struct CryptoPAn {
    aes: Aes128EcbEnc,
    pad: BitVec<u8, Msb0>,
}

impl CryptoPAn {
    /// Creates new outer block for encryption using key. Panics if key size isn't 256 bits
    pub fn from_key(key: &[u8]) -> Self {
        // Split AES key into two parts - one is padding
        // Encrypt the padding with the first part of the key
        let mut aes = Aes128EcbEnc::new(GenericArray::from_slice(&key[0..16]));

        let mut buf = [0u8; 16];
        buf.copy_from_slice(&key[16..32]);
        let g = GenericArray::from_mut_slice(&mut buf);
        aes.encrypt_block_mut(g);

        Self {
            aes,
            pad: BitVec::from_slice(g.as_slice()),
        }
    }
    /// Creates new outer block for encryption using str key. Panics if key size isn't 256 bits.
    pub fn from_str_key(string_key: &str) -> Self {
        Self::from_key(string_key.as_bytes())
    }
    /// Encrypts either a V4 or V6 IP address using the key
    pub fn encrypt(&self, input: IpAddr) -> IpAddr {
        let bits = match input {
            V4(b) => BitVec::<u8, Msb0>::from_slice(&b.octets()),
            V6(b) => BitVec::<u8, Msb0>::from_slice(&b.octets()),
        };

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

        let bs = encrypted.as_bitslice();

        if input.is_ipv4() {
            V4(Ipv4Addr::new(
                bs[0..8].load::<u8>(),
                bs[8..16].load::<u8>(),
                bs[16..24].load::<u8>(),
                bs[24..32].load::<u8>(),
            ))
        } else {
            V6(Ipv6Addr::new(
                bs[0..16].load::<u16>(),
                bs[16..32].load::<u16>(),
                bs[32..48].load::<u16>(),
                bs[48..64].load::<u16>(),
                bs[64..80].load::<u16>(),
                bs[80..96].load::<u16>(),
                bs[96..112].load::<u16>(),
                bs[112..128].load::<u16>(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, str::FromStr};

    fn get_key() -> CryptoPAn {
        CryptoPAn::from_key(&[
            21u8, 34u8, 23u8, 141u8, 51u8, 164u8, 207u8, 128u8, 19u8, 10u8, 91u8, 22u8, 73u8,
            144u8, 125u8, 16u8, 216u8, 152u8, 143u8, 131u8, 121u8, 121u8, 101u8, 39u8, 98u8, 87u8,
            76u8, 45u8, 42u8, 132u8, 34u8, 2u8,
        ])
    }

    #[test]
    fn test_encrypt_ipv4() {
        // This is a copy of the tests for the Go implementation
        // https://github.com/Yawning/cryptopan/blob/master/cryptopan_test.go
        // IPV4 tests
        let ipv4 = HashMap::from([
            ("128.11.68.132", "135.242.180.132"),
            ("129.118.74.4", "134.136.186.123"),
            ("130.132.252.244", "133.68.164.234"),
            ("141.223.7.43", "141.167.8.160"),
            ("141.233.145.108", "141.129.237.235"),
            ("152.163.225.39", "151.140.114.167"),
            ("156.29.3.236", "147.225.12.42"),
            ("165.247.96.84", "162.9.99.234"),
            ("166.107.77.190", "160.132.178.185"),
            ("192.102.249.13", "252.138.62.131"),
            ("192.215.32.125", "252.43.47.189"),
            ("192.233.80.103", "252.25.108.8"),
            ("192.41.57.43", "252.222.221.184"),
            ("193.150.244.223", "253.169.52.216"),
            ("195.205.63.100", "255.186.223.5"),
            ("198.200.171.101", "249.199.68.213"),
            ("198.26.132.101", "249.36.123.202"),
            ("198.36.213.5", "249.7.21.132"),
            ("198.51.77.238", "249.18.186.254"),
            ("199.217.79.101", "248.38.184.213"),
            ("202.49.198.20", "245.206.7.234"),
            ("203.12.160.252", "244.248.163.4"),
            ("204.184.162.189", "243.192.77.90"),
            ("204.202.136.230", "243.178.4.198"),
            ("204.29.20.4", "243.33.20.123"),
            ("205.178.38.67", "242.108.198.51"),
            ("205.188.147.153", "242.96.16.101"),
            ("205.188.248.25", "242.96.88.27"),
            ("205.245.121.43", "242.21.121.163"),
            ("207.105.49.5", "241.118.205.138"),
            ("207.135.65.238", "241.202.129.222"),
            ("207.155.9.214", "241.220.250.22"),
            ("207.188.7.45", "241.255.249.220"),
            ("207.25.71.27", "241.33.119.156"),
            ("207.33.151.131", "241.1.233.131"),
            ("208.147.89.59", "227.237.98.191"),
            ("208.234.120.210", "227.154.67.17"),
            ("208.28.185.184", "227.39.94.90"),
            ("208.52.56.122", "227.8.63.165"),
            ("209.12.231.7", "226.243.167.8"),
            ("209.238.72.3", "226.6.119.243"),
            ("209.246.74.109", "226.22.124.76"),
            ("209.68.60.238", "226.184.220.233"),
            ("209.85.249.6", "226.170.70.6"),
            ("212.120.124.31", "228.135.163.231"),
            ("212.146.8.236", "228.19.4.234"),
            ("212.186.227.154", "228.59.98.98"),
            ("212.204.172.118", "228.71.195.169"),
            ("212.206.130.201", "228.69.242.193"),
            ("216.148.237.145", "235.84.194.111"),
            ("216.157.30.252", "235.89.31.26"),
            ("216.184.159.48", "235.96.225.78"),
            ("216.227.10.221", "235.28.253.36"),
            ("216.254.18.172", "235.7.16.162"),
            ("216.32.132.250", "235.192.139.38"),
            ("216.35.217.178", "235.195.157.81"),
            ("24.0.250.221", "100.15.198.226"),
            ("24.13.62.231", "100.2.192.247"),
            ("24.14.213.138", "100.1.42.141"),
            ("24.5.0.80", "100.9.15.210"),
            ("24.7.198.88", "100.10.6.25"),
            ("24.94.26.44", "100.88.228.35"),
            ("38.15.67.68", "64.3.66.187"),
            ("4.3.88.225", "124.60.155.63"),
            ("63.14.55.111", "95.9.215.7"),
            ("63.195.241.44", "95.179.238.44"),
            ("63.97.7.140", "95.97.9.123"),
            ("64.14.118.196", "0.255.183.58"),
            ("64.34.154.117", "0.221.154.117"),
            ("64.39.15.238", "0.219.7.41"),
        ]);

        for (k, v) in ipv4 {
            assert_eq!(
                v.to_string(),
                get_key()
                    .encrypt(V4(Ipv4Addr::from_str(k).unwrap()))
                    .to_string()
            );
        }
    }

    #[test]
    fn test_encrypt_ipv6() {
        let ipv6 = HashMap::from([
            (
                "144:bc02:3f60:1dd9:7f02:8eff:f1e6:1edc",
                "4479:8123:80c4:ac3d:728f:e372:ece:1c1e",
            ),
            ("::1", "ff78:1f0:c09f:df20:8083:f1b1:407:ed00"),
            ("::2", "ff78:1f0:c09f:df20:8083:f1b1:407:ef00"),
            ("::ffff", "ff78:1f0:c09f:df20:8083:f1b1:407:38f8"),
            ("2001:db8::1", "144:bc02:3f60:1dd9:7f02:8eff:f1e6:1edc"),
            ("2001:db8::2", "144:bc02:3f60:1dd9:7f02:8eff:f1e6:1cdc"),
        ]);

        for (k, v) in ipv6 {
            assert_eq!(
                v.to_string(),
                get_key()
                    .encrypt(V6(Ipv6Addr::from_str(k).unwrap()))
                    .to_string()
            );
        }
    }
}
