#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

pub use hybrid_array as array;

use core::marker::PhantomData;
use hybrid_array::{Array, ArraySize};

/// Cursor for implementing sponge-based absorption and squeezing.
///
/// This type wraps `u8` and enforces that its value is always smaller than `Rate`.
///
/// `Rate` MUST be smaller than `U256`, trying to initialize cursor with an invalid rate will
/// result in a compilation error.
#[derive(Debug, Clone)]
pub struct SpongeCursor<Rate: ArraySize> {
    pos: u8,
    _pd: PhantomData<Rate>,
}

impl<Rate: ArraySize> Default for SpongeCursor<Rate> {
    fn default() -> Self {
        const {
            assert!(Rate::USIZE != 0);
            assert!(Rate::USIZE < u8::MAX as usize);
        }

        Self {
            pos: 0,
            _pd: PhantomData,
        }
    }
}

impl<Rate: ArraySize> SpongeCursor<Rate> {
    /// Create new cursor with the provided position.
    ///
    /// Returns `None` if `pos` is bigger or equal to `Rate`.
    #[must_use]
    pub fn new(pos: u8) -> Option<Self> {
        if usize::from(pos) < Rate::USIZE {
            Some(Self {
                pos,
                _pd: PhantomData,
            })
        } else {
            None
        }
    }

    /// Absorb bytes from `data` into a `u64`-based state using little ednian byte order.
    ///
    /// Size of state MUST be greater or equal to `Rate`. Using an invalid `N` will result in
    /// a compilation error.
    #[allow(
        clippy::missing_panics_doc,
        reason = "the function is panic-free, see: https://rust.godbolt.org/z/WdordTqvv"
    )]
    #[inline]
    pub fn absorb_u64_le<const N: usize>(
        &mut self,
        state: &mut [u64; N],
        sponge: fn(&mut [u64; N]),
        mut data: &[u8],
    ) {
        const {
            assert!(Rate::USIZE <= size_of::<[u64; N]>());
            assert!(Rate::USIZE < u8::MAX as usize);
            assert!(Rate::USIZE % size_of::<u64>() == 0);
        };

        if self.pos != 0 {
            let pos = self.pos();
            let rem_len = Rate::USIZE - pos;

            let is_partial = data.len() < rem_len;
            let head = if is_partial {
                data
            } else {
                let (head, tail) = data.split_at(rem_len);
                data = tail;
                head
            };

            xor_in::<N, Rate>(state, pos, head);

            if is_partial {
                self.pos = u8::try_from(pos + head.len()).expect("the sum is smaller than Rate");
                return;
            }

            sponge(state);
        }

        let blocks = data.chunks_exact(Rate::USIZE);
        let tail = blocks.remainder();

        for block in blocks {
            let chunks = block.chunks_exact(size_of::<u64>());
            assert!(chunks.remainder().is_empty());

            for (dst, chunk) in state.iter_mut().zip(chunks) {
                let chunk = chunk.try_into().expect("chunk has correct length");
                *dst ^= u64::from_le_bytes(chunk);
            }

            sponge(state);
        }

        if !tail.is_empty() {
            xor_in::<N, Rate>(state, 0, tail);
        }

        self.pos = u8::try_from(tail.len()).expect("tail.len() is smaller than Rate");
    }

    /// Squeeze data into `dst`.
    #[allow(clippy::missing_panics_doc, reason = "the function is panic-free")]
    pub fn squeeze_u64_le<const N: usize>(
        &mut self,
        state: &mut [u64; N],
        sponge: fn(&mut [u64; N]),
        mut dst: &mut [u8],
    ) {
        const {
            assert!(Rate::USIZE <= size_of::<[u64; N]>());
            assert!(Rate::USIZE < u8::MAX as usize);
            assert!(Rate::USIZE % size_of::<u64>() == 0);
        };

        if self.pos != 0 {
            let pos = self.pos();
            let rem_len = Rate::USIZE - pos;

            let is_partial = dst.len() < rem_len;
            let split_pos = if is_partial { dst.len() } else { rem_len };

            let (head, tail) = dst.split_at_mut(split_pos);
            dst = tail;

            read_into::<N, Rate>(state, pos, head);

            if is_partial {
                self.pos = u8::try_from(pos + head.len()).expect("the sum is smaller than Rate");
                return;
            }
        }

        let mut blocks = dst.chunks_exact_mut(Rate::USIZE);

        for block in &mut blocks {
            sponge(state);

            let mut dst_chunks = block.chunks_exact_mut(size_of::<u64>());
            for (src, dst_chunk) in state.iter_mut().zip(&mut dst_chunks) {
                dst_chunk.copy_from_slice(&src.to_le_bytes());
            }
            assert!(dst_chunks.into_remainder().is_empty());
        }

        let tail = blocks.into_remainder();

        if !tail.is_empty() {
            sponge(state);
            read_into::<N, Rate>(state, 0, tail);
        }

        self.pos = u8::try_from(tail.len()).expect("tail.len() is smaller than Rate");
    }

    /// Get current cursor position.
    #[must_use]
    #[inline(always)]
    pub fn pos(&self) -> usize {
        let pos = usize::from(self.pos);
        debug_assert!(pos < Rate::USIZE);
        if pos < Rate::USIZE {
            pos
        } else {
            // SAFETY: the type enforces that `pos` is always smaller than `Rate`
            unsafe { core::hint::unreachable_unchecked() };
        }
    }
}

#[cfg(feature = "zeroize")]
impl<Rate: ArraySize> zeroize::Zeroize for SpongeCursor<Rate> {
    fn zeroize(&mut self) {
        self.pos.zeroize();
    }
}

#[inline(always)]
fn xor_in<const N: usize, Rate: ArraySize>(state: &mut [u64; N], offset: usize, data: &[u8]) {
    const {
        assert!(size_of::<Array<u8, Rate>>() <= size_of::<[u64; N]>());
        assert!(Rate::USIZE % size_of::<u64>() == 0);
    };

    if cfg!(target_endian = "little") {
        // SAFETY: casting of `&mut [u64; N]` into `&mut [u8; M]` is safe if
        // `size_of::<[u8; M]>() <= size_of::<[u64; N]>())`,
        // the same applies to `Array` since it's a transparent wrapper around `[T; N]`
        let sub_state: &mut Array<u8, Rate> = unsafe { &mut *(state.as_mut_ptr().cast()) };

        let dst = &mut sub_state[offset..][..data.len()];

        for i in 0..dst.len() {
            dst[i] ^= data[i];
        }
    } else {
        let mut buf = Array::<u8, Rate>::default();
        buf[offset..][..data.len()].copy_from_slice(data);

        let chunks = buf.chunks_exact(size_of::<u64>());
        assert!(chunks.remainder().is_empty());

        for (dst, chunk) in state.iter_mut().zip(chunks) {
            let chunk = chunk.try_into().expect("chunk has correct length");
            *dst ^= u64::from_le_bytes(chunk);
        }
    }
}

#[inline(always)]
fn read_into<const N: usize, Rate: ArraySize>(state: &[u64; N], offset: usize, dst: &mut [u8]) {
    const {
        assert!(size_of::<Array<u8, Rate>>() <= size_of::<[u64; N]>());
        assert!(Rate::USIZE % size_of::<u64>() == 0);
    };

    if cfg!(target_endian = "little") {
        // SAFETY: casting of `&mut [u64; N]` into `&mut [u8; M]` is safe if
        // `size_of::<[u8; M]>() <= size_of::<[u64; N]>())`,
        // the same applies to `Array` since it's a transparent wrapper around `[T; N]`
        let sub_state: &Array<u8, Rate> = unsafe { &*(state.as_ptr().cast()) };
        dst.copy_from_slice(&sub_state[offset..][..dst.len()]);
    } else {
        let mut buf = Array::<u8, Rate>::default();
        let mut chunks = buf.chunks_exact_mut(size_of::<u64>());
        for (src, dst) in state.iter().zip(&mut chunks) {
            dst.copy_from_slice(&src.to_le_bytes());
        }
        assert!(chunks.into_remainder().is_empty());
        dst.copy_from_slice(&buf[offset..][..dst.len()]);
    }
}
