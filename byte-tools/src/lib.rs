#![no_std]

/// Copy bytes from `src` to `dst`
///
/// Panics if the size of `src` is bigger than the size of `dst`
#[inline(always)]
pub fn copy(src: &[u8], dst: &mut [u8]) {
    assert!(dst.len() >= src.len());
    let dst = &mut dst[..src.len()]; // make sure that dst is the same length as src
    dst.copy_from_slice(src);        // as it is required by copy_from_slice
}

/// Zero all bytes in `dst`
#[inline(always)]
pub fn zero(dst: &mut [u8]) {
    set(dst, 0);
}

/// Sets all bytes in `dst` equal to `value`
#[inline(always)]
pub fn set(dst: &mut [u8], value: u8) {
    for elem in dst.iter_mut() {
        *elem = value;
    }
}
