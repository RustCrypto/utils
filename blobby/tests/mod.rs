#![no_std]
extern crate blobby;

fn check(data: &[u8], result: &[&[u8]]) {
    let mut blob = blobby::BlobIterator::new(data).unwrap();
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
fn test_single0() {
    let data = b"blobby1";
    check(data, &[]);
}

#[test]
fn test_single1() {
    let data = b"blobby1\x01a";
    check(data, &[b"a"]);
}


#[test]
fn test_single2() {
    let data = b"blobby1\x03abc\x00\x01a";
    check(data, &[b"abc", b"", b"a"]);
}

#[test]
fn test_single3() {
    let data = b"blobby2\x03\x00abc\x00\x00\x01\x00a";
    check(data, &[b"abc", b"", b"a"]);
}


#[test]
fn test_single4() {
    let data = b"blobby4\x03\x00\x00\x00abc\x00\x00\x00\x00\x01\x00\x00\x00a";
    check(data, &[b"abc", b"", b"a"]);
}


#[test]
fn test_single5() {
    let data = b"blobby8\
        \x03\x00\x00\x00\x00\x00\x00\x00abc\
        \x00\x00\x00\x00\x00\x00\x00\x00\
        \x01\x00\x00\x00\x00\x00\x00\x00a";
    check(data, &[b"abc", b"", b"a"]);
}

#[test]
fn test_double() {
    let data = b"blobby1\x03abc\x00\x01a\x02cd";
    let result: &[[&[u8]; 2]] = &[[b"abc", b""], [b"a", b"cd"]];

    let mut blob = blobby::Blob2Iterator::new(data).unwrap();
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
fn test_triple() {
    let data = b"blobby1\x03abc\x00\x01a\x02cd\x03def\x00";
    let result: &[[&[u8]; 3]] = &[[b"abc", b"", b"a"], [b"cd", b"def", b""]];

    let mut blob = blobby::Blob3Iterator::new(data).unwrap();
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
fn test_quadruple() {
    let data = b"blobby1\x03abc\x00\x01a\x02cd";
    let result: &[[&[u8]; 4]] = &[[b"abc", b"", b"a", b"cd"]];

    let mut blob = blobby::Blob4Iterator::new(data).unwrap();
    let mut res = result.iter();
    loop {
        match (blob.next(), res.next()) {
            (Some(v1), Some(v2)) => assert_eq!(&v1, v2),
            (None, None) => break,
            _ => panic!("items number mismatch"),
        }
    }
}
