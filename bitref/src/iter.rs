//! Iterator over bits.

use crate::BitSlice;
use core::iter::{FusedIterator, Iterator};

/// Iterator over the bits of a [`BitSlice`].
#[derive(Clone, Debug)]
pub struct Iter<'a> {
    slice: &'a BitSlice,
}

impl<'a> Iter<'a> {
    /// Create a new iterator over a `BitSlice`.
    #[must_use]
    pub fn new(slice: &'a BitSlice) -> Self {
        Self { slice }
    }
}

impl Iterator for Iter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        let (bit, rest) = self.slice.split_first()?;
        self.slice = rest;
        Some(bit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.slice.len()
    }

    fn last(self) -> Option<bool> {
        self.slice.last()
    }

    fn nth(&mut self, n: usize) -> Option<bool> {
        self.slice = self.slice.get_slice(n..self.slice.len()).ok()?;
        self.next()
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<bool> {
        let (bit, rest) = self.slice.split_last()?;
        self.slice = rest;
        Some(bit)
    }

    fn nth_back(&mut self, n: usize) -> Option<bool> {
        let new_end = self.slice.len().checked_sub(n)?;
        self.slice = self.slice.get_slice(0..new_end).ok()?;
        self.next_back()
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.slice.len()
    }
}

impl FusedIterator for Iter<'_> {}
