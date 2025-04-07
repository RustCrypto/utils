use digest::{Update, XofReader};
use std::io;

/// Wrapper which allows to update state of an [`Update`] implementor by writing into it
/// and to read data from a [`XofReader`] implementor.
#[derive(Debug)]
pub struct IoWrapper<T>(pub T);

impl<T: Update> io::Write for IoWrapper<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Update::update(&mut self.0, buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: XofReader> io::Read for IoWrapper<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        XofReader::read(&mut self.0, buf);
        Ok(buf.len())
    }
}
