#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

mod u64_le_utils;

/// Cursor for implementing sponge-based absorption and squeezing.
///
/// This type wraps `u8` and enforces that its value is always smaller than `RATE`.
///
/// `RATE` MUST be smaller than `256`, trying to initialize cursor with an invalid rate will
/// result in a compilation error.
#[derive(Debug, Clone)]
pub struct SpongeCursor<const RATE: usize> {
    pos: u8,
}

impl<const RATE: usize> Default for SpongeCursor<RATE> {
    fn default() -> Self {
        const {
            assert!(RATE != 0);
            assert!(RATE < u8::MAX as usize);
        }

        Self { pos: 0 }
    }
}

// Note that the methods should compile into a panic-free code,
// see: https://rust.godbolt.org/z/r93WE8zq3
impl<const RATE: usize> SpongeCursor<RATE> {
    /// Create new cursor with the provided position.
    ///
    /// Returns `None` if `pos` is bigger or equal to `RATE`.
    #[must_use]
    pub fn new(pos: u8) -> Option<Self> {
        if usize::from(pos) < RATE {
            Some(Self { pos })
        } else {
            None
        }
    }

    /// Get current cursor position as `u8`.
    #[must_use]
    #[inline(always)]
    #[allow(clippy::missing_panics_doc, reason = "the method is panic-free")]
    pub fn raw_pos(&self) -> u8 {
        let rate_u8 = u8::try_from(RATE).expect("RATE is smaller than 256");
        debug_assert!(self.pos < rate_u8);
        if self.pos < rate_u8 {
            self.pos
        } else {
            // SAFETY: the type enforces that `pos` is always smaller than `RATE`
            unsafe { core::hint::unreachable_unchecked() };
        }
    }

    /// Get current cursor position as `usize`.
    #[must_use]
    #[inline(always)]
    pub fn pos(&self) -> usize {
        let pos = usize::from(self.pos);
        debug_assert!(pos < RATE);
        if pos < RATE {
            pos
        } else {
            // SAFETY: the type enforces that `pos` is always smaller than `RATE`
            unsafe { core::hint::unreachable_unchecked() };
        }
    }

    /// Set new cursor position.
    ///
    /// # Panics
    /// If `new_pos` is greater or equal to `RATE`.
    fn set_pos(&mut self, new_pos: usize) {
        assert!(new_pos < RATE);
        self.pos = u8::try_from(new_pos).expect("`new_pos` is smaller than `RATE`");
    }

    /// Absorb bytes from `data` into `state` using little-endian byte order.
    ///
    /// Size of `state` in bytes MUST be greater or equal to `RATE`.
    /// Using an invalid `N` will result in a compilation error.
    #[allow(clippy::missing_panics_doc, reason = "the method is panic-free")]
    #[inline]
    pub fn absorb_u64_le<const N: usize>(
        &mut self,
        state: &mut [u64; N],
        sponge: fn(&mut [u64; N]),
        mut data: &[u8],
    ) {
        const {
            assert!(RATE <= size_of::<[u64; N]>());
            assert!(RATE < u8::MAX as usize);
            assert!(RATE % size_of::<u64>() == 0);
        };

        if self.pos != 0 {
            let pos = self.pos();
            let rem_len = RATE
                .checked_sub(pos)
                .expect("`pos` is always smaller than `RATE`");

            if data.len() < rem_len {
                u64_le_utils::absorb_partial::<N, RATE>(state, pos, data);
                self.set_pos(pos + data.len());
                return;
            }

            let (head, tail) = data.split_at(rem_len);
            data = tail;
            u64_le_utils::absorb_partial::<N, RATE>(state, pos, head);

            sponge(state);
        }

        let blocks = data.chunks_exact(RATE);
        let tail = blocks.remainder();

        for block in blocks {
            let block: &[u8; RATE] = block.try_into().expect("`block` has correct size");
            u64_le_utils::absorb_full(state, block);
            sponge(state);
        }

        if !tail.is_empty() {
            u64_le_utils::absorb_partial::<N, RATE>(state, 0, tail);
        }

        self.set_pos(tail.len());
    }

    /// Squeeze data from `state` by reading it into `buf` using little-endian byte order.
    ///
    /// Size of `state` in bytes MUST be greater or equal to `RATE`.
    /// Using an invalid `N` will result in a compilation error.
    #[inline]
    pub fn squeeze_read_u64_le<const N: usize>(
        &mut self,
        state: &mut [u64; N],
        sponge: fn(&mut [u64; N]),
        buf: &mut [u8],
    ) {
        self.squeeze_inner_u64_le(
            state,
            sponge,
            buf,
            u64_le_utils::squeeze_read_partial::<N, RATE>,
            u64_le_utils::squeeze_read_full,
        );
    }

    /// Squeeze data from `state` by XOR-ing it with data in `buf` using little-endian byte order.
    ///
    /// Size of `state` in bytes MUST be greater or equal to `RATE`.
    /// Using an invalid `N` will result in a compilation error.
    #[inline]
    pub fn squeeze_xor_u64_le<const N: usize>(
        &mut self,
        state: &mut [u64; N],
        sponge: fn(&mut [u64; N]),
        buf: &mut [u8],
    ) {
        self.squeeze_inner_u64_le(
            state,
            sponge,
            buf,
            u64_le_utils::squeeze_xor_partial::<N, RATE>,
            u64_le_utils::squeeze_xor_full,
        );
    }

    /// Squeeze data by calling custom functions using little-endian byte order.
    #[inline(always)]
    fn squeeze_inner_u64_le<const N: usize>(
        &mut self,
        state: &mut [u64; N],
        sponge: fn(&mut [u64; N]),
        mut buf: &mut [u8],
        process_partial: fn(&[u64; N], usize, &mut [u8]),
        process_full: fn(&[u64; N], &mut [u8; RATE]),
    ) {
        const {
            assert!(RATE <= size_of::<[u64; N]>());
            assert!(RATE < u8::MAX as usize);
            assert!(RATE % size_of::<u64>() == 0);
        };

        if self.pos != 0 {
            let pos = self.pos();
            let rem_len = RATE - pos;

            if buf.len() < rem_len {
                process_partial(state, pos, buf);
                self.set_pos(pos + buf.len());
                return;
            }

            let (head, tail) = buf.split_at_mut(rem_len);
            buf = tail;

            process_partial(state, pos, head);
        }

        let mut blocks = buf.chunks_exact_mut(RATE);

        for block in &mut blocks {
            sponge(state);
            let block = block.try_into().expect("`block` has correct size");
            process_full(state, block);
        }

        let tail = blocks.into_remainder();

        if !tail.is_empty() {
            sponge(state);
            process_partial(state, 0, tail);
        }

        self.set_pos(tail.len());
    }
}

#[cfg(feature = "zeroize")]
impl<const RATE: usize> zeroize::Zeroize for SpongeCursor<RATE> {
    fn zeroize(&mut self) {
        self.pos.zeroize();
    }
}
