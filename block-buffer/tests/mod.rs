use block_buffer::{
    array::{
        typenum::{U10, U16, U24, U4, U8},
        Array,
    },
    EagerBuffer, LazyBuffer, ReadBuffer,
};
use hex_literal::hex;

#[test]
fn test_eager_digest_pad() {
    let mut buf = EagerBuffer::<U4>::default();
    let inputs = [
        &b"01234567"[..],
        &b"89"[..],
        &b"abcdefghij"[..],
        &b"klmnopqrs"[..],
        &b"tuv"[..],
        &b"wx"[..],
    ];
    let exp_blocks = [
        (0, &[b"0123", b"4567"][..]),
        (2, &[b"89ab"][..]),
        (2, &[b"cdef", b"ghij"][..]),
        (3, &[b"klmn", b"opqr"][..]),
        (4, &[b"stuv"][..]),
    ];
    let exp_poses = [0, 2, 0, 1, 0, 2];

    let mut n = 0;
    for (i, input) in inputs.iter().enumerate() {
        buf.digest_blocks(input, |b| {
            let (j, exp) = exp_blocks[n];
            n += 1;
            assert_eq!(i, j);
            assert_eq!(b.len(), exp.len());
            assert!(b.iter().zip(exp.iter()).all(|v| v.0[..] == v.1[..]));
        });
        assert_eq!(exp_poses[i], buf.get_pos());
    }
    assert_eq!(buf.pad_with_zeros()[..], b"wx\0\0"[..]);
    assert_eq!(buf.get_pos(), 0);
}

#[test]
fn test_lazy_digest_pad() {
    let mut buf = LazyBuffer::<U4>::default();
    let inputs = [
        &b"01234567"[..],
        &b"89"[..],
        &b"abcdefghij"[..],
        &b"klmnopqrs"[..],
    ];
    let expected = [
        (0, &[b"0123"][..]),
        (1, &[b"4567"][..]),
        (2, &[b"89ab"][..]),
        (2, &[b"cdef"][..]),
        (3, &[b"ghij"][..]),
        (3, &[b"klmn", b"opqr"][..]),
    ];
    let exp_poses = [4, 2, 4, 1];

    let mut n = 0;
    for (i, input) in inputs.iter().enumerate() {
        buf.digest_blocks(input, |b| {
            let (j, exp) = expected[n];
            n += 1;
            assert_eq!(i, j);
            assert_eq!(b.len(), exp.len());
            assert!(b.iter().zip(exp.iter()).all(|v| v.0[..] == v.1[..]));
        });
        assert_eq!(exp_poses[i], buf.get_pos());
    }
    assert_eq!(buf.pad_with_zeros()[..], b"s\0\0\0"[..]);
    assert_eq!(buf.get_pos(), 0);
}

#[test]
fn test_read() {
    type Buf = ReadBuffer<U4>;
    let mut buf = Buf::default();

    let mut n = 0u8;
    let mut gen = |block: &mut Array<u8, U4>| {
        block.iter_mut().for_each(|b| *b = n);
        n += 1;
    };

    let mut out = [0u8; 6];
    buf.read(&mut out, &mut gen);
    assert_eq!(out, [0, 0, 0, 0, 1, 1]);
    assert_eq!(buf.get_pos(), 2);
    assert_eq!(buf.remaining(), 2);

    let mut out = [0u8; 3];
    buf.read(&mut out, &mut gen);
    assert_eq!(out, [1, 1, 2]);
    assert_eq!(buf.get_pos(), 1);
    assert_eq!(buf.remaining(), 3);

    let mut out = [0u8; 3];
    buf.read(&mut out, &mut gen);
    assert_eq!(out, [2, 2, 2]);
    assert_eq!(buf.get_pos(), 4);
    assert_eq!(buf.remaining(), 0);

    assert_eq!(n, 3);
}

#[test]
fn test_eager_paddings() {
    let mut buf_be = EagerBuffer::<U8>::new(&[0x42]);
    let mut buf_le = buf_be.clone();
    let mut out_be = Vec::<u8>::new();
    let mut out_le = Vec::<u8>::new();
    let len = 0x0001_0203_0405_0607;
    buf_be.len64_padding_be(len, |block| out_be.extend(block));
    buf_le.len64_padding_le(len, |block| out_le.extend(block));

    assert_eq!(out_be, hex!("4280000000000000 0001020304050607"));
    assert_eq!(out_le, hex!("4280000000000000 0706050403020100"));

    let mut buf_be = EagerBuffer::<U10>::new(&[0x42]);
    let mut buf_le = buf_be.clone();
    let mut out_be = Vec::<u8>::new();
    let mut out_le = Vec::<u8>::new();
    buf_be.len64_padding_be(len, |block| out_be.extend(block));
    buf_le.len64_padding_le(len, |block| out_le.extend(block));

    assert_eq!(out_be, hex!("42800001020304050607"));
    assert_eq!(out_le, hex!("42800706050403020100"));

    let mut buf = EagerBuffer::<U16>::new(&[0x42]);
    let mut out = Vec::<u8>::new();
    let len = 0x0001_0203_0405_0607_0809_0a0b_0c0d_0e0f;
    buf.len128_padding_be(len, |block| out.extend(block));
    assert_eq!(
        out,
        hex!(
            "42800000000000000000000000000000"
            "000102030405060708090a0b0c0d0e0f"
        ),
    );

    let mut buf = EagerBuffer::<U24>::new(&[0x42]);
    let mut out = Vec::<u8>::new();
    let len = 0x0001_0203_0405_0607_0809_0a0b_0c0d_0e0f;
    buf.len128_padding_be(len, |block| out.extend(block));
    assert_eq!(
        out,
        hex!("4280000000000000 0001020304050607 08090a0b0c0d0e0f")
    );

    let mut buf = EagerBuffer::<U4>::new(&[0x42]);
    let mut out = Vec::<u8>::new();
    buf.digest_pad(0xff, &hex!("101112"), |block| out.extend(block));
    assert_eq!(out, hex!("42ff000000101112"));

    let mut buf = EagerBuffer::<U4>::new(&[0x42]);
    let mut out = Vec::<u8>::new();
    buf.digest_pad(0xff, &hex!("1011"), |block| out.extend(block));
    assert_eq!(out, hex!("42ff1011"));
}

#[test]
fn test_try_new() {
    assert!(EagerBuffer::<U4>::try_new(&[0; 3]).is_ok());
    assert!(EagerBuffer::<U4>::try_new(&[0; 4]).is_err());
    assert!(LazyBuffer::<U4>::try_new(&[0; 4]).is_ok());
    assert!(LazyBuffer::<U4>::try_new(&[0; 5]).is_err());
}

#[test]
fn test_eager_serialize() {
    type Buf = EagerBuffer<U4>;

    let mut buf1 = Buf::default();
    let ser0 = buf1.serialize();
    assert_eq!(&ser0[..], &[0, 0, 0, 0]);
    assert_eq!(Buf::deserialize(&ser0).unwrap().serialize(), ser0);

    buf1.digest_blocks(&[41, 42], |_| {});

    let ser1 = buf1.serialize();
    assert_eq!(&ser1[..], &[41, 42, 0, 2]);

    let mut buf2 = Buf::deserialize(&ser1).unwrap();
    assert_eq!(buf1.serialize(), ser1);

    buf1.digest_blocks(&[43], |_| {});
    buf2.digest_blocks(&[43], |_| {});

    let ser2 = buf1.serialize();
    assert_eq!(&ser2[..], &[41, 42, 43, 3]);
    assert_eq!(buf1.serialize(), ser2);

    let mut buf3 = Buf::deserialize(&ser2).unwrap();
    assert_eq!(buf3.serialize(), ser2);

    buf1.digest_blocks(&[44], |_| {});
    buf2.digest_blocks(&[44], |_| {});
    buf3.digest_blocks(&[44], |_| {});

    let ser3 = buf1.serialize();
    assert_eq!(&ser3[..], &[0, 0, 0, 0]);
    assert_eq!(buf2.serialize(), ser3);
    assert_eq!(buf3.serialize(), ser3);

    // Invalid position
    let buf = Array([0, 0, 0, 4]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([0, 0, 0, 10]);
    assert!(Buf::deserialize(&buf).is_err());
    // "Garbage" bytes are not zeroized
    let buf = Array([1, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([0, 1, 0, 1]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([0, 0, 1, 2]);
    assert!(Buf::deserialize(&buf).is_err());
}

#[test]
fn test_lazy_serialize() {
    type Buf = LazyBuffer<U4>;

    let mut buf1 = Buf::default();
    let ser0 = buf1.serialize();
    assert_eq!(&ser0[..], &[0, 0, 0, 0, 0]);
    Buf::deserialize(&ser0).unwrap();
    assert_eq!(Buf::deserialize(&ser0).unwrap().serialize(), ser0);

    buf1.digest_blocks(&[41, 42], |_| {});

    let ser1 = buf1.serialize();
    assert_eq!(&ser1[..], &[2, 41, 42, 0, 0]);

    let mut buf2 = Buf::deserialize(&ser1).unwrap();
    assert_eq!(buf1.serialize(), ser1);

    buf1.digest_blocks(&[43], |_| {});
    buf2.digest_blocks(&[43], |_| {});

    let ser2 = buf1.serialize();
    assert_eq!(&ser2[..], &[3, 41, 42, 43, 0]);
    assert_eq!(buf1.serialize(), ser2);

    let mut buf3 = Buf::deserialize(&ser2).unwrap();
    assert_eq!(buf3.serialize(), ser2);

    buf1.digest_blocks(&[44], |_| {});
    buf2.digest_blocks(&[44], |_| {});
    buf3.digest_blocks(&[44], |_| {});

    let ser3 = buf1.serialize();
    assert_eq!(&ser3[..], &[4, 41, 42, 43, 44]);
    assert_eq!(buf2.serialize(), ser3);
    assert_eq!(buf3.serialize(), ser3);

    buf1.digest_blocks(&[45], |_| {});
    buf2.digest_blocks(&[45], |_| {});
    buf3.digest_blocks(&[45], |_| {});

    let ser4 = buf1.serialize();
    assert_eq!(&ser4[..], &[1, 45, 0, 0, 0]);
    assert_eq!(buf2.serialize(), ser4);
    assert_eq!(buf3.serialize(), ser4);

    // Invalid position
    let buf = Array([10, 0, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([5, 0, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    // "Garbage" bytes are not zeroized
    let buf = Array([0, 1, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([1, 0, 1, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([2, 0, 0, 1, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([3, 0, 0, 0, 1]);
    assert!(Buf::deserialize(&buf).is_err());
}

#[test]
fn test_read_serialize() {
    type Buf = ReadBuffer<U4>;

    let mut n = 42u8;
    let mut gen = |block: &mut Array<u8, U4>| {
        block.iter_mut().for_each(|b| {
            *b = n;
            n += 1;
        });
    };

    let mut buf1 = Buf::default();
    let ser0 = buf1.serialize();
    assert_eq!(&ser0[..], &[4, 0, 0, 0]);
    assert_eq!(Buf::deserialize(&ser0).unwrap().serialize(), ser0);

    buf1.read(&mut [0; 2], &mut gen);

    let ser1 = buf1.serialize();
    assert_eq!(&ser1[..], &[2, 0, 44, 45]);

    let mut buf2 = Buf::deserialize(&ser1).unwrap();
    assert_eq!(buf1.serialize(), ser1);

    buf1.read(&mut [0; 1], &mut gen);
    buf2.read(&mut [0; 1], &mut gen);

    let ser2 = buf1.serialize();
    assert_eq!(&ser2[..], &[3, 0, 0, 45]);
    assert_eq!(buf1.serialize(), ser2);

    let mut buf3 = Buf::deserialize(&ser2).unwrap();
    assert_eq!(buf3.serialize(), ser2);

    buf1.read(&mut [0; 1], &mut gen);
    buf2.read(&mut [0; 1], &mut gen);
    buf3.read(&mut [0; 1], &mut gen);

    let ser3 = buf1.serialize();
    assert_eq!(&ser3[..], &[4, 0, 0, 0]);
    assert_eq!(buf2.serialize(), ser3);
    assert_eq!(buf3.serialize(), ser3);

    buf1.read(&mut [0; 1], &mut gen);
    buf2.read(&mut [0; 1], &mut gen);
    buf3.read(&mut [0; 1], &mut gen);

    // note that each buffer calls `gen`, so they get filled
    // with different data
    assert_eq!(&buf1.serialize()[..], &[1, 47, 48, 49]);
    assert_eq!(&buf2.serialize()[..], &[1, 51, 52, 53]);
    assert_eq!(&buf3.serialize()[..], &[1, 55, 56, 57]);

    // Invalid position
    let buf = Array([0, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([5, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([10, 0, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    // "Garbage" bytes are not zeroized
    let buf = Array([2, 1, 0, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([3, 0, 1, 0]);
    assert!(Buf::deserialize(&buf).is_err());
    let buf = Array([4, 0, 0, 1]);
    assert!(Buf::deserialize(&buf).is_err());
}
