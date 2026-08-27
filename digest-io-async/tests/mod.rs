use crate::{HashReader, HashWriter};
use bytes::Bytes;
use digest::Digest;
use futures::stream;
use sha2::Sha256;
use tokio_util::io::StreamReader;

#[tokio::test]
async fn test_async_read() {
    let data = b"the quick brown fox jumps over the lazy dog".repeat(1000);

    // Feed the stream chunk by chunk with an odd sized buffer.
    let chunks = stream::iter(
        data.chunks(37)
            .map(|c| Ok::<_, std::io::Error>(Bytes::copy_from_slice(c)))
            .collect::<Vec<_>>(),
    );
    let mut reader = HashReader::<Sha256, _>::new(StreamReader::new(chunks));
    let mut sink = Vec::new();
    tokio::io::copy(&mut reader, &mut sink).await.unwrap();

    assert_eq!(sink, data);
    assert_eq!(reader.finalize(), Sha256::digest(&data));
}

#[tokio::test]
async fn test_async_write() {
    let data = b"the quick brown fox jumps over the lazy dog".repeat(1000);

    // Feed the stream chunk by chunk with an odd sized buffer.
    let chunks = stream::iter(
        data.chunks(37)
            .map(|c| Ok::<_, std::io::Error>(Bytes::copy_from_slice(c)))
            .collect::<Vec<_>>(),
    );
    let mut source = StreamReader::new(chunks);

    let mut writer = HashWriter::<Sha256, _>::new(Vec::new());
    tokio::io::copy(&mut source, &mut writer).await.unwrap();
    writer.flush().await.unwrap();

    let (hasher, sink) = writer.into_parts();
    assert_eq!(sink, data);
    assert_eq!(hasher.finalize(), Sha256::digest(&data));
}
