#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

cfg_if::cfg_if!(
    if #[cfg(miri)] {
        mod fallback;
    } else if #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "android")))] {
        #[doc(hidden)]
        pub mod aarch64_linux;
    } else if #[cfg(all(target_arch = "aarch64", target_vendor = "apple"))] {
        #[doc(hidden)]
        pub mod aarch64_apple;
    } else if #[cfg(all(target_arch = "loongarch64", target_os = "linux"))] {
        #[doc(hidden)]
        pub mod loongarch64_linux;
    } else if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        // CPUID is not available on SGX. Freestanding and UEFI targets
        // do not support SIMD features with default compilation flags.
        cfg_if::cfg_if!(
            if #[cfg(any(target_env = "sgx", target_os = "none", target_os = "uefi"))] {
                mod fallback;
            } else {
                mod x86;
            }
        );
    } else {
        mod fallback;
    }
);

/// Create module with CPU feature detection code.
#[macro_export]
macro_rules! new {
    ($mod_name:ident, $($tf:tt),+ $(,)?) => {
        mod $mod_name {
            use core::sync::atomic::{AtomicU8, Ordering::Relaxed};

            const UNINIT: u8 = u8::MAX;
            static STORAGE: AtomicU8 = AtomicU8::new(UNINIT);

            /// Initialization token
            #[derive(Copy, Clone, Debug)]
            pub struct InitToken(());

            impl InitToken {
                /// Get initialized value
                #[inline(always)]
                pub fn get(&self) -> bool {
                    if cfg!(all($(target_feature=$tf,)*)) {
                        true
                    } else if $crate::__can_detect!($($tf),+) {
                        STORAGE.load(Relaxed) == 1
                    } else {
                        false
                    }
                }
            }

            /// Get stored value and initialization token,
            /// initializing underlying storage if needed.
            #[inline]
            pub fn init_get() -> (InitToken, bool) {
                let res = if cfg!(all($(target_feature=$tf,)*)) {
                    true
                } else if $crate::__can_detect!($($tf),+) {
                    #[cold]
                    fn init_inner() -> bool {
                        let res = $crate::__detect!($($tf),+);
                        STORAGE.store(res as u8, Relaxed);
                        res
                    }

                    // Relaxed ordering is fine, as we only have a single atomic variable.
                    let storage_val = STORAGE.load(Relaxed);

                    if storage_val == UNINIT {
                        init_inner()
                    } else {
                        storage_val == 1
                    }
                } else {
                    false
                };

                (InitToken(()), res)
            }

            /// Initialize underlying storage if needed and get initialization token.
            #[inline]
            pub fn init() -> InitToken {
                init_get().0
            }

            /// Initialize underlying storage if needed and get stored value.
            #[inline]
            pub fn get() -> bool {
                init_get().1
            }
        }
    };
}
