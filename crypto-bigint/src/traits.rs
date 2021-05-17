//! Traits provided by this crate

/// Number of bits required to express a given big integer.
pub trait NumBits {
    /// Number of bits required to express this integer.
    const NUM_BITS: usize;
}

/// Number of bytes required to express a given big integer.
pub trait NumBytes {
    /// Number of bytes required to express this integer.
    const NUM_BYTES: usize;
}
