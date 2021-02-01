//! Base64 variants

/// Core encoder/decoder functions for a particular Base64 variant
pub trait Variant {
    /// Is this encoding padded?
    const PADDED: bool;

    /// Decode 6-bits of a Base64 message
    fn decode_6bits(src: u8) -> i16;

    /// Encode 6-bits of a Base64 message
    fn encode_6bits(src: i16) -> u8;
}
