use crate::{InOutBuf, errors::OutIsTooSmallError};
use core::{marker::PhantomData, slice};

#[cfg(feature = "block-padding")]
use {
    crate::{InOut, errors::PadError},
    block_padding::Padding,
    hybrid_array::{Array, ArraySize},
};

/// Custom slice type which references one immutable (input) slice and one
/// mutable (output) slice. Input and output slices are either the same or
/// do not overlap. Length of the output slice is always equal or bigger than
/// length of the input slice.
pub struct InOutBufReserved<'inp, 'out, T> {
    in_ptr: *const T,
    out_ptr: *mut T,
    in_len: usize,
    out_len: usize,
    _pd: PhantomData<(&'inp T, &'out mut T)>,
}

impl<'a, T> InOutBufReserved<'a, 'a, T> {
    /// Crate [`InOutBufReserved`] from a single mutable slice.
    ///
    /// # Errors
    /// If `out` is too small.
    pub fn from_mut_slice(buf: &'a mut [T], msg_len: usize) -> Result<Self, OutIsTooSmallError> {
        if msg_len > buf.len() {
            return Err(OutIsTooSmallError);
        }
        let p = buf.as_mut_ptr();
        let out_len = buf.len();
        Ok(Self {
            in_ptr: p,
            out_ptr: p,
            in_len: msg_len,
            out_len,
            _pd: PhantomData,
        })
    }
}

impl<T> InOutBufReserved<'_, '_, T> {
    /// Create [`InOutBufReserved`] from raw input and output pointers.
    ///
    /// # Safety
    /// Behavior is undefined if any of the following conditions are violated:
    /// - `in_ptr` must point to a properly initialized value of type `T` and
    ///   must be valid for reads for `in_len * mem::size_of::<T>()` many bytes.
    /// - `out_ptr` must point to a properly initialized value of type `T` and
    ///   must be valid for both reads and writes for `out_len * mem::size_of::<T>()`
    ///   many bytes.
    /// - `in_ptr` and `out_ptr` must be either equal or non-overlapping.
    /// - If `in_ptr` and `out_ptr` are equal, then the memory referenced by
    ///   them must not be accessed through any other pointer (not derived from
    ///   the return value) for the duration of lifetime 'a. Both read and write
    ///   accesses are forbidden.
    /// - If `in_ptr` and `out_ptr` are not equal, then the memory referenced by
    ///   `out_ptr` must not be accessed through any other pointer (not derived from
    ///   the return value) for the duration of lifetime 'a. Both read and write
    ///   accesses are forbidden. The memory referenced by `in_ptr` must not be
    ///   mutated for the duration of lifetime `'a`, except inside an `UnsafeCell`.
    /// - The total size `in_len * mem::size_of::<T>()` and
    ///   `out_len * mem::size_of::<T>()`  must be no larger than `isize::MAX`.
    #[inline(always)]
    pub unsafe fn from_raw(
        in_ptr: *const T,
        in_len: usize,
        out_ptr: *mut T,
        out_len: usize,
    ) -> Self {
        Self {
            in_ptr,
            out_ptr,
            in_len,
            out_len,
            _pd: PhantomData,
        }
    }

    /// Get raw input and output pointers.
    #[inline(always)]
    #[must_use]
    pub fn into_raw(self) -> (*const T, *mut T) {
        (self.in_ptr, self.out_ptr)
    }

    /// Get input buffer length.
    #[inline(always)]
    #[must_use]
    pub fn get_in_len(&self) -> usize {
        self.in_len
    }

    /// Get output buffer length.
    #[inline(always)]
    #[must_use]
    pub fn get_out_len(&self) -> usize {
        self.out_len
    }

    /// Split buffer into `InOutBuf` with input length and mutable slice pointing to
    /// the remaining reserved suffix.
    pub fn split_reserved(&mut self) -> (InOutBuf<'_, '_, T>, &mut [T]) {
        let in_len = self.get_in_len();
        let out_len = self.get_out_len();
        let in_ptr = self.get_in().as_ptr();
        let out_ptr = self.get_out().as_mut_ptr();
        // This never underflows because the type ensures that `out_len` is
        // bigger or equal to `in_len`.
        let tail_len = out_len - in_len;
        unsafe {
            let body = InOutBuf::from_raw(in_ptr, out_ptr, in_len);
            let tail = slice::from_raw_parts_mut(out_ptr.add(in_len), tail_len);
            (body, tail)
        }
    }
}

impl<'inp, 'out, T> InOutBufReserved<'inp, 'out, T> {
    /// Crate [`InOutBufReserved`] from two separate slices.
    ///
    /// # Errors
    /// If `out` is too small.
    pub fn from_slices(
        in_buf: &'inp [T],
        out_buf: &'out mut [T],
    ) -> Result<Self, OutIsTooSmallError> {
        if in_buf.len() > out_buf.len() {
            return Err(OutIsTooSmallError);
        }
        Ok(Self {
            in_ptr: in_buf.as_ptr(),
            out_ptr: out_buf.as_mut_ptr(),
            in_len: in_buf.len(),
            out_len: out_buf.len(),
            _pd: PhantomData,
        })
    }

    /// Get input slice.
    #[inline(always)]
    #[must_use]
    pub fn get_in(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.in_ptr, self.in_len) }
    }

    /// Get output slice.
    #[inline(always)]
    pub fn get_out(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.out_ptr, self.out_len) }
    }

    /// Consume `self` and get output slice with lifetime `'out`.
    #[inline(always)]
    #[must_use]
    pub fn into_out(self) -> &'out mut [T] {
        unsafe { slice::from_raw_parts_mut(self.out_ptr, self.out_len) }
    }
}

#[cfg(feature = "block-padding")]
impl<'inp, 'out> InOutBufReserved<'inp, 'out, u8> {
    /// Transform buffer into [`PaddedInOutBuf`] using padding algorithm `P`.
    ///
    /// # Errors
    /// If the padding is invalid
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    pub fn into_padded_blocks<P, BS>(self) -> Result<PaddedInOutBuf<'inp, 'out, BS>, PadError>
    where
        P: Padding,
        BS: ArraySize,
    {
        let bs = BS::USIZE;
        let blocks_len = self.in_len / bs;

        use block_padding::PaddedData;
        let (blocks, tail_block) = match P::pad_detached(self.get_in()) {
            PaddedData::Pad { blocks, tail_block } => (blocks, Some(tail_block)),
            PaddedData::NoPad { blocks } => (blocks, None),
            PaddedData::Error => return Err(PadError),
        };

        assert_eq!(blocks.len(), blocks_len);

        let out_len = self.out_len;
        let (in_ptr, out_ptr) = self.into_raw();

        let blocks = unsafe {
            InOutBuf::from_raw(
                in_ptr.cast::<Array<u8, BS>>(),
                out_ptr.cast::<Array<u8, BS>>(),
                blocks_len,
            )
        };

        let Some(tail_block) = tail_block else {
            let tail_inout = None;
            return Ok(PaddedInOutBuf { blocks, tail_inout });
        };

        let blocks_byte_len = blocks_len * bs;
        let reserve_len = out_len - blocks_byte_len;
        if reserve_len < tail_block.len() {
            return Err(PadError);
        }
        // SAFETY: we checked that the out buffer has enough bytes in reserve
        let tail_out: &mut Array<u8, BS> = unsafe {
            let tail_out_ptr = out_ptr.add(blocks_byte_len);
            &mut *(tail_out_ptr.cast())
        };

        let tail_inout = Some((tail_block, tail_out));

        Ok(PaddedInOutBuf { blocks, tail_inout })
    }
}

/// Variant of [`InOutBuf`] with optional padded tail block.
#[cfg(feature = "block-padding")]
#[allow(clippy::type_complexity)]
pub struct PaddedInOutBuf<'inp, 'out, BS: ArraySize> {
    blocks: InOutBuf<'inp, 'out, Array<u8, BS>>,
    tail_inout: Option<(Array<u8, BS>, &'out mut Array<u8, BS>)>,
}

#[cfg(feature = "block-padding")]
impl<'out, BS: ArraySize> PaddedInOutBuf<'_, 'out, BS> {
    /// Get full blocks.
    #[inline(always)]
    pub fn get_blocks(&mut self) -> InOutBuf<'_, '_, Array<u8, BS>> {
        self.blocks.reborrow()
    }

    /// Get padded tail block.
    ///
    /// Most padding implementations always return `Some`.
    #[inline(always)]
    pub fn get_tail_block(&mut self) -> Option<InOut<'_, '_, Array<u8, BS>>> {
        self.tail_inout.as_mut().map(|(in_block, out_block)| {
            let in_block = &*in_block;
            let out_block = &mut **out_block;
            InOut::from((in_block, out_block))
        })
    }

    /// Convert buffer into output slice.
    #[inline(always)]
    pub fn into_out(self) -> &'out [u8] {
        let total_blocks = if self.tail_inout.is_some() {
            self.blocks.len() + 1
        } else {
            self.blocks.len()
        };
        let res_len = BS::USIZE * total_blocks;
        let (_, out_ptr) = self.blocks.into_raw();
        // SAFETY: `res_len` is always valid for the output buffer since
        // it's checked during type construction
        unsafe { slice::from_raw_parts(out_ptr as *const u8, res_len) }
    }
}
