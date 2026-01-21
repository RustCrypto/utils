//! `blobby` tests
#![cfg(feature = "alloc")]

const ITEMS_LEN: usize = 10;
const DEDUP_LEN: usize = 3;
const TEST_BLOBS: &[&[u8]; ITEMS_LEN] = &[
    b"1",
    b"12",
    b"1",
    b"1",
    b"123",
    &[42; 100_000],
    &[42; 100_000],
    &[13; 7_000],
    &[13; 7_000],
    &[13; 5_000],
];

/// Performs a round-trip test.
#[test]
fn blobby_rondtrip_test() -> Result<(), blobby::Error> {
    let (blobby_data, dedup_len) = blobby::encode_blobs(TEST_BLOBS);
    assert_eq!(dedup_len, DEDUP_LEN);
    assert_eq!(blobby_data.len(), 112_025);

    let decoded_blobs = blobby::parse_into_array::<ITEMS_LEN, DEDUP_LEN>(&blobby_data)?;
    assert_eq!(decoded_blobs, TEST_BLOBS[..]);

    let decoded_blobs = blobby::parse_into_vec(&blobby_data)?;
    assert_eq!(decoded_blobs, TEST_BLOBS[..]);

    Ok(())
}
