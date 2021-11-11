use core::{marker::PhantomData};
use generic_array::{ArrayLength, GenericArray};
use crate::{InOut, InTmpOutBuf};

/// Custom pointer type which contains one immutable (input) and two mutable
/// (temporary and output) pointers.
///
/// Input and output pointers are either equal or non-overlapping. The
/// temporary pointer never overlaps with both input and output.
pub struct InTmpOut<'a, T> {
    pub(crate) in_ptr: *const T,
    pub(crate) tmp_ptr: *mut T,
    pub(crate) out_ptr: *mut T,
    pub(crate) _pd: PhantomData<(&'a T, &'a mut T)>,
}

impl<'a, T> InTmpOut<'a, T> {
    /// Create `InOut` from input and temporary parts.
    #[inline]
    pub fn inout_from_intmp(self) -> InOut<'a, T> {
        InOut {
            in_ptr: self.in_ptr,
            out_ptr: self.tmp_ptr,
            _pd: PhantomData,
        }
    }

    /// Create `InOut` from temporary part.
    #[inline]
    pub fn inout_from_tmptmp(self) -> InOut<'a, T> {
        InOut {
            in_ptr: self.tmp_ptr as *const T,
            out_ptr: self.tmp_ptr,
            _pd: PhantomData,
        }
    }

    /// Reborrow `self`.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> InTmpOut<'b, T> {
        Self {
            in_ptr: self.in_ptr,
            tmp_ptr: self.tmp_ptr,
            out_ptr: self.out_ptr,
            _pd: PhantomData,
        }
    }
}

impl<'a, T, N: ArrayLength<T>> InTmpOut<'a, GenericArray<T, N>> {
    /// Convert `InTmpOut` array to `InTmpOutBuf`.
    #[inline(always)]
    pub fn into_buf(self) -> InTmpOutBuf<'a, T> {
        InTmpOutBuf {
            in_ptr: self.in_ptr as *const T,
            tmp_ptr: self.tmp_ptr as *mut T,
            out_ptr: self.out_ptr as *mut T,
            len: N::USIZE,
            _pd: PhantomData,
        }
    }
}
