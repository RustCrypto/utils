//! Tests for `InOutBufReserved`
#![cfg(feature = "block-padding")]
use hybrid_array::sizes::U16;
use inout::{
    InOutBufReserved,
    block_padding::{Iso7816, Padding},
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
