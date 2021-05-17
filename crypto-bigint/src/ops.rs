//! Arithmetic operations designed for efficient lowering to machine code..

/// Computes `a + b + carry`, returning the result along with the new carry.
/// 32-bit version.
#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub const fn adc(a: u32, b: u32, carry: u32) -> (u32, u32) {
    let ret = (a as u64) + (b as u64) + (carry as u64);
    (ret as u32, (ret >> 32) as u32)
}

/// Computes `a + b + carry`, returning the result along with the new carry.
/// 64-bit version.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let ret = (a as u128) + (b as u128) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}
