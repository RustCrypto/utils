use digest::{Digest, FixedOutputReset, Output, Reset};
use std::io;

#[cfg(feature = "tokio")]
use {
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
    tokio::io::{AsyncRead, ReadBuf},
};

/// Abstraction over a reader which hashes the data being read
#[cfg(not(feature = "tokio"))]
#[derive(Debug)]
pub struct HashReader<D, R> {
    reader: R,
    hasher: D,
}

#[cfg(feature = "tokio")]
pin_project_lite::pin_project! {
    /// Abstraction over a reader which hashes the data being read
    #[derive(Debug)]
    pub struct HashReader<D, R> {
        #[pin]
        reader: R,
        hasher: D,
    }
}

impl<D: Digest, R> HashReader<D, R> {
    /// Construct a new `HashReader` given an existing `reader` by value.
    pub fn new(reader: R) -> Self {
        Self::new_from_parts(D::new(), reader)
    }

    /// Construct a new `HashReader` given an existing `hasher` and `reader` by value.
    pub fn new_from_parts(hasher: D, reader: R) -> Self {
        HashReader { reader, hasher }
    }

    /// Replace the reader with another reader
    pub fn replace_reader(&mut self, reader: R) {
        self.reader = reader;
    }

    /// Gets a reference to the underlying hasher
    pub fn get_hasher(&self) -> &D {
        &self.hasher
    }

    /// Gets a reference to the underlying reader
    pub fn get_reader(&self) -> &R {
        &self.reader
    }

    /// Gets a mutable reference to the underlying hasher
    pub fn get_hasher_mut(&mut self) -> &mut D {
        &mut self.hasher
    }

    /// Gets a mutable reference to the underlying reader
    /// Direct reads from the underlying reader are not hashed
    pub fn get_reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consume the HashReader and return its hasher
    pub fn into_hasher(self) -> D {
        self.hasher
    }

    /// Consume the HashReader and return its internal reader
    pub fn into_inner_reader(self) -> R {
        self.reader
    }

    /// Consume the HashReader and return its hasher and internal reader
    pub fn into_parts(self) -> (D, R) {
        (self.hasher, self.reader)
    }

    /// Retrieve result and consume HashReader instance.
    pub fn finalize(self) -> Output<D> {
        self.hasher.finalize()
    }

    /// Write result into provided array and consume the HashReader instance.
    pub fn finalize_into(self, out: &mut Output<D>) {
        self.hasher.finalize_into(out)
    }

    /// Get output size of the hasher
    pub fn output_size() -> usize {
        <D as Digest>::output_size()
    }
}

impl<D: Digest + Clone, R: Clone> Clone for HashReader<D, R> {
    fn clone(&self) -> HashReader<D, R> {
        HashReader {
            reader: self.reader.clone(),
            hasher: self.hasher.clone(),
        }
    }
}

impl<D: Digest, R: io::Read> io::Read for HashReader<D, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.reader.read(buf)?;

        if bytes > 0 {
            self.hasher.update(&buf[0..bytes]);
        }

        Ok(bytes)
    }
}

#[cfg(feature = "tokio")]
impl<D: Digest, R: AsyncRead> AsyncRead for HashReader<D, R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        let filled_before = buf.filled().len();
        let result = this.reader.poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = result {
            let filled_after = buf.filled().len();
            if filled_after > filled_before {
                this.hasher
                    .update(&buf.filled()[filled_before..filled_after]);
            }
        }
        result
    }
}

impl<D: Digest + FixedOutputReset, R: io::Read> HashReader<D, R> {
    /// Retrieve result and reset hasher instance.
    pub fn finalize_reset(&mut self) -> Output<D> {
        Digest::finalize_reset(&mut self.hasher)
    }

    /// Rrite result into provided array and reset the hasher instance.
    pub fn finalize_into_reset(&mut self, out: &mut Output<D>) {
        Digest::finalize_into_reset(&mut self.hasher, out)
    }
}

impl<D: Digest + Reset, R> Reset for HashReader<D, R> {
    fn reset(&mut self) {
        Digest::reset(&mut self.hasher)
    }
}

impl<D: Digest, R: io::BufRead> HashReader<D, R> {
    /// Read and hash all bytes remaining in the reader, discarding the data
    /// Based on implementation in b2sum crate, MIT License Copyright (c) 2017 John Downey
    pub fn hash_to_end(&mut self) {
        loop {
            let count = {
                let data = self.reader.fill_buf().unwrap();
                if data.is_empty() {
                    break;
                }

                self.hasher.update(data);
                data.len()
            };

            self.reader.consume(count);
        }
    }
}

#[cfg(all(test, feature = "tokio"))]
mod tokio_tests {
    use super::HashReader;
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
}
