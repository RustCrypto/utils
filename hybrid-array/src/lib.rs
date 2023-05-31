#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
    clippy::implicit_saturating_sub,
    clippy::integer_arithmetic,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

pub use typenum;
pub use typenum::consts;

use core::{
    array::{IntoIter, TryFromSliceError},
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut, Index, IndexMut, Range},
    slice::{Iter, IterMut},
};
use typenum::Unsigned;

/// Hybrid typenum-based and const generic array type.
///
/// Provides the flexibility of typenum-based expressions while also
/// allowing interoperability and a transition path to const generics.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Array<T, U: ArraySize>(pub U::ArrayType<T>);

impl<T, U> Array<T, U>
where
    U: ArraySize,
{
    /// Create array where each array element `T` is returned by the `cb` call.
    pub fn from_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Self(ArrayExt::from_fn(cb))
    }

    /// Create array from a slice.
    pub fn from_slice(slice: &[T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy,
    {
        ArrayExt::from_slice(slice).map(Self)
    }

    /// Returns an iterator over the array.
    #[inline]
    fn iter(&self) -> Iter<'_, T> {
        self.as_ref().iter()
    }

    /// Returns an iterator that allows modifying each value.
    #[inline]
    fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.as_mut().iter_mut()
    }

    /// Returns a slice containing the entire array. Equivalent to `&s[..]`.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_ref()
    }

    /// Returns a mutable slice containing the entire array. Equivalent to `&mut s[..]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut()
    }

    /// Convert the given slice into a reference to a hybrid array.
    ///
    /// # Panics
    ///
    /// Panics if the slice's length doesn't match the array type.
    // TODO(tarcieri): deprecate this before the v0.2 release
    // #[deprecated(since = "0.2.0", note = "use TryFrom instead")]
    #[inline]
    pub fn ref_from_slice(slice: &[T]) -> &Self {
        slice.try_into().expect("slice length mismatch")
    }

    /// Convert the given mutable slice to a mutable reference to a hybrid array.
    ///
    /// # Panics
    ///
    /// Panics if the slice's length doesn't match the array type.
    // TODO(tarcieri): deprecate this before the v0.2 release
    // #[deprecated(since = "0.2.0", note = "use TryFrom instead")]
    #[inline]
    pub fn ref_from_mut_slice(slice: &mut [T]) -> &mut Self {
        slice.try_into().expect("slice length mismatch")
    }

    /// Clone the contents of the slice as a new hybrid array.
    ///
    /// # Panics
    ///
    /// Panics if the slice's length doesn't match the array type.
    // TODO(tarcieri): deprecate this before the v0.2 release
    // #[deprecated(since = "0.2.0", note = "use TryFrom instead")]
    #[inline]
    pub fn clone_from_slice(slice: &[T]) -> Self
    where
        Self: Clone,
    {
        Self::ref_from_slice(slice).clone()
    }
}

impl<T, U, const N: usize> AsRef<[T; N]> for Array<T, U>
where
    Self: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn as_ref(&self) -> &[T; N] {
        self.as_core_array()
    }
}

impl<T, U, const N: usize> AsMut<[T; N]> for Array<T, U>
where
    Self: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T; N] {
        self.as_mut_core_array()
    }
}

impl<T, U, const N: usize> Borrow<[T; N]> for Array<T, U>
where
    Self: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn borrow(&self) -> &[T; N] {
        self.as_core_array()
    }
}

impl<T, U, const N: usize> BorrowMut<[T; N]> for Array<T, U>
where
    Self: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T; N] {
        self.as_mut_core_array()
    }
}

impl<T, U> Default for Array<T, U>
where
    T: Default,
    U: ArraySize,
{
    fn default() -> Self {
        Self(ArrayExt::from_fn(|_| Default::default()))
    }
}

impl<T, U> Deref for Array<T, U>
where
    U: ArraySize,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T, U> DerefMut for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<T, U, const N: usize> From<[T; N]> for Array<T, U>
where
    Self: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn from(arr: [T; N]) -> Array<T, U> {
        Self::from_core_array(arr)
    }
}

impl<'a, T, U, const N: usize> From<&'a [T; N]> for &'a Array<T, U>
where
    Array<T, U>: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn from(array_ref: &'a [T; N]) -> &'a Array<T, U> {
        <Array<T, U>>::ref_from_core_array(array_ref)
    }
}

impl<'a, T, U, const N: usize> From<&'a mut [T; N]> for &'a mut Array<T, U>
where
    Array<T, U>: ArrayOps<T, N>,
    U: ArraySize,
{
    #[inline]
    fn from(array_ref: &'a mut [T; N]) -> &'a mut Array<T, U> {
        <Array<T, U>>::ref_from_mut_core_array(array_ref)
    }
}

impl<T, I, U> Index<I> for Array<T, U>
where
    [T]: Index<I>,
    U: ArraySize,
{
    type Output = <[T] as Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, I, U> IndexMut<I> for Array<T, U>
where
    [T]: IndexMut<I>,
    U: ArraySize,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<'a, T, U> TryFrom<&'a [T]> for Array<T, U>
where
    T: Copy,
    U: ArraySize,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a [T]) -> Result<Array<T, U>, TryFromSliceError> {
        ArrayExt::from_slice(slice).map(Self)
    }
}

impl<'a, T, U> TryFrom<&'a [T]> for &'a Array<T, U>
where
    U: ArraySize,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a [T]) -> Result<Self, TryFromSliceError> {
        check_slice_length::<T, U>(slice)?;

        // SAFETY: `Array<T, U>` is a `repr(transparent)` newtype for a core
        // array with length checked above.
        Ok(unsafe { *(slice.as_ptr() as *const Self) })
    }
}

impl<'a, T, U> TryFrom<&'a mut [T]> for &'a mut Array<T, U>
where
    U: ArraySize,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a mut [T]) -> Result<Self, TryFromSliceError> {
        check_slice_length::<T, U>(slice)?;

        // SAFETY: `Array<T, U>` is a `repr(transparent)` newtype for a core
        // array with length checked above.
        Ok(unsafe { *(slice.as_ptr() as *mut Self) })
    }
}

/// Generate a [`TryFromSliceError`] if the slice doesn't match the given length.
#[cfg_attr(debug_assertions, allow(clippy::panic_in_result_fn))]
fn check_slice_length<T, U: ArraySize>(slice: &[T]) -> Result<(), TryFromSliceError> {
    debug_assert_eq!(Array::<(), U>::default().len(), U::USIZE);

    if slice.len() != U::USIZE {
        // Hack: `TryFromSliceError` lacks a public constructor
        <&[T; 1]>::try_from([].as_slice())?;

        #[cfg(debug_assertions)]
        unreachable!();
    }

    Ok(())
}

/// Byte array type.
pub type ByteArray<U> = Array<u8, U>;

/// Array operations which are const generic over a given array size.
pub trait ArrayOps<T, const N: usize>:
    AsRef<[T; N]>
    + AsMut<[T; N]>
    + Borrow<[T; N]>
    + BorrowMut<[T; N]>
    + From<[T; N]>
    + Index<usize>
    + Index<Range<usize>>
    + IndexMut<usize>
    + IndexMut<Range<usize>>
    + IntoIterator
    + Sized
{
    /// Size of an array as a `usize`.
    ///
    /// Not to be confused with [`ArrayOps::Size`], which is `typenum`-based.
    const SIZE: usize;

    /// [`ArraySize`] type: `typenum`-provided [`Unsigned`] integer.
    ///
    /// Not to be confused with [`ArrayOps::SIZE`], which is a `usize`.
    type Size: ArraySize;

    /// Returns a reference to the inner array.
    fn as_core_array(&self) -> &[T; N];

    /// Returns a mutable reference to the inner array.
    fn as_mut_core_array(&mut self) -> &mut [T; N];

    /// Create array from Rust's core array type.
    fn from_core_array(arr: [T; N]) -> Self;

    /// Create array reference from reference to Rust's core array type.
    fn ref_from_core_array(arr: &[T; N]) -> &Self;

    /// Create mutable array reference from reference to Rust's core array type.
    fn ref_from_mut_core_array(arr: &mut [T; N]) -> &mut Self;

    /// Returns an array of the same size as `self`, with function `f` applied to each element
    /// in order.
    fn map_to_core_array<F, U>(self, f: F) -> [U; N]
    where
        F: FnMut(T) -> U;
}

/// Extension trait with helper functions for core arrays.
pub trait ArrayExt<T>: Sized {
    /// Create array using the given callback function for each element.
    fn from_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T;

    /// Create array from a slice, returning [`TryFromSliceError`] if the slice
    /// length does not match the array length.
    fn from_slice(slice: &[T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy;
}

impl<T, const N: usize> ArrayExt<T> for [T; N] {
    fn from_fn<F>(mut cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        let mut idx = 0;

        [(); N].map(|_| {
            let res = cb(idx);
            idx = idx.saturating_add(1); // TODO(tarcieri): better overflow handling?
            res
        })
    }

    fn from_slice(slice: &[T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy,
    {
        slice.try_into()
    }
}

/// Trait which associates a [`usize`] size and `ArrayType` with a
/// `typenum`-provided [`Unsigned`] integer.
///
/// # Safety
///
/// `ArrayType` MUST be an array with a number of elements exactly equal to
/// [`Unsigned::USIZE`].
///
/// Failure to so will cause undefined behavior.
///
/// NOTE: do not implement this trait yourself. It is implemented for types in
/// [`typenum::consts`].
pub unsafe trait ArraySize: Unsigned {
    /// Array type which corresponds to this size.
    type ArrayType<T>: AsRef<[T]> + AsMut<[T]> + IntoArray<T> + ArrayExt<T>;
}

/// Convert the given type into an [`Array`].
pub trait IntoArray<T> {
    /// Size of the [`Array`].
    type Size: ArraySize;

    /// Convert into the `hybrid-array` crate's [`Array`] type.
    fn into_hybrid_array(self) -> Array<T, Self::Size>;
}

macro_rules! impl_array_size {
    ($($len:expr => $ty:ident),+) => {
        $(
            impl<T> ArrayOps<T, $len> for Array<T, typenum::$ty> {
                const SIZE: usize = $len;
                type Size = typenum::$ty;

                #[inline]
                fn as_core_array(&self) -> &[T; $len] {
                    &self.0
                }

                #[inline]
                fn as_mut_core_array(&mut self) -> &mut [T; $len] {
                    &mut self.0
                }

                #[inline]
                fn from_core_array(arr: [T; $len]) -> Self {
                    Self(arr)
                }

                #[inline]
                fn ref_from_core_array(array_ref: &[T; $len]) -> &Self {
                    // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; $len]`
                    unsafe { &*(array_ref.as_ptr() as *const Self) }
                }

                #[inline]
                fn ref_from_mut_core_array(array_ref: &mut [T; $len]) -> &mut Self {
                    // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; $len]`
                    unsafe { &mut *(array_ref.as_mut_ptr() as *mut Self) }
                }

                #[inline]
                fn map_to_core_array<F, U>(self, f: F) -> [U; $len]
                where
                    F: FnMut(T) -> U
                {
                    self.0.map(f)
                }
            }

            unsafe impl ArraySize for typenum::$ty {
                type ArrayType<T> = [T; $len];
            }

            impl<T> IntoArray<T> for [T; $len] {
                type Size = typenum::$ty;

                fn into_hybrid_array(self) -> Array<T, Self::Size> {
                    Array::from_core_array(self)
                }
            }

            impl<T> IntoIterator for Array<T, typenum::$ty> {
                type Item = T;
                type IntoIter = IntoIter<T, $len>;

                /// Creates a consuming iterator, that is, one that moves each value out of
                /// the array (from start to end). The array cannot be used after calling
                /// this unless `T` implements `Copy`, so the whole array is copied.
                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_iter()
                }
            }

            impl<'a, T> IntoIterator for &'a Array<T, typenum::$ty> {
                type Item = &'a T;
                type IntoIter = Iter<'a, T>;

                fn into_iter(self) -> Iter<'a, T> {
                    self.iter()
                }
            }

            impl<'a, T> IntoIterator for &'a mut Array<T, typenum::$ty> {
                type Item = &'a mut T;
                type IntoIter = IterMut<'a, T>;

                #[inline]
                fn into_iter(self) -> IterMut<'a, T> {
                    self.iter_mut()
                }
            }
        )+
     };
}

impl_array_size! {
    0 => U0,
    1 => U1,
    2 => U2,
    3 => U3,
    4 => U4,
    5 => U5,
    6 => U6,
    7 => U7,
    8 => U8,
    9 => U9,
    10 => U10,
    11 => U11,
    12 => U12,
    13 => U13,
    14 => U14,
    15 => U15,
    16 => U16,
    17 => U17,
    18 => U18,
    19 => U19,
    20 => U20,
    21 => U21,
    22 => U22,
    23 => U23,
    24 => U24,
    25 => U25,
    26 => U26,
    27 => U27,
    28 => U28,
    29 => U29,
    30 => U30,
    31 => U31,
    32 => U32,
    33 => U33,
    34 => U34,
    35 => U35,
    36 => U36,
    37 => U37,
    38 => U38,
    39 => U39,
    40 => U40,
    41 => U41,
    42 => U42,
    43 => U43,
    44 => U44,
    45 => U45,
    46 => U46,
    47 => U47,
    48 => U48,
    49 => U49,
    50 => U50,
    51 => U51,
    52 => U52,
    53 => U53,
    54 => U54,
    55 => U55,
    56 => U56,
    57 => U57,
    58 => U58,
    59 => U59,
    60 => U60,
    61 => U61,
    62 => U62,
    63 => U63,
    64 => U64,
    96 => U96,
    128 => U128,
    192 => U192,
    256 => U256,
    384 => U384,
    448 => U448,
    512 => U512,
    768 => U768,
    896 => U896,
    1024 => U1024,
    2048 => U2048,
    4096 => U4096,
    8192 => U8192
}

#[cfg(test)]
mod tests {
    use super::ByteArray;
    use typenum::{U0, U3, U6, U7};

    #[test]
    fn tryfrom_slice_for_array_ref() {
        let slice: &[u8] = &[1, 2, 3, 4, 5, 6];
        assert!(ByteArray::<U0>::try_from(slice).is_err());
        assert!(ByteArray::<U3>::try_from(slice).is_err());

        let array_ref = ByteArray::<U6>::try_from(slice).unwrap();
        assert_eq!(&*array_ref, slice);

        assert!(ByteArray::<U7>::try_from(slice).is_err());
    }
}
