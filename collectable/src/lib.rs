//! This crate provides a set of traits that can be used to abstract over
//! various collection types. In particular, it supports traits for fallible
//! operations (such as [`TryExtend`]) which are useful for supporting
//! fixed-capacity containers, such as `heapless::Vec` or `tinyvec::Array`.

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Collection types implement all of the traits in this crate.
pub trait Collection<T>:
    AsRef<[T]> + AsMut<[T]> + Default + Length + Truncate + TryExtend<T> + TryPush<T>
{
}

/// Obtain the length of a collection.
pub trait Length {
    /// Get the length of this collection.
    fn len(&self) -> usize;

    /// Is the collection empty?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Truncate the collection to the provided length.
pub trait Truncate {
    /// Truncate this buffer to the given number of elements.
    ///
    /// If `len` is bigger than the current number of elements (or the total
    /// capacity of the buffer) no changes are made to the contents.
    fn truncate(&mut self, len: usize);
}

/// Fallible equivalent of [`core::iter::Extend`] - extends a collection
/// with the contents of an iterator, but with the option to return an error
/// in the event the container's capacity has been exceeded.
///
/// [`core::iter::Extend`]: https://doc.rust-lang.org/core/iter/trait.Extend.html
pub trait TryExtend<A> {
    /// Error type.
    type Error;

    /// Try to extend the collection from the given iterator.
    fn try_extend<T>(&mut self, iter: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = A>;

    /// Try to extend the collection from the given slice.
    fn try_extend_from_slice(&mut self, slice: &[A]) -> Result<(), Self::Error>
    where
        A: Clone,
    {
        self.try_extend(slice.iter().cloned())
    }
}

/// Try to build a collection type from an [`Iterator`].
///
/// Fallible in the event the capacity of the underlying container type is
/// exceeded.
pub trait TryFromIterator<A>: Sized {
    /// Error type.
    type Error;

    /// Try to create a new collection from the given iterator, potentially
    /// returning an error if the underlying collection's capacity is exceeded.
    fn try_from_iter<T>(iter: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator<Item = A>;
}

impl<A, C: Default + TryExtend<A>> TryFromIterator<A> for C {
    type Error = <Self as TryExtend<A>>::Error;

    fn try_from_iter<T>(iter: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator<Item = A>,
    {
        let mut collection = Self::default();
        collection.try_extend(iter)?;
        Ok(collection)
    }
}

/// Try to push an element onto a collection
pub trait TryPush<T> {
    /// Try to push an element onto a collection.
    ///
    /// Returns the original element if it's full.
    fn try_push(&mut self, item: T) -> Result<(), T>;
}

/// [`TryCollect`] is an extension to [`Iterator`] which allows for performing
/// a fallible collection into a collection type.
pub trait TryCollect<A> {
    fn try_collect<B>(&mut self) -> Result<B, B::Error>
    where
        B: TryFromIterator<A>;
}

impl<A, T> TryCollect<A> for T
where
    T: Iterator<Item = A>,
{
    fn try_collect<B>(&mut self) -> Result<B, B::Error>
    where
        B: TryFromIterator<A>,
    {
        B::try_from_iter(self)
    }
}

#[cfg(feature = "alloc")]
mod vec_impls {
    use super::{Length, Truncate, TryExtend};
    use alloc::vec::Vec;
    use core::convert::Infallible;

    impl<T> Length for Vec<T> {
        fn len(&self) -> usize {
            Vec::len(self)
        }
    }

    impl<T> Truncate for Vec<T> {
        fn truncate(&mut self, len: usize) {
            Vec::truncate(self, len);
        }
    }

    impl<A> TryExtend<A> for Vec<A> {
        type Error = Infallible;

        fn try_extend<T: IntoIterator<Item = A>>(&mut self, iter: T) -> Result<(), Infallible> {
            Vec::extend(self, iter);
            Ok(())
        }
    }
}
