//! PEM encoder.

use crate::{
    grammar, Error, Result, BASE64_WRAP_WIDTH, ENCAPSULATION_BOUNDARY_DELIMITER,
    POST_ENCAPSULATION_BOUNDARY, PRE_ENCAPSULATION_BOUNDARY,
};
use base64ct::{Base64, Encoding};

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Newline character to use
// TODO(tarcieri): make newline character customizable
const NEWLINE: &[u8] = b"\n";

/// Encode a PEM document according to RFC 7468's "Strict" grammar.
pub fn encode<'a>(label: &str, input: &[u8], buf: &'a mut [u8]) -> Result<&'a [u8]> {
    grammar::validate_label(label.as_bytes())?;

    let mut buf = Buffer::new(buf);
    buf.write(PRE_ENCAPSULATION_BOUNDARY)?;
    buf.write(label.as_bytes())?;
    buf.writeln(ENCAPSULATION_BOUNDARY_DELIMITER)?;

    for chunk in input.chunks((BASE64_WRAP_WIDTH * 3) / 4) {
        buf.write_base64ln(chunk)?;
    }

    buf.write(POST_ENCAPSULATION_BOUNDARY)?;
    buf.write(label.as_bytes())?;
    buf.writeln(ENCAPSULATION_BOUNDARY_DELIMITER)?;
    buf.finish()
}

/// Get the length of a PEM encoded document with the given bytes and label.
pub fn encoded_len(label: &str, input: &[u8]) -> usize {
    // TODO(tarcieri): use checked arithmetic
    PRE_ENCAPSULATION_BOUNDARY.len()
        + label.as_bytes().len()
        + ENCAPSULATION_BOUNDARY_DELIMITER.len()
        + NEWLINE.len()
        + input
            .chunks((BASE64_WRAP_WIDTH * 3) / 4)
            .fold(0, |acc, chunk| {
                acc + Base64::encoded_len(chunk) + NEWLINE.len()
            })
        + POST_ENCAPSULATION_BOUNDARY.len()
        + label.as_bytes().len()
        + ENCAPSULATION_BOUNDARY_DELIMITER.len()
        + NEWLINE.len()
}

/// Encode a PEM document according to RFC 7468's "Strict" grammar, returning
/// the result as a [`String`].
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn encode_string(label: &str, input: &[u8]) -> Result<String> {
    let mut buf = vec![0u8; encoded_len(label, input)];
    encode(label, input, &mut buf)?;
    String::from_utf8(buf).map_err(|_| Error::CharacterEncoding)
}

/// Output buffer for writing encoded PEM output.
struct Buffer<'a> {
    /// Backing byte slice where PEM output is being written.
    bytes: &'a mut [u8],

    /// Total number of bytes written into the buffer so far.
    position: usize,
}

impl<'a> Buffer<'a> {
    /// Initialize buffer.
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    /// Write a byte slice to the buffer.
    pub fn write(&mut self, slice: &[u8]) -> Result<()> {
        let reserved = self.reserve(slice.len())?;
        reserved.copy_from_slice(slice);
        Ok(())
    }

    /// Write a byte slice to the buffer with a newline at the end.
    pub fn writeln(&mut self, slice: &[u8]) -> Result<()> {
        self.write(slice)?;
        self.write(NEWLINE)
    }

    /// Write Base64-encoded data to the buffer.
    ///
    /// Automatically adds a newline at the end.
    pub fn write_base64ln(&mut self, bytes: &[u8]) -> Result<()> {
        let reserved = self.reserve(Base64::encoded_len(bytes))?;
        Base64::encode(bytes, reserved)?;
        self.write(NEWLINE)
    }

    /// Finish writing to the buffer, returning the portion that has been
    /// written to.
    pub fn finish(self) -> Result<&'a [u8]> {
        self.bytes.get(..self.position).ok_or(Error::Length)
    }

    /// Reserve space in the encoding buffer, returning a mutable slice.
    fn reserve(&mut self, nbytes: usize) -> Result<&mut [u8]> {
        let new_position = self.position.checked_add(nbytes).ok_or(Error::Length)?;

        let reserved = self
            .bytes
            .get_mut(self.position..new_position)
            .ok_or(Error::Length)?;

        self.position = new_position;
        Ok(reserved)
    }
}
