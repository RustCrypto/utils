//! [`Zeroize`] impls for WASM SIMD registers.

use crate::{atomic_fence, volatile_write, Zeroize};

use core::arch::wasm32::v128;

macro_rules! impl_zeroize_for_simd_register {
    ($($type:ty),* $(,)?) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(target_arch = "wasm32", target_family = "wasm")))]
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

impl_zeroize_for_simd_register!(v128);
