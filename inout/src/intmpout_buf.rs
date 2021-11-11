use core::{marker::PhantomData, ptr, slice};
use generic_array::{ArrayLength, GenericArray};
use crate::InOutBuf;

/// Custom slice type which references one immutable (input) slice and two
/// mutable (temporary and output) slices of equal length. Input and output
/// slices are either the same or do not overlap. The temporary slice never
/// overlaps with both input and output slices.
pub struct InTmpOutBuf<'a, T> {
    pub(crate) in_ptr: *const T,
    pub(crate) tmp_ptr: *mut T,
    pub(crate) out_ptr: *mut T,
    pub(crate) len: usize,
    pub(crate) _pd: PhantomData<(&'a T, &'a mut T)>,
}

impl<'a, T> InTmpOutBuf<'a, T> {
    /// Create `InTmpOutBuf` from raw input and output pointers.
    ///
    /// # Safety
    /// Behavior is undefined if any of the following conditions are violated:
    /// - `in_ptr` must point to a properly initialized value of type `T` and
    /// must be valid for reads for `len * mem::size_of::<T>()` many bytes.
    /// - `tmp_ptr` and `out_ptr` must point to a properly initialized value of type `T` and
    /// must be valid for both reads and writes for `len * mem::size_of::<T>()`
    /// many bytes.
    /// - `in_ptr` and `out_ptr` must be either equal or non-overlapping.
    /// - The memory referenced by `tmp_ptr` must not be accessed through any
    /// other pointer (not derived from the return value) for the duration of
    /// lifetime 'a. Both read and write accesses are forbidden.
    /// - If `in_ptr` and `out_ptr` are equal, then the memory referenced by
    /// them must not be accessed through any other pointer (not derived from
    /// the return value) for the duration of lifetime 'a. Both read and write
    /// accesses are forbidden.
    /// - If `in_ptr` and `out_ptr` are not equal, then the memory referenced by
    /// `out_ptr` must not be accessed through any other pointer (not derived from
    /// the return value) for the duration of lifetime 'a. Both read and write
    /// accesses are forbidden. The memory referenced by `in_ptr` must not be
    /// mutated for the duration of lifetime `'a`, except inside an `UnsafeCell`.
    /// - The total size `len * mem::size_of::<T>()`  must be no larger than `isize::MAX`.
    #[inline]
    pub unsafe fn from_raw(in_ptr: *const T, tmp_ptr: *mut T, out_ptr: *mut T, len: usize) -> Self {
        Self {
            in_ptr,
            tmp_ptr,
            out_ptr,
            len,
            _pd: PhantomData,
        }
    }

    /// Get length of the inner buffers.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reborrow `self`.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> InTmpOutBuf<'b, T> {
        Self {
            in_ptr: self.in_ptr,
            tmp_ptr: self.tmp_ptr,
            out_ptr: self.out_ptr,
            len: self.len,
            _pd: PhantomData,
        }
    }

    /// Get input slice.
    #[inline]
    pub fn get_in(&self) -> &'a [T] {
        unsafe { slice::from_raw_parts(self.in_ptr, self.len) }
    }

    /// Get temporary slice.
    #[inline]
    pub fn get_tmp(self) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(self.tmp_ptr, self.len) }
    }

    /// Get output slice.
    #[inline]
    pub fn get_out(self) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(self.out_ptr, self.len) }
    }

    /// Get input and temporary slices.
    pub fn get_in_tmp(self) -> (&'a [T], &'a mut [T]) {
        unsafe {
            (
                slice::from_raw_parts(self.in_ptr, self.len),
                slice::from_raw_parts_mut(self.tmp_ptr, self.len),
            )
        }
    }

    /// Copy data from temporary slice to output.
    #[inline]
    pub fn copy_tmp2out(&mut self) {
        unsafe { core::ptr::copy_nonoverlapping(self.tmp_ptr as *const T, self.out_ptr, self.len) }
    }

    /// Construct `InOutBuf` from input and temporary slices.
    #[inline]
    pub fn inout_from_intmp(self) -> InOutBuf<'a, T> {
        let (input, temp) = self.get_in_tmp();
        InOutBuf::new(input, temp).expect("Length of slices is equal")
    }

    /// Construct `InOutBuf` from temporary slice.
    #[inline]
    pub fn inout_from_tmptmp(self) -> InOutBuf<'a, T> {
        self.get_tmp().into()
    }
}

impl<'a, N: ArrayLength<u8>> InTmpOutBuf<'a, GenericArray<u8, N>> {
    /// XORs data fron input and temporary slices and writes result to output
    /// slice.
    #[inline(always)]
    pub fn xor_intmp2out(&mut self) {
        unsafe {
            let in_ptr = self.in_ptr;
            let tmp_ptr = self.tmp_ptr;
            let out_ptr = self.out_ptr;
            for i in 0..self.len() {
                let a = ptr::read(in_ptr.add(i));
                let b = ptr::read(tmp_ptr.add(i));
                let mut res = GenericArray::<u8, N>::default();
                for j in 0..N::USIZE {
                    res[j] = a[j] ^ b[j];
                }
                ptr::write(out_ptr.add(i), res);
            }
        }
    }
}
