use digest::{Digest, FixedOutputReset, Output, Reset};
use std::io;

#[cfg(feature = "tokio")]
use {
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
    tokio::io::AsyncWrite,
};

/// Abstraction over a writer which hashes the data being written.
#[cfg(not(feature = "tokio"))]
#[derive(Debug)]
pub struct HashWriter<D, W> {
    writer: W,
    hasher: D,
}

#[cfg(feature = "tokio")]
pin_project_lite::pin_project! {
    /// Abstraction over a writer which hashes the data being written.
    #[derive(Debug)]
    pub struct HashWriter<D, W> {
        #[pin]
        writer: W,
        hasher: D,
    }
}

impl<D: Digest, W> HashWriter<D, W> {
    /// Construct a new `HashWriter` given an existing `writer` by value.
    pub fn new(writer: W) -> Self {
        Self::new_from_parts(D::new(), writer)
    }

    /// Construct a new `HashWriter` given an existing `hasher` and `writer` by value.
    pub fn new_from_parts(hasher: D, writer: W) -> Self {
        HashWriter { writer, hasher }
    }

    /// Replace the writer with another writer
    pub fn replace_writer(&mut self, writer: W) {
        self.writer = writer;
    }

    /// Gets a reference to the underlying hasher
    pub fn get_hasher(&self) -> &D {
        &self.hasher
    }

    /// Gets a reference to the underlying writer
    pub fn get_writer(&self) -> &W {
        &self.writer
    }

    /// Gets a mutable reference to the underlying hasher
    /// Updates to the digest are not written to the underlying writer
    pub fn get_hasher_mut(&mut self) -> &mut D {
        &mut self.hasher
    }

    /// Gets a mutable reference to the underlying writer
    /// Direct writes to the underlying writer are not hashed
    pub fn get_writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Consume the HashWriter and return its hasher
    pub fn into_hasher(self) -> D {
        self.hasher
    }

    /// Consume the HashWriter and return its internal writer
    pub fn into_inner_writer(self) -> W {
        self.writer
    }

    /// Consume the HashWriter and return its hasher and internal writer
    pub fn into_parts(self) -> (D, W) {
        (self.hasher, self.writer)
    }

    /// Retrieve result and consume HashWriter instance.
    pub fn finalize(self) -> Output<D> {
        self.hasher.finalize()
    }

    /// Write result into provided array and consume the HashWriter instance.
    pub fn finalize_into(self, out: &mut Output<D>) {
        self.hasher.finalize_into(out)
    }

    /// Get output size of the hasher
    pub fn output_size() -> usize {
        <D as Digest>::output_size()
    }
}

impl<D: Digest + Clone, W: Clone> Clone for HashWriter<D, W> {
    fn clone(&self) -> HashWriter<D, W> {
        HashWriter {
            writer: self.writer.clone(),
            hasher: self.hasher.clone(),
        }
    }
}

impl<D: Digest, W: io::Write> io::Write for HashWriter<D, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes = self.writer.write(buf)?;

        if bytes > 0 {
            self.hasher.update(&buf[0..bytes]);
        }

        Ok(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(feature = "tokio")]
impl<D: Digest, W: AsyncWrite> AsyncWrite for HashWriter<D, W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let result = this.writer.poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = result {
            if n > 0 {
                this.hasher.update(&buf[..n]);
            }
        }
        result
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_shutdown(cx)
    }
}

impl<D: Digest + FixedOutputReset, W: io::Write> HashWriter<D, W> {
    /// Retrieve result and reset hasher instance.
    pub fn finalize_reset(&mut self) -> Output<D> {
        Digest::finalize_reset(&mut self.hasher)
    }

    /// Write result into provided array and reset the hasher instance.
    pub fn finalize_into_reset(&mut self, out: &mut Output<D>) {
        Digest::finalize_into_reset(&mut self.hasher, out)
    }
}

impl<D: Digest + Reset, W> Reset for HashWriter<D, W> {
    fn reset(&mut self) {
        Digest::reset(&mut self.hasher)
    }
}

#[cfg(all(test, feature = "tokio"))]
mod tokio_tests {
    use super::HashWriter;
    use bytes::Bytes;
    use digest::Digest;
    use futures::stream;
    use sha2::Sha256;
    use tokio::io::AsyncWriteExt;
    use tokio_util::io::StreamReader;

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
}
