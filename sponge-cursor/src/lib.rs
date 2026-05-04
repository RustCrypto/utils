#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

mod utils;

/// Cursor for implementing sponge-based absorption and squeezing.
///
/// This type wraps `u8` and enforces that its value is always smaller than `Rate`.
///
/// `Rate` MUST be smaller than `U256`, trying to initialize cursor with an invalid rate will
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
// see: https://rust.godbolt.org/z/3v4zWr4ox
impl<const RATE: usize> SpongeCursor<RATE> {
    /// Create new cursor with the provided position.
    ///
    /// Returns `None` if `pos` is bigger or equal to `Rate`.
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
            // SAFETY: the type enforces that `pos` is always smaller than `Rate`
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
            // SAFETY: the type enforces that `pos` is always smaller than `Rate`
            unsafe { core::hint::unreachable_unchecked() };
        }
    }

    /// Absorb bytes from `data` into a `u64`-based state using little ednian byte order.
    ///
    /// Size of state MUST be greater or equal to `Rate`. Using an invalid `N` will result in
    /// a compilation error.
    #[allow(
        clippy::missing_panics_doc,
        reason = "the method is panic-free, see: https://rust.godbolt.org/z/88Wf1qEsr"
    )]
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
            let rem_len = RATE - pos;

            let is_partial = data.len() < rem_len;
            let head = if is_partial {
                data
            } else {
                let (head, tail) = data.split_at(rem_len);
                data = tail;
                head
            };

            utils::absorb_partial::<N, RATE>(state, pos, head);

            if is_partial {
                self.pos = u8::try_from(pos + head.len()).expect("the sum is smaller than Rate");
                return;
            }

            sponge(state);
        }

        let blocks = data.chunks_exact(RATE);
        let tail = blocks.remainder();

        for block in blocks {
            let block: &[u8; RATE] = block.try_into().expect("block has correct size");
            utils::absorb_full(state, block);
            sponge(state);
        }

        if !tail.is_empty() {
            utils::absorb_partial::<N, RATE>(state, 0, tail);
        }

        self.pos = u8::try_from(tail.len()).expect("tail.len() is smaller than RATE");
    }

    /// Squeeze data by reading it into `buf`.
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
            utils::squeeze_read_partial::<N, RATE>,
            utils::squeeze_read_full,
        );
    }

    /// Squeeze data by XOR-ing it with data in `buf`.
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
            utils::squeeze_xor_partial::<N, RATE>,
            utils::squeeze_xor_full,
        );
    }

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
                self.pos = u8::try_from(pos + buf.len()).expect("the sum is smaller than RATE");
                return;
            }

            let (head, tail) = buf.split_at_mut(rem_len);
            buf = tail;

            process_partial(state, pos, head);
        }

        let mut blocks = buf.chunks_exact_mut(RATE);

        for block in &mut blocks {
            sponge(state);
            let block = block.try_into().expect("block has correct size");
            process_full(state, block);
        }

        let tail = blocks.into_remainder();

        if !tail.is_empty() {
            sponge(state);
            process_partial(state, 0, tail);
        }

        self.pos = u8::try_from(tail.len()).expect("tail.len() is smaller than RATE");
    }
}

#[cfg(feature = "zeroize")]
impl<const RATE: usize> zeroize::Zeroize for SpongeCursor<RATE> {
    fn zeroize(&mut self) {
        self.pos.zeroize();
    }
}
