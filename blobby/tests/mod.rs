#![no_std]
extern crate blobby;

fn check_dup(data: &[u8], result: &[&[u8]]) {
    let mut blob = blobby::DupBlobIterator::new(data).unwrap();
    let mut res = result.iter();
    loop {
        match (blob.next(), res.next()) {
            (Some(v1), Some(v2)) => assert_eq!(&v1, v2),
            (None, None) => break,
            _ => panic!("items number mismatch"),
        }
    }
}

fn check_unique(data: &[u8], result: &[&[u8]]) {
    let mut blob = blobby::UniqueBlobIterator::new(data).unwrap();
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
fn dup_empty() {
    let data = b"BLBD\x00\x00\x00\x00";
    check_dup(data, &[]);
}

#[test]
fn dup_single() {
    let data = b"BLBD\
        \x01\x00\x00\x00\
        \x00\x00\x00\x00\x0A\x00\x00\x00\
        0123456789\
    ";
    check_dup(data, &[b"0123456789"]);
}

#[test]
fn dup_double() {
    let data = b"BLBD\
        \x02\x00\x00\x00\
        \x00\x00\x00\x00\x0A\x00\x00\x00\
        \x0A\x00\x00\x00\x0D\x00\x00\x00\
        0123456789\
        abc\
    ";
    check_dup(data, &[b"0123456789", b"abc"]);
}

#[test]
fn dup_dublicate() {
    let data = b"BLBD\
        \x03\x00\x00\x00\
        \x00\x00\x00\x00\x0A\x00\x00\x00\
        \x0A\x00\x00\x00\x0D\x00\x00\x00\
        \x00\x00\x00\x00\x0A\x00\x00\x00\
        0123456789\
        abc\
    ";
    check_dup(data, &[b"0123456789", b"abc", b"0123456789"]);
}

#[test]
fn unique_empty() {
    let data = b"BLBU\x00\x00\x00\x00";
    check_unique(data, &[]);
}

#[test]
fn unique_single() {
    let data = b"BLBU\
        \x01\x00\x00\x00\
        \x0A\x00\x00\x00\
        0123456789\
    ";
    check_unique(data, &[b"0123456789"]);
}

#[test]
fn unique_double() {
    let data = b"BLBU\
        \x02\x00\x00\x00\
        \x0A\x00\x00\x00\
        \x03\x00\x00\x00\
        0123456789\
        abc\
    ";
    check_unique(data, &[b"0123456789", b"abc"]);
}
