//! Limbs are smaller integers into which a big integer is subdivided.
// TODO(tarcieri): `Limb` newtype?

/// Big integers are modeled as an array of smaller integers called "limbs".
#[cfg(target_pointer_width = "32")]
pub type Limb = u32;

/// Big integers are modeled as an array of smaller integers called "limbs".
#[cfg(target_pointer_width = "64")]
pub type Limb = u64;

/// Computes `a + b + carry`, returning the result along with the new carry.
/// 32-bit version.
#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub(crate) const fn adc(a: Limb, b: Limb, carry: Limb) -> (Limb, Limb) {
    let ret = (a as u64) + (b as u64) + (carry as u64);
    (ret as u32, (ret >> 32) as u32)
}

/// Computes `a + b + carry`, returning the result along with the new carry.
/// 64-bit version.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub(crate) const fn adc(a: Limb, b: Limb, carry: Limb) -> (Limb, Limb) {
    let ret = (a as u128) + (b as u128) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}

/// Computes `a - (b + borrow)`, returning the result along with the new borrow.
/// 32-bit version.
#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub const fn sbb(a: Limb, b: Limb, borrow: Limb) -> (Limb, Limb) {
    let ret = (a as u64).wrapping_sub((b as u64) + ((borrow >> 31) as u64));
    (ret as u32, (ret >> 32) as u32)
}

/// Computes `a - (b + borrow)`, returning the result along with the new borrow.
/// 64-bit version.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub const fn sbb(a: Limb, b: Limb, borrow: Limb) -> (Limb, Limb) {
    let ret = (a as u128).wrapping_sub((b as u128) + ((borrow >> 63) as u128));
    (ret as u64, (ret >> 64) as u64)
}

/// Computes `a + (b * c) + carry`, returning the result along with the new carry.
/// 32-bit version.
#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub(crate) const fn mac(a: Limb, b: Limb, c: Limb, carry: Limb) -> (Limb, Limb) {
    let ret = (a as u64) + ((b as u64) * (c as u64)) + (carry as u64);
    (ret as u32, (ret >> 32) as u32)
}

/// Computes `a + (b * c) + carry`, returning the result along with the new carry.
/// 64-bit version.
#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub(crate) const fn mac(a: Limb, b: Limb, c: Limb, carry: Limb) -> (Limb, Limb) {
    let ret = (a as u128) + ((b as u128) * (c as u128)) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}
