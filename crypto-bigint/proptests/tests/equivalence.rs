//! Equivalence tests between `num-bigint` and `crypto-bigint`

use crypto_bigint::{Encoding, U256};
use num_bigint::BigUint;
use proptest::prelude::*;
use std::mem;

fn to_biguint(uint: &U256) -> BigUint {
    BigUint::from_bytes_be(uint.to_be_bytes().as_ref())
}

fn to_uint(big_uint: BigUint) -> U256 {
    let mut input = [0u8; U256::BYTE_SIZE];
    let encoded = big_uint.to_bytes_be();

    match U256::BYTE_SIZE.checked_sub(encoded.len()) {
        Some(off) => input[off..].copy_from_slice(&encoded),
        None => {
            let off = encoded.len() - U256::BYTE_SIZE;
            input.copy_from_slice(&encoded[off..]);
        }
    }

    U256::from_be_slice(&input)
}

prop_compose! {
    fn uint()(bytes in any::<[u8; 32]>()) -> U256 {
        U256::from_be_slice(&bytes)
    }
}

proptest! {
    #[test]
    fn wrapping_add(a in uint(), b in uint()) {
        let a_bi = to_biguint(&a);
        let b_bi = to_biguint(&b);

        let expected = to_uint(a_bi + b_bi);
        let actual = a.wrapping_add(&b);

        assert_eq!(expected, actual);
    }

    #[test]
    fn wrapping_sub(mut a in uint(), mut b in uint()) {
        if b > a {
            mem::swap(&mut a, &mut b);
        }

        let a_bi = to_biguint(&a);
        let b_bi = to_biguint(&b);

        let expected = to_uint(a_bi - b_bi);
        let actual = a.wrapping_sub(&b);

        assert_eq!(expected, actual);
    }

    #[test]
    fn wrapping_mul(a in uint(), b in uint()) {
        let a_bi = to_biguint(&a);
        let b_bi = to_biguint(&b);

        let expected = to_uint(a_bi * b_bi);
        let actual = a.wrapping_mul(&b);

        assert_eq!(expected, actual);
    }
}
