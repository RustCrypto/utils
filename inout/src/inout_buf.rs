use core::{convert::TryInto, marker::PhantomData, slice};
use generic_array::{ArrayLength, GenericArray};
use crate::{InOut, InTmpOutBuf};

/// Custom slice type which references one immutable (input) slice and one
/// mutable (output) slice of equal length. Input and output slices are
/// either the same or do not overlap.
pub struct InOutBuf<'a, T> {
    pub(crate) in_ptr: *const T,
    pub(crate) out_ptr: *mut T,
    pub(crate) len: usize,
    pub(crate) _pd: PhantomData<(&'a T, &'a mut T)>,
}

impl<'a, T> From<&'a mut [T]> for InOutBuf<'a, T> {
    fn from(buf: &'a mut [T]) -> Self {
        Self {
            in_ptr: buf.as_ptr(),
            out_ptr: buf.as_mut_ptr(),
            len: buf.len(),
            _pd: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for InOutBuf<'a, T> {
    type Item = InOut<'a, T>;
    type IntoIter = InOutBufIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        InOutBufIter { buf: self, pos: 0 }
    }
}

impl<'a, T> InOutBuf<'a, T> {
    /// Create `InOutBuf` from a single mutable reference.
    pub fn from_mut(val: &mut T) -> Self {
        Self {
            in_ptr: val as *mut T as *const T,
            out_ptr: val as *mut T,
            len: 1,
            _pd: PhantomData,
        }
    }

    /// Create `InOutBuf` from a pair of immutable and mutable references.
    #[inline]
    pub fn from_refs(in_val: &'a T, out_val: &'a mut T) -> Self {
        Self {
            in_ptr: in_val as *const T,
            out_ptr: out_val as *mut T,
            len: 1,
            _pd: PhantomData,
        }
    }

    /// Create `InOutBuf` from immutable and mutable slices.
    ///
    /// Returns an error if length of slices is not equal to each other.
    #[inline]
    pub fn new(in_buf: &'a [T], out_buf: &'a mut [T]) -> Result<Self, NotEqualError> {
        if in_buf.len() != out_buf.len() {
            Err(NotEqualError)
        } else {
            Ok(Self {
                in_ptr: in_buf.as_ptr(),
                out_ptr: out_buf.as_mut_ptr(),
                len: in_buf.len(),
                _pd: Default::default(),
            })
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

    /// Returns `InOut` for given position.
    ///
    /// # Panics
    /// If `pos` greater or equal to buffer length.
    #[inline]
    pub fn get<'b>(&'b mut self, pos: usize) -> InOut<'b, T> {
        assert!(pos < self.len);
        unsafe {
            InOut {
                in_ptr: self.in_ptr.add(pos),
                out_ptr: self.out_ptr.add(pos),
                _pd: PhantomData,
            }
        }
    }

    /// Get input slice.
    #[inline]
    pub fn get_in(&self) -> &'a [T] {
        unsafe { slice::from_raw_parts(self.in_ptr, self.len) }
    }

    /// Get output slice.
    #[inline]
    pub fn get_out(self) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(self.out_ptr, self.len) }
    }

    /// Get raw input and output pointers.
    #[inline]
    pub fn into_raw(self) -> (*const T, *mut T) {
        (self.in_ptr, self.out_ptr)
    }

    /// Reborrow `self`.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> InOutBuf<'b, T> {
        Self {
            in_ptr: self.in_ptr,
            out_ptr: self.out_ptr,
            len: self.len,
            _pd: PhantomData,
        }
    }

    /// Create `InOutBuf` from raw input and output pointers.
    ///
    /// # Safety
    /// Behavior is undefined if any of the following conditions are violated:
    /// - `in_ptr` must point to a properly initialized value of type `T` and
    /// must be valid for reads for `len * mem::size_of::<T>()` many bytes.
    /// - `out_ptr` must point to a properly initialized value of type `T` and
    /// must be valid for both reads and writes for `len * mem::size_of::<T>()`
    /// many bytes.
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
    /// - The total size `len * mem::size_of::<T>()`  must be no larger than `isize::MAX`.
    #[inline]
    pub unsafe fn from_raw(in_ptr: *const T, out_ptr: *mut T, len: usize) -> InOutBuf<'a, T> {
        Self {
            in_ptr,
            out_ptr,
            len,
            _pd: PhantomData,
        }
    }

    /// Divides one buffer into two at `mid` index.
    ///
    /// The first will contain all indices from `[0, mid)` (excluding
    /// the index `mid` itself) and the second will contain all
    /// indices from `[mid, len)` (excluding the index `len` itself).
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    #[inline]
    pub fn split_at(self, mid: usize) -> (InOutBuf<'a, T>, InOutBuf<'a, T>) {
        assert!(mid <= self.len);
        let (tail_in_ptr, tail_out_ptr) = unsafe { (self.in_ptr.add(mid), self.out_ptr.add(mid)) };
        (
            InOutBuf {
                in_ptr: self.in_ptr,
                out_ptr: self.out_ptr,
                len: mid,
                _pd: PhantomData,
            },
            InOutBuf {
                in_ptr: tail_in_ptr,
                out_ptr: tail_out_ptr,
                len: self.len() - mid,
                _pd: PhantomData,
            },
        )
    }

    /// Partition buffer into 2 parts: buffer of arrays and tail.
    #[inline]
    pub fn into_chunks<N: ArrayLength<T>>(
        self,
    ) -> (InOutBuf<'a, GenericArray<T, N>>, InOutBuf<'a, T>) {
        let chunks = self.len() / N::USIZE;
        let tail_pos = N::USIZE * chunks;
        let tail_len = self.len() - tail_pos;
        unsafe {
            let chunks = InOutBuf {
                in_ptr: self.in_ptr as *const GenericArray<T, N>,
                out_ptr: self.out_ptr as *mut GenericArray<T, N>,
                len: chunks,
                _pd: PhantomData,
            };
            let tail = InOutBuf {
                in_ptr: self.in_ptr.add(tail_pos),
                out_ptr: self.out_ptr.add(tail_pos),
                len: tail_len,
                _pd: PhantomData,
            };
            (chunks, tail)
        }
    }

    /// Extend buffer with a temporary array.
    ///
    /// # Panics
    /// Of buffer length is greater than length of the temporary array.
    #[inline(always)]
    pub fn extend_with_tmp<N: ArrayLength<T>>(
        self,
        tmp: &'a mut GenericArray<T, N>,
    ) -> InTmpOutBuf<'a, T> {
        assert!(self.len <= tmp.len());
        InTmpOutBuf {
            in_ptr: self.in_ptr,
            tmp_ptr: tmp.as_mut_ptr(),
            out_ptr: self.out_ptr,
            len: self.len,
            _pd: PhantomData,
        }
    }

    /// Process data in buffer in chunks of size `N`.
    // TODO: decribe behavior and arguments in detail
    #[inline(always)]
    pub fn process_chunks<N, S, PRE, POST, PC, PT>(
        self,
        mut state: S,
        mut pre_fn: PRE,
        mut post_fn: POST,
        mut proc_chunk: PC,
        mut proc_tail: PT,
    ) where
        T: Default,
        N: ArrayLength<T>,
        PRE: FnMut(InTmpOutBuf<'_, T>) -> InSrc,
        POST: FnMut(InTmpOutBuf<'_, T>),
        PC: FnMut(&mut S, InOut<'_, GenericArray<T, N>>),
        PT: FnMut(&mut S, InOutBuf<'_, T>),
    {
        let (chunks, tail) = self.into_chunks::<N>();
        for chunk in chunks {
            let mut tmp = GenericArray::<T, N>::default();
            let mut buf = chunk.extend_with_tmp(&mut tmp);
            let chunk = match pre_fn(buf.reborrow().into_buf()) {
                InSrc::In => buf.reborrow().inout_from_intmp(),
                InSrc::Tmp => buf.reborrow().inout_from_tmptmp(),
            };
            proc_chunk(&mut state, chunk);
            post_fn(buf.into_buf());
        }

        if tail.is_empty() {
            return;
        }
        let mut tmp = GenericArray::<T, N>::default();
        let mut buf = tail.extend_with_tmp(&mut tmp);
        let tail = match pre_fn(buf.reborrow()) {
            InSrc::In => buf.reborrow().inout_from_intmp(),
            InSrc::Tmp => buf.reborrow().inout_from_tmptmp(),
        };
        proc_tail(&mut state, tail);
        post_fn(buf);
    }
}

impl<'a> InOutBuf<'a, u8> {
    /// XORs `data` with values behind the input slice and write
    /// result to the output slice.
    ///
    /// # Panics
    /// If `data` length is not equal to the buffer length.
    #[inline]
    #[allow(clippy::needless_range_loop)]
    pub fn xor(&mut self, data: &[u8]) {
        assert_eq!(self.len(), data.len());
        unsafe {
            for i in 0..data.len() {
                let in_ptr = self.in_ptr.add(i);
                let out_ptr = self.out_ptr.add(i);
                *out_ptr = *in_ptr ^ data[i];
            }
        }
    }
}

impl<'a, T, N> TryInto<InOut<'a, GenericArray<T, N>>> for InOutBuf<'a, T>
where
    N: ArrayLength<T>,
{
    type Error = IntoArrayError;

    fn try_into(self) -> Result<InOut<'a, GenericArray<T, N>>, Self::Error> {
        if self.len() == N::USIZE {
            Ok(InOut {
                in_ptr: self.in_ptr as *const _,
                out_ptr: self.out_ptr as *mut _,
                _pd: PhantomData,
            })
        } else {
            Err(IntoArrayError)
        }
    }
}

/// Iterator over [`InOutBuf`].
pub struct InOutBufIter<'a, T> {
    buf: InOutBuf<'a, T>,
    pos: usize,
}

impl<'a, T> Iterator for InOutBufIter<'a, T> {
    type Item = InOut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() == self.pos {
            return None;
        }
        let res = unsafe {
            InOut {
                in_ptr: self.buf.in_ptr.add(self.pos),
                out_ptr: self.buf.out_ptr.add(self.pos),
                _pd: PhantomData,
            }
        };
        self.pos += 1;
        Some(res)
    }
}

/// The enum which controls which slice to use from `InTmpOutBuf` as input.
pub enum InSrc {
    /// Use input slice as input.
    In,
    /// Use temporary slice as input.
    Tmp,
}

/// The error returned when slice can not be converted into array.
#[derive(Copy, Clone, Debug)]
pub struct IntoArrayError;

/// The error returned when input and output slices have different length
/// and thus can not be converted to `InOutBuf`.
#[derive(Copy, Clone, Debug)]
pub struct NotEqualError;
