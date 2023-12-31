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
    clippy::arithmetic_side_effects,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

mod impls;

pub use typenum;
pub use typenum::consts;

use core::{
    array::TryFromSliceError,
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Add, Deref, DerefMut, Index, IndexMut, Range, Sub},
    ptr,
    slice::{self, Iter, IterMut},
};
use typenum::{Diff, Sum, Unsigned};

#[cfg(feature = "zeroize")]
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Hybrid typenum-based and const generic array type.
///
/// Provides the flexibility of typenum-based expressions while also
/// allowing interoperability and a transition path to const generics.
#[repr(transparent)]
pub struct Array<T, U: ArraySize>(pub U::ArrayType<T>);

type SplitResult<T, U, N> = (Array<T, N>, Array<T, Diff<U, N>>);
type SplitRefResult<'a, T, U, N> = (&'a Array<T, N>, &'a Array<T, Diff<U, N>>);
type SplitRefMutResult<'a, T, U, N> = (&'a mut Array<T, N>, &'a mut Array<T, Diff<U, N>>);

impl<T, U> Array<T, U>
where
    U: ArraySize,
{
    /// Create array where each array element `T` is returned by the `cb` call.
    pub fn from_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Self(FromFn::from_fn(cb))
    }

    /// Create array from a slice.
    // TODO(tarcieri): deprecate this before the v0.2 release
    // #[deprecated(since = "0.2.0", note = "use TryFrom instead")]
    pub fn from_slice(slice: &[T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy,
    {
        slice.try_into()
    }

    /// Returns an iterator over the array.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.as_ref().iter()
    }

    /// Returns an iterator that allows modifying each value.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
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

    /// Concatenates `self` with `other`.
    #[inline]
    pub fn concat<N>(self, other: Array<T, N>) -> Array<T, Sum<U, N>>
    where
        N: ArraySize,
        U: Add<N>,
        Sum<U, N>: ArraySize,
    {
        let mut result = MaybeUninit::uninit();
        let result_ptr = result.as_mut_ptr() as *mut Self;

        unsafe {
            ptr::write(result_ptr, self);
            ptr::write(result_ptr.add(1) as *mut _, other);
            result.assume_init()
        }
    }

    /// Splits `self` at index `N` in two arrays.
    ///
    /// New arrays hold the original memory from `self`.
    #[inline]
    pub fn split<N>(self) -> SplitResult<T, U, N>
    where
        U: Sub<N>,
        N: ArraySize,
        Diff<U, N>: ArraySize,
    {
        unsafe {
            let array = ManuallyDrop::new(self);
            let head = ptr::read(array.as_ptr() as *const _);
            let tail = ptr::read(array.as_ptr().add(N::USIZE) as *const _);
            (head, tail)
        }
    }

    /// Splits `&self` at index `N` in two array references.
    #[inline]
    pub fn split_ref<N>(&self) -> SplitRefResult<'_, T, U, N>
    where
        U: Sub<N>,
        N: ArraySize,
        Diff<U, N>: ArraySize,
    {
        unsafe {
            let array_ptr = self.as_ptr();
            let head = &*(array_ptr as *const _);
            let tail = &*(array_ptr.add(N::USIZE) as *const _);
            (head, tail)
        }
    }

    /// Splits `&mut self` at index `N` in two mutable array references.
    #[inline]
    pub fn split_ref_mut<N>(&mut self) -> SplitRefMutResult<'_, T, U, N>
    where
        U: Sub<N>,
        N: ArraySize,
        Diff<U, N>: ArraySize,
    {
        unsafe {
            let array_ptr = self.as_mut_ptr();
            let head = &mut *(array_ptr as *mut _);
            let tail = &mut *(array_ptr.add(N::USIZE) as *mut _);
            (head, tail)
        }
    }
}

impl<T, U> AsRef<[T]> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
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

impl<T, U> AsMut<[T]> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
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

impl<T, U> Clone for Array<T, U>
where
    T: Clone,
    U: ArraySize,
{
    fn clone(&self) -> Self {
        Self::from_fn(|n| self.0.as_ref()[n].clone())
    }
}

impl<T, U> Copy for Array<T, U>
where
    T: Copy,
    U: ArraySize,
    U::ArrayType<T>: Copy,
{
}

impl<T, U> Debug for Array<T, U>
where
    T: Debug,
    U: ArraySize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Array").field(&self.0.as_ref()).finish()
    }
}

impl<T, U> Default for Array<T, U>
where
    T: Default,
    U: ArraySize,
{
    fn default() -> Self {
        Self::from_fn(|_| Default::default())
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

impl<T, U> Eq for Array<T, U>
where
    T: Eq,
    U: ArraySize,
{
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

impl<T, U, const N: usize> From<Array<T, U>> for [T; N]
where
    Array<T, U>: ArrayOps<T, N>,
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(arr: Array<T, U>) -> [T; N] {
        arr.0
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

impl<T, U> Hash for Array<T, U>
where
    T: Hash,
    U: ArraySize,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
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

impl<T, U> IntoIterator for Array<T, U>
where
    U: ArraySize,
{
    type Item = T;
    type IntoIter = <U::ArrayType<T> as IntoIterator>::IntoIter;

    /// Creates a consuming iterator, that is, one that moves each value out of
    /// the array (from start to end). The array cannot be used after calling
    /// this unless `T` implements `Copy`, so the whole array is copied.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, U> IntoIterator for &'a Array<T, U>
where
    U: ArraySize,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, U> IntoIterator for &'a mut Array<T, U>
where
    U: ArraySize,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T, U> PartialEq for Array<T, U>
where
    T: PartialEq,
    U: ArraySize,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().eq(other.0.as_ref())
    }
}

impl<T, U> PartialOrd for Array<T, U>
where
    T: PartialOrd,
    U: ArraySize,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.as_ref().partial_cmp(other.0.as_ref())
    }
}

impl<T, U> Ord for Array<T, U>
where
    T: Ord,
    U: ArraySize,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
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
        check_slice_length::<T, U>(slice)?;
        Ok(Self::from_fn(|n| slice[n]))
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
        Ok(unsafe { &*(slice.as_ptr() as *const Array<T, U>) })
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
        Ok(unsafe { &mut *(slice.as_ptr() as *mut Array<T, U>) })
    }
}

#[cfg(feature = "zeroize")]
impl<T, U> Zeroize for Array<T, U>
where
    T: Zeroize,
    U: ArraySize,
{
    fn zeroize(&mut self) {
        self.0.as_mut().iter_mut().zeroize()
    }
}

#[cfg(feature = "zeroize")]
impl<T, U> ZeroizeOnDrop for Array<T, U>
where
    T: ZeroizeOnDrop,
    U: ArraySize,
{
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

/// Array operations which are const generic over a given array size.
pub trait ArrayOps<T, const N: usize>:
    AssociatedArraySize
    + Borrow<[T; N]>
    + BorrowMut<[T; N]>
    + From<[T; N]>
    + FromFn<T>
    + Into<[T; N]>
    + IntoIterator<Item = T>
    + Sized
    + SliceOps<T>
{
    /// Size of an array as a `usize`.
    ///
    /// Not to be confused with [`AssociatedArraySize::Size`], which is [`typenum`]-based.
    const SIZE: usize;

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

impl<T, const N: usize> ArrayOps<T, N> for [T; N]
where
    Self: AssociatedArraySize,
{
    const SIZE: usize = N;

    #[inline]
    fn as_core_array(&self) -> &[T; N] {
        self
    }

    #[inline]
    fn as_mut_core_array(&mut self) -> &mut [T; N] {
        self
    }

    #[inline]
    fn from_core_array(arr: [T; N]) -> Self {
        arr
    }

    #[inline]
    fn ref_from_core_array(array_ref: &[T; N]) -> &Self {
        array_ref
    }

    #[inline]
    fn ref_from_mut_core_array(array_ref: &mut [T; N]) -> &mut Self {
        array_ref
    }

    #[inline]
    fn map_to_core_array<F, U>(self, f: F) -> [U; N]
    where
        F: FnMut(T) -> U,
    {
        self.map(f)
    }
}

/// Slice operations which don't have access to a const generic array size.
pub trait SliceOps<T>:
    AsRef<[T]>
    + AsMut<[T]>
    + Index<usize>
    + Index<Range<usize>>
    + IndexMut<usize>
    + IndexMut<Range<usize>>
{
    /// Splits the shared slice into a slice of `N`-element arrays.
    ///
    /// See [`slice_as_chunks`] for more information.
    fn as_array_chunks<N: ArraySize>(&self) -> (&[Array<T, N>], &[T]) {
        slice_as_chunks(self.as_ref())
    }

    /// Splits the exclusive slice into a slice of `N`-element arrays.
    ///
    /// See [`slice_as_chunks_mut`] for more information.
    fn as_array_chunks_mut<N: ArraySize>(&mut self) -> (&mut [Array<T, N>], &mut [T]) {
        slice_as_chunks_mut(self.as_mut())
    }
}

impl<T> SliceOps<T> for [T] {}
impl<T, const N: usize> SliceOps<T> for [T; N] {}
impl<T, U: ArraySize> SliceOps<T> for Array<T, U> {}

/// Trait which associates a [`usize`] size and `ArrayType` with a
/// `typenum`-provided [`Unsigned`] integer.
///
/// # Safety
///
/// `ArrayType` MUST be an array with a number of elements exactly equal to
/// [`Unsigned::USIZE`]. Breaking this requirement will cause undefined behavior.
///
/// NOTE: This trait is effectively sealed and can not be implemented by third-party crates.
/// It is implemented only for a number of types defined in [`typenum::consts`].
pub unsafe trait ArraySize: Unsigned {
    /// Array type which corresponds to this size.
    type ArrayType<T>: AssociatedArraySize<Size = Self>
        + From<Array<T, Self>>
        + FromFn<T>
        + Into<Array<T, Self>>
        + IntoIterator<Item = T>
        + SliceOps<T>;
}

/// Associates an [`ArraySize`] with a given type.
pub trait AssociatedArraySize: Sized {
    /// Size of an array type, expressed as a [`typenum`]-based [`ArraySize`].
    type Size: ArraySize;
}

/// Construct an array type from the given function.
pub trait FromFn<T>: Sized {
    /// Create array using the given callback function for each element.
    fn from_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T;
}

impl<T, U> FromFn<T> for Array<T, U>
where
    U: ArraySize,
{
    fn from_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Array::from_fn(cb)
    }
}

impl<T, const N: usize> FromFn<T> for [T; N] {
    fn from_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        core::array::from_fn(cb)
    }
}

/// Splits the shared slice into a slice of `N`-element arrays, starting at the beginning
/// of the slice, and a remainder slice with length strictly less than `N`.
///
/// # Panics
/// Panics if `N` is 0.
#[allow(clippy::arithmetic_side_effects)]
pub fn slice_as_chunks<T, N: ArraySize>(buf: &[T]) -> (&[Array<T, N>], &[T]) {
    assert!(N::USIZE != 0, "chunk size must be non-zero");
    // Arithmetic safety: we have checked that `N::USIZE` is not zero, thus
    // division always returns correct result. `tail_pos` can not be bigger than `buf.len()`,
    // thus overflow on multiplication and underflow on substraction are impossible.
    let chunks_len = buf.len() / N::USIZE;
    let tail_pos = N::USIZE * chunks_len;
    let tail_len = buf.len() - tail_pos;
    unsafe {
        let ptr = buf.as_ptr();
        let chunks = slice::from_raw_parts(ptr as *const Array<T, N>, chunks_len);
        let tail = slice::from_raw_parts(ptr.add(tail_pos), tail_len);
        (chunks, tail)
    }
}

/// Splits the exclusive slice into a slice of `N`-element arrays, starting at the beginning
/// of the slice, and a remainder slice with length strictly less than `N`.
///
/// # Panics
/// Panics if `N` is 0.
#[allow(clippy::arithmetic_side_effects)]
pub fn slice_as_chunks_mut<T, N: ArraySize>(buf: &mut [T]) -> (&mut [Array<T, N>], &mut [T]) {
    assert!(N::USIZE != 0, "chunk size must be non-zero");
    // Arithmetic safety: we have checked that `N::USIZE` is not zero, thus
    // division always returns correct result. `tail_pos` can not be bigger than `buf.len()`,
    // thus overflow on multiplication and underflow on substraction are impossible.
    let chunks_len = buf.len() / N::USIZE;
    let tail_pos = N::USIZE * chunks_len;
    let tail_len = buf.len() - tail_pos;
    unsafe {
        let ptr = buf.as_mut_ptr();
        let chunks = slice::from_raw_parts_mut(ptr as *mut Array<T, N>, chunks_len);
        let tail = slice::from_raw_parts_mut(ptr.add(tail_pos), tail_len);
        (chunks, tail)
    }
}
