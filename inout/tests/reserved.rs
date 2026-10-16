//! Tests for `InOutBufReserved`
#![cfg(feature = "block-padding")]
use hybrid_array::sizes::U16;
use inout::{
    InOutBufReserved,
    block_padding::{Iso7816, Padding, Pkcs7},
};

#[test]
fn padded_inout_buf_reserved() {
    let mut buf = [0xFF; 48];
    let inout_buf = InOutBufReserved::from_mut_slice(&mut buf, 46).unwrap();
    let mut padded_blocks = inout_buf.into_padded_blocks::<Iso7816, U16>().unwrap();

    assert_eq!(padded_blocks.get_blocks().len(), 2);
    padded_blocks
        .get_blocks()
        .get_out()
        .iter()
        .flatten()
        .for_each(|x| assert_eq!(*x, 0xFF));

    let Some(tail) = padded_blocks.get_tail_block() else {
        panic!("expected a tail block");
    };
    let arr_unpadded = Iso7816::unpad(tail.get_in()).unwrap();
    assert_eq!(arr_unpadded, &[0xFF; 14]);

    let out = padded_blocks.into_out();
    assert_eq!(out, &[0xFF; 48]);
}

#[test]
fn reserved_invalid_lengths() {
    let msg = [42; 32];
    let mut buf = [0; 32];

    for msg_len in 0..msg.len() {
        let msg = &msg[..msg_len];
        for out_len in 0..buf.len() {
            let out_buf = &mut buf[..out_len];

            let reserved = match InOutBufReserved::from_slices(msg, out_buf) {
                Ok(r) => r,
                Err(_) => {
                    assert!(msg_len > out_len);
                    continue;
                }
            };

            assert!(msg_len <= out_len);

            let full_blocks = msg_len / 16;
            let out_blocks = out_len / 16;

            let mut padded = match reserved.into_padded_blocks::<Pkcs7, U16>() {
                Ok(r) => r,
                Err(_) => {
                    assert!(out_blocks <= full_blocks);
                    continue;
                }
            };

            assert!(out_blocks > full_blocks);

            // Access the referenced memory to check it with Miri
            let mut full_blocks = padded.get_blocks();
            full_blocks.get_in().to_vec();
            full_blocks.get_out().to_vec();

            let mut tail_block = padded.get_tail_block().unwrap();
            tail_block.get_in().to_vec();
            tail_block.get_out().to_vec();

            padded.into_out().to_vec();
        }
    }
}
