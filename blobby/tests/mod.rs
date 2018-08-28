#![no_std]
extern crate blobby;

fn check(data: &[u8], result: &[&[u8]]) {
    let mut blob = blobby::BlobIterator::new(data);
    let mut res = result.iter();
    loop {
        match (blob.next(), res.next()) {
            (Some(v1), Some(v2)) => assert_eq!(&v1, v2),
            (None, None) => break,
            _ => panic!("items number mismatch"),
        }
    }
}

#[test]
fn empty() {
    let data = b"\x00\x00\x00\x00";
    check(data, &[]);
}

#[test]
fn single() {
    let data = b"\
        \x01\x00\x00\x00\
        \x0A\x00\x00\x00\
        0123456789\
    ";
    check(data, &[b"0123456789"]);
}

#[test]
fn double() {
    let data = b"\
        \x02\x00\x00\x00\
        \x0A\x00\x00\x00\
        \x03\x00\x00\x00\
        0123456789\
        abc\
    ";
    check(data, &[b"0123456789", b"abc"]);
}
