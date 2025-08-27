#![cfg(feature = "alloc")]

const TEST_BLOBS: &[&[u8]] = &[
    b"1",
    b"12",
    b"1",
    b"1",
    b"123",
    &[42; 100_000],
    &[42; 100_000],
    &[13; 70_000],
];

#[test]
fn blobby_rondtrip_test() -> Result<(), blobby::Error> {
    let (blobby_data, dedup_len) = blobby::encode_blobs(TEST_BLOBS);
    assert_eq!(dedup_len, 2);
    assert_eq!(blobby_data.len(), 170_022);

    let decoded_blobs = blobby::parse_into_array::<8, 2>(&blobby_data)?;
    assert_eq!(decoded_blobs, TEST_BLOBS);

    let decoded_blobs = blobby::parse_into_vec(&blobby_data)?;
    assert_eq!(decoded_blobs, TEST_BLOBS);

    Ok(())
}
