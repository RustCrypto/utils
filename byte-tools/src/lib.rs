#![no_std]
use core::{mem, ptr};

/// Write a u64 into a vector, which must be 8 bytes long. The value is written
/// in big-endian format.
pub fn write_u64_be(dst: &mut [u8], mut input: u64) {
    assert!(dst.len() == 8);
    input = input.to_be();
    unsafe {
        let tmp = &input as *const _ as *const u8;
        ptr::copy_nonoverlapping(tmp, dst.get_unchecked_mut(0), 8);
    }
}

/// Write a u64 into a vector, which must be 8 bytes long. The value is written
/// in little-endian format.
pub fn write_u64_le(dst: &mut [u8], mut input: u64) {
    assert!(dst.len() == 8);
    input = input.to_le();
    unsafe {
        let tmp = &input as *const _ as *const u8;
        ptr::copy_nonoverlapping(tmp, dst.get_unchecked_mut(0), 8);
    }
}

/// Write a vector of u64s into a vector of bytes. The values are written in
/// little-endian format.
pub fn write_u64v_be(dst: &mut [u8], input: &[u64]) {
    assert!(dst.len() == 8 * input.len());
    unsafe {
        let mut x: *mut u8 = dst.get_unchecked_mut(0);
        let mut y: *const u64 = input.get_unchecked(0);
        for _ in 0..input.len() {
            let tmp = (*y).to_be();
            ptr::copy_nonoverlapping(&tmp as *const _ as *const u8, x, 8);
            x = x.offset(8);
            y = y.offset(1);
        }
    }
}

/// Write a vector of u64s into a vector of bytes. The values are written in
/// little-endian format.
pub fn write_u64v_le(dst: &mut [u8], input: &[u64]) {
    assert!(dst.len() == 8 * input.len());
    unsafe {
        let mut x: *mut u8 = dst.get_unchecked_mut(0);
        let mut y: *const u64 = input.get_unchecked(0);
        for _ in 0..input.len() {
            let tmp = (*y).to_le();
            ptr::copy_nonoverlapping(&tmp as *const _ as *const u8, x, 8);
            x = x.offset(8);
            y = y.offset(1);
        }
    }
}

/// Write a u32 into a vector, which must be 4 bytes long. The value is written
/// in big-endian format.
pub fn write_u32_be(dst: &mut [u8], mut input: u32) {
    assert!(dst.len() == 4);
    input = input.to_be();
    unsafe {
        let tmp = &input as *const _ as *const u8;
        ptr::copy_nonoverlapping(tmp, dst.get_unchecked_mut(0), 4);
    }
}

/// Write a u32 into a vector, which must be 4 bytes long. The value is written
/// in little-endian format.
pub fn write_u32_le(dst: &mut [u8], mut input: u32) {
    assert!(dst.len() == 4);
    input = input.to_le();
    unsafe {
        let tmp = &input as *const _ as *const u8;
        ptr::copy_nonoverlapping(tmp, dst.get_unchecked_mut(0), 4);
    }
}

/// Write a vector of u32s into a vector of bytes. The values are written in
/// little-endian format.
pub fn write_u32v_le(dst: &mut [u8], input: &[u32]) {
    assert!(dst.len() == 4 * input.len());
    unsafe {
        let mut x: *mut u8 = dst.get_unchecked_mut(0);
        let mut y: *const u32 = input.get_unchecked(0);
        for _ in 0..input.len() {
            let tmp = (*y).to_le();
            ptr::copy_nonoverlapping(&tmp as *const _ as *const u8, x, 4);
            x = x.offset(4);
            y = y.offset(1);
        }
    }
}

/// Write a vector of u32s into a vector of bytes. The values are written in
/// big-endian format.
pub fn write_u32v_be(dst: &mut [u8], input: &[u32]) {
    assert!(dst.len() == 4 * input.len());
    unsafe {
        let mut x: *mut u8 = dst.get_unchecked_mut(0);
        let mut y: *const u32 = input.get_unchecked(0);
        for _ in 0..input.len() {
            let tmp = (*y).to_be();
            ptr::copy_nonoverlapping(&tmp as *const _ as *const u8, x, 4);
            x = x.offset(4);
            y = y.offset(1);
        }
    }
}

/// Read a vector of bytes into a vector of u64s. The values are read in
/// big-endian format.
pub fn read_u64v_be(dst: &mut [u64], input: &[u8]) {
    assert!(dst.len() * 8 == input.len());
    unsafe {
        let mut x: *mut u64 = dst.get_unchecked_mut(0);
        let mut y: *const u8 = input.get_unchecked(0);
        for _ in 0..dst.len() {
            let mut tmp: u64 = mem::uninitialized();
            ptr::copy_nonoverlapping(y, &mut tmp as *mut _ as *mut u8, 8);
            *x = u64::from_be(tmp);
            x = x.offset(1);
            y = y.offset(8);
        }
    }
}

/// Read a vector of bytes into a vector of u64s. The values are read in
/// little-endian format.
pub fn read_u64v_le(dst: &mut [u64], input: &[u8]) {
    assert!(dst.len() * 8 == input.len());
    unsafe {
        let mut x: *mut u64 = dst.get_unchecked_mut(0);
        let mut y: *const u8 = input.get_unchecked(0);
        for _ in 0..dst.len() {
            let mut tmp: u64 = mem::uninitialized();
            ptr::copy_nonoverlapping(y, &mut tmp as *mut _ as *mut u8, 8);
            *x = u64::from_le(tmp);
            x = x.offset(1);
            y = y.offset(8);
        }
    }
}

/// Read a vector of bytes into a vector of u32s. The values are read in
/// big-endian format.
pub fn read_u32v_be(dst: &mut [u32], input: &[u8]) {
    assert!(dst.len() * 4 == input.len());
    unsafe {
        let mut x: *mut u32 = dst.get_unchecked_mut(0);
        let mut y: *const u8 = input.get_unchecked(0);
        for _ in 0..dst.len() {
            let mut tmp: u32 = mem::uninitialized();
            ptr::copy_nonoverlapping(y, &mut tmp as *mut _ as *mut u8, 4);
            *x = u32::from_be(tmp);
            x = x.offset(1);
            y = y.offset(4);
        }
    }
}

/// Read a vector of bytes into a vector of u32s. The values are read in
/// little-endian format.
pub fn read_u32v_le(dst: &mut [u32], input: &[u8]) {
    assert!(dst.len() * 4 == input.len());
    unsafe {
        let mut x: *mut u32 = dst.get_unchecked_mut(0);
        let mut y: *const u8 = input.get_unchecked(0);
        for _ in 0..dst.len() {
            let mut tmp: u32 = mem::uninitialized();
            ptr::copy_nonoverlapping(y, &mut tmp as *mut _ as *mut u8, 4);
            *x = u32::from_le(tmp);
            x = x.offset(1);
            y = y.offset(4);
        }
    }
}

/// Read the value of a vector of bytes as a u32 value in little-endian format.
pub fn read_u32_le(input: &[u8]) -> u32 {
    assert!(input.len() == 4);
    unsafe {
        let mut tmp: u32 = mem::uninitialized();
        ptr::copy_nonoverlapping(input.get_unchecked(0),
                                 &mut tmp as *mut _ as *mut u8,
                                 4);
        u32::from_le(tmp)
    }
}

/// Read the value of a vector of bytes as a u32 value in big-endian format.
pub fn read_u32_be(input: &[u8]) -> u32 {
    assert!(input.len() == 4);
    unsafe {
        let mut tmp: u32 = mem::uninitialized();
        ptr::copy_nonoverlapping(input.get_unchecked(0),
                                 &mut tmp as *mut _ as *mut u8,
                                 4);
        u32::from_be(tmp)
    }
}

/// XOR plaintext and keystream, storing the result in dst.
pub fn xor_keystream(dst: &mut [u8], plaintext: &[u8], keystream: &[u8]) {
    assert!(dst.len() == plaintext.len());
    assert!(plaintext.len() <= keystream.len());

    // Do one byte at a time, using unsafe to skip bounds checking.
    let p = plaintext.as_ptr();
    let k = keystream.as_ptr();
    let d = dst.as_mut_ptr();
    for i in 0isize..plaintext.len() as isize {
        unsafe { *d.offset(i) = *p.offset(i) ^ *k.offset(i) };
    }
}

/// Copy bytes from src to dest
#[inline]
pub fn copy_memory(src: &[u8], dst: &mut [u8]) {
    assert!(dst.len() >= src.len());
    unsafe {
        let srcp = src.as_ptr();
        let dstp = dst.as_mut_ptr();
        ptr::copy_nonoverlapping(srcp, dstp, src.len());
    }
}

/// Zero all bytes in dst
#[inline]
pub fn zero(dst: &mut [u8]) {
    unsafe {
        ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
    }
}

/// Convert the value in bytes to the number of bits, a tuple where the 1st
/// item is the high-order value and the 2nd item is the low order value.
fn to_bits(x: u64) -> (u64, u64) { (x >> 61, x << 3) }

/// Adds the specified number of bytes to the bit count. panic!() if this
/// would cause numeric overflow.
pub fn add_bytes_to_bits(bits: u64, bytes: u64) -> u64 {
    let (new_high_bits, new_low_bits) = to_bits(bytes);

    if new_high_bits > 0 {
        panic!("Numeric overflow occured.")
    }

    bits.checked_add(new_low_bits).expect("Numeric overflow occured.")
}

/// Adds the specified number of bytes to the bit count, which is a tuple where
/// the first element is the high order value. panic!() if this would cause
/// numeric overflow.
pub fn add_bytes_to_bits_tuple(bits: (u64, u64), bytes: u64) -> (u64, u64) {
    let (new_high_bits, new_low_bits) = to_bits(bytes);
    let (hi, low) = bits;

    // Add the low order value - if there is no overflow, then add the high
    // order values. If the addition of the low order values causes overflow,
    // add one to the high order values before adding them.
    match low.checked_add(new_low_bits) {
        Some(x) => {
            if new_high_bits == 0 {
                // This is the fast path - every other alternative will rarely
                // occur in practice considering how large an input would need
                // to be for those paths to be used.
                (hi, x)
            } else {
                match hi.checked_add(new_high_bits) {
                    Some(y) => (y, x),
                    None => panic!("Numeric overflow occured."),
                }
            }
        },
        None => {
            let z = match new_high_bits.checked_add(1) {
                Some(w) => w,
                None => panic!("Numeric overflow occured."),
            };
            match hi.checked_add(z) {
                // This re-executes the addition that was already performed
                // earlier when overflow occured, this time allowing the
                // overflow to happen. Technically, this could be avoided by
                // using the checked add intrinsic directly, but that involves
                // using unsafe code and is not really worthwhile considering
                // how infrequently code will run in practice. This is the
                // reason that this function requires that the type T be
                // UnsignedInt - overflow is not defined for Signed types.
                // This function could be implemented for signed types as well
                // if that were needed.
                Some(y) => (y, low.wrapping_add(new_low_bits)),
                None => panic!("Numeric overflow occured."),
            }
        },
    }
}

#[cfg(test)]
pub mod tests;
