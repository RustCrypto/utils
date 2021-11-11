
use core::{marker::PhantomData, ptr};
use generic_array::{ArrayLength, GenericArray};
use crate::{InTmpOut, InOutBuf};


/// Custom pointer type which contains one immutable (input) and one mutable
/// (output) pointer, which are either equal or non-overlapping.
pub struct InOut<'a, T> {
    pub(crate) in_ptr: *const T,
    pub(crate) out_ptr: *mut T,
    pub(crate) _pd: PhantomData<(&'a T, &'a mut T)>,
}

impl<'a, T> InOut<'a, T> {
    /// Reborrow `self`.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> InOut<'b, T> {
        Self {
            in_ptr: self.in_ptr,
            out_ptr: self.out_ptr,
            _pd: PhantomData,
        }
    }

    /// Get immutable reference to the input value.
    #[inline]
    pub fn get_in(&self) -> &'a T {
        unsafe { &*self.in_ptr }
    }

    /// Get mutable reference to the output value.
    #[inline]
    pub fn get_out(self) -> &'a mut T {
        unsafe { &mut *self.out_ptr }
    }

    /// Convert `self` to a pair of raw input and output pointers.
    #[inline]
    pub fn into_raw(self) -> (*const T, *mut T) {
        (self.in_ptr, self.out_ptr)
    }

    /// Extend `self` with a temporary pointer.
    #[inline(always)]
    pub fn extend_with_tmp(self, tmp: &'a mut T) -> InTmpOut<'a, T> {
        InTmpOut {
            in_ptr: self.in_ptr,
            tmp_ptr: tmp as *mut T,
            out_ptr: self.out_ptr,
            _pd: PhantomData,
        }
    }

    /// Create `InOut` from raw input and output pointers.
    ///
    /// # Safety
    /// Behavior is undefined if any of the following conditions are violated:
    /// - `in_ptr` must point to a properly initialized value of type `T` and
    /// must be valid for reads.
    /// - `out_ptr` must point to a properly initialized value of type `T` and
    /// must be valid for both reads and writes.
    /// - `in_ptr` and `out_ptr` must be either equal or non-overlapping.
    /// - If `in_ptr` and `out_ptr` are equal, then the memory referenced by
    /// them must not be accessed through any other pointer (not derived from
    /// the return value) for the duration of lifetime 'a. Both read and write
    /// accesses are forbidden.
    /// - If `in_ptr` and `out_ptr` are not equal, then the memory referenced by
    /// `out_ptr` must not be accessed through any other pointer (not derived from
    /// the return value) for the duration of lifetime 'a. Both read and write
    /// accesses are forbidden. The memory referenced by `in_ptr` must not be
    /// mutated for the duration of lifetime `'a`, except inside an `UnsafeCell`.
    #[inline]
    pub unsafe fn from_raw(in_ptr: *const T, out_ptr: *mut T) -> InOut<'a, T> {
        Self {
            in_ptr,
            out_ptr,
            _pd: PhantomData,
        }
    }
}

impl<'a, T> From<&'a mut T> for InOut<'a, T> {
    fn from(val: &'a mut T) -> Self {
        Self {
            in_ptr: val as *mut T as *const T,
            out_ptr: val as *mut T,
            _pd: PhantomData,
        }
    }
}

impl<'a, T> From<(&'a T, &'a mut T)> for InOut<'a, T> {
    fn from((in_val, out_val): (&'a T, &'a mut T)) -> Self {
        Self {
            in_ptr: in_val as *const T,
            out_ptr: out_val as *mut T,
            _pd: Default::default(),
        }
    }
}

impl<'a, T, N: ArrayLength<T>> InOut<'a, GenericArray<T, N>> {
    /// Returns `InOut` for the given position.
    ///
    /// # Panics
    /// If `pos` greater or equal to array length.
    pub fn get<'b>(&'b mut self, pos: usize) -> InOut<'b, T> {
        assert!(pos <= N::USIZE);
        unsafe {
            InOut {
                in_ptr: (self.in_ptr as *const T).add(pos),
                out_ptr: (self.out_ptr as *mut T).add(pos),
                _pd: PhantomData,
            }
        }
    }

    /// Convert `InOut` array to `InOutBuf`.
    #[inline(always)]
    pub fn into_buf(self) -> InOutBuf<'a, T> {
        InOutBuf {
            in_ptr: self.in_ptr as *const T,
            out_ptr: self.out_ptr as *mut T,
            len: N::USIZE,
            _pd: PhantomData,
        }
    }
}

impl<'a, N: ArrayLength<u8>> InOut<'a, GenericArray<u8, N>> {
    /// XOR `data` with values behind the input slice and write
    /// result to the output slice.
    ///
    /// # Panics
    /// If `data` length is not equal to the buffer length.
    #[inline]
    #[allow(clippy::needless_range_loop)]
    pub fn xor(&mut self, data: &[u8]) {
        assert_eq!(N::USIZE, data.len());
        unsafe {
            let input = ptr::read(self.in_ptr);
            let mut temp = GenericArray::<u8, N>::default();
            for i in 0..N::USIZE {
                temp[i] = input[i] ^ data[i]
            }
            ptr::write(self.out_ptr, temp);
        }
    }
}
