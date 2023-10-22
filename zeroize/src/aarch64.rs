//! [`Zeroize`] impls for ARM64 SIMD registers.

use crate::{atomic_fence, volatile_write, Zeroize};

use core::arch::aarch64::*;

macro_rules! impl_zeroize_for_simd_register {
    ($($type:ty),* $(,)?) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(target_arch = "aarch64")))]
            impl Zeroize for $type {
                #[inline]
                fn zeroize(&mut self) {
                    volatile_write(self, unsafe { core::mem::zeroed() });
                    atomic_fence();
                }
            }
        )+
    };
}

impl_zeroize_for_simd_register! {
    uint8x8_t,
    uint8x16_t,
    uint16x4_t,
    uint16x8_t,
    uint32x2_t,
    uint32x4_t,
    uint64x1_t,
    uint64x2_t,
    int8x8_t,
    int8x16_t,
    int16x4_t,
    int16x8_t,
    int32x2_t,
    int32x4_t,
    int64x1_t,
    int64x2_t,
    float32x2_t,
    float32x4_t,
    float64x1_t,
    float64x2_t,
    poly8x8_t,
    poly8x16_t,
    poly16x4_t,
    poly16x8_t,
    poly64x1_t,
    poly64x2_t,
}
