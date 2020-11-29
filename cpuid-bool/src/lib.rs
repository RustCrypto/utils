//! Macro for checking CPU capabilities at runtime.
//!
//! # Example
//! ```
//! // This macro creates `cpuid_aes_sha` module
//! cpuid_bool::new!(cpuid_aes_sha, "aes", "sha");
//!
//! // `token` is a Zero Sized Type value, which guarantees
//! // that underlying static storage got properly initialized,
//! // which allows to omit initialization branch
//! let token: cpuid_aes_sha::InitToken = cpuid_aes_sha::init();
//! if token.get() {
//!     println!("CPU supports both SHA and AES extensions");
//! } else {
//!     println!("SHA and AES extensions are not supported");
//! }
//!
//! // If stored value needed only once you can get stored value
//! // omitting the token
//! let val = cpuid_aes_sha::get();
//! assert_eq!(val, token.get());
//!
//! // Additionally you can get both token and value
//! let (token, val) = cpuid_aes_sha::init_get();
//! assert_eq!(val, token.get());
//! ```
//! Note that if all tested target features are enabled via compiler options
//! (e.g. by using `RUSTFLAGS`), the `get` method will always return `true`
//! and `init` will not use CPUID instruction. Such behavior allows
//! compiler to completely eliminate fallback code.
//!
//! After first call macro caches result and returns it in subsequent
//! calls, thus runtime overhead for them is minimal.
#![no_std]
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!("This crate works only on x86 and x86-64 targets.");

/// Create module with CPUID bool code.
#[macro_export]
macro_rules! new {
    ($mod_name:ident, $($tf:tt),+ $(,)? ) => {
        mod $mod_name {
            use core::sync::atomic::{AtomicU8, Ordering::Relaxed};

            const UNINIT: u8 = u8::max_value();
            static STORAGE: AtomicU8 = AtomicU8::new(UNINIT);

            /// Initialization token
            #[derive(Copy, Clone, Debug)]
            pub struct InitToken(());

            impl InitToken {
                /// Get initialized value
                #[inline(always)]
                pub fn get(&self) -> bool {
                    // CPUID is not available on SGX targets
                    #[cfg(all(not(target_env = "sgx"), not(all($(target_feature=$tf, )*))))]
                    let res = STORAGE.load(Relaxed) == 1;
                    #[cfg(all(target_env = "sgx", not(all($(target_feature=$tf, )*))))]
                    let res = false;
                    #[cfg(all($(target_feature=$tf, )*))]
                    let res = true;
                    res
                }
            }

            /// Initialize underlying storage if needed and get
            /// stored value and initialization token.
            #[inline]
            pub fn init_get() -> (InitToken, bool) {
                // CPUID is not available on SGX targets
                #[cfg(all(not(target_env = "sgx"), not(all($(target_feature=$tf, )*))))]
                let res = {
                    #[cfg(target_arch = "x86")]
                    use core::arch::x86::{__cpuid, __cpuid_count};
                    #[cfg(target_arch = "x86_64")]
                    use core::arch::x86_64::{__cpuid, __cpuid_count};

                    // Relaxed ordering is fine, as we only have a single atomic variable.
                    let val = STORAGE.load(Relaxed);
                    if val == UNINIT {
                        #[allow(unused_variables)]
                        let cr = unsafe {
                            [__cpuid(1), __cpuid_count(7, 0)]
                        };
                        let res = $(cpuid_bool::check!(cr, $tf) & )+ true;
                        STORAGE.store(res as u8, Relaxed);
                        res
                    } else {
                        val == 1
                    }
                };
                #[cfg(all(target_env = "sgx", not(all($(target_feature=$tf, )*))))]
                let res = false;
                #[cfg(all($(target_feature=$tf, )*))]
                let res = true;

                (InitToken(()), res)
            }

            /// Initialize underlying storage if needed and get
            /// initialization token.
            #[inline]
            pub fn init() -> InitToken {
                init_get().0
            }

            /// Initialize underlying storage if needed and get
            /// stored value.
            #[inline]
            pub fn get() -> bool {
                init_get().1
            }
        }
    };
}

// TODO: find how to define private macro usable inside a public one
macro_rules! expand_check_macro {
    ($(($name:tt, $i:expr, $reg:ident, $offset:expr)),* $(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                ($cr:expr, $name) => { ($cr[$i].$reg & (1 << $offset) != 0) };
            )*
        }
    };
}

expand_check_macro! {
    ("mmx", 0, edx, 23),
    ("sse", 0, edx, 25),
    ("sse2", 0, edx, 26),
    ("sse3", 0, ecx, 0),
    ("pclmulqdq", 0, ecx, 1),
    ("ssse3", 0, ecx, 9),
    ("fma", 0, ecx, 12),
    ("sse4.1", 0, ecx, 19),
    ("sse4.2", 0, ecx, 20),
    ("popcnt", 0, ecx, 23),
    ("aes", 0, ecx, 25),
    ("avx", 0, ecx, 28),
    ("rdrand", 0, ecx, 30),
    ("sgx", 1, ebx, 2),
    ("bmi1", 1, ebx, 3),
    ("avx2", 1, ebx, 5),
    ("bmi2", 1, ebx, 8),
    ("rdseed", 1, ebx, 18),
    ("adx", 1, ebx, 19),
    ("sha", 1, ebx, 29),
}
