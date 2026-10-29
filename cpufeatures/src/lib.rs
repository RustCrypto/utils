#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

#[cfg(not(miri))]
#[cfg(target_arch = "aarch64")]
#[doc(hidden)]
pub mod aarch64;

#[cfg(not(miri))]
#[cfg(target_arch = "loongarch64")]
#[doc(hidden)]
pub mod loongarch64;

#[cfg(not(miri))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(miri)]
mod miri;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "loongarch64",
    target_arch = "x86",
    target_arch = "x86_64"
)))]
compile_error!("This crate works only on `aarch64`, `loongarch64`, `x86`, and `x86-64` targets.");

/// Create module with CPU feature detection code.
///
/// # Single target feature set
///
/// The module gets a `get` function returning whether all listed target features are available:
///
/// ```rust,ignore
/// cpufeatures::new!(aes_sha, "aes", "sha");
///
/// if aes_sha::get() {
///     // ...
/// }
/// ```
///
/// # Multiple target feature sets
///
/// Several named sets can be declared instead, separated with `;`. The module then gets a
/// `Features` enum with one variant per set and `get` returns the first variant whose target
/// features are all available. The trailing entry carries no target features and names the
/// variant returned when none of the sets is available.
///
/// ```rust,ignore
/// cpufeatures::new!(
///     backend;
///     Avx2: "avx2", "aes";
///     Aes: "aes", "sse4.1";
///     Soft;
/// );
///
/// use backend::Features;
///
/// match backend::get() {
///     Features::Avx2 => { /* ... */ }
///     Features::Aes => { /* ... */ }
///     Features::Soft => { /* ... */ }
/// }
/// ```
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
                /// Initialize token, performing CPU feature detection.
                pub fn init() -> Self {
                    init()
                }

                /// Initialize token and return a `bool` indicating if the feature is supported.
                pub fn init_get() -> (Self, bool) {
                    init_get()
                }

                /// Get initialized value.
                #[inline(always)]
                pub fn get(&self) -> bool {
                    $crate::__unless_target_features! {
                        $($tf),+ => {
                            STORAGE.load(Relaxed) == 1
                        }
                    }
                }
            }

            /// Get stored value and initialization token,
            /// initializing underlying storage if needed.
            #[inline]
            pub fn init_get() -> (InitToken, bool) {
                let res = $crate::__unless_target_features! {
                    $($tf),+ => {
                        #[cold]
                        fn init_inner() -> bool {
                            let res = $crate::__detect_target_features!($($tf),+);
                            STORAGE.store(res as u8, Relaxed);
                            res
                        }

                        // Relaxed ordering is fine, as we only have a single atomic variable.
                        let val = STORAGE.load(Relaxed);

                        if val == UNINIT {
                            init_inner()
                        } else {
                            val == 1
                        }
                    }
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
    // Every entry has the same shape, a name optionally followed by target features, so that the
    // matcher never has to decide between starting one more set and finishing the list.
    // Spelling the trailing entry out as a bare name in its own position instead would be a local
    // ambiguity, as both alternatives bind an `ident`.
    ($mod_name:ident; $($variant:ident $(: $($tf:tt),+)?);+ $(;)?) => {
        mod $mod_name {
            use core::sync::atomic::{AtomicU8, Ordering::Relaxed};

            /// Target feature set detected at runtime.
            ///
            /// Variants are ordered as declared, i.e. the detected one is always the first variant
            /// whose target features are all available. The last variant declares no target
            /// features and is the one detected when no other is available.
            #[derive(Copy, Clone, Debug, Eq, PartialEq)]
            #[repr(u8)]
            pub enum Features {
                $(
                    #[doc = concat!("The `", stringify!($variant), "` target feature set.")]
                    $(
                        #[doc = concat!("\nAvailable target features:", $(" `", $tf, "`",)+)]
                    )?
                    $variant,
                )*
            }

            // Whether each entry declares target features. `!$tf.is_empty()` is a constant `true`
            // whose only purpose is to depend on `$tf`, so that the term is emitted exactly for
            // the entries that have one.
            const HAS_TARGET_FEATURES: &[bool] = &[$(false $($(|| !$tf.is_empty())+)?),+];

            const _: () = {
                let len = HAS_TARGET_FEATURES.len();

                assert!(len > 1, "`cpufeatures::new!` expects at least one target feature set");
                assert!(
                    !HAS_TARGET_FEATURES[len - 1],
                    "the last `cpufeatures::new!` entry names the variant detected when none \
                     of the target feature sets is available and must not declare any itself"
                );

                let mut i = 0;
                while i < len - 1 {
                    assert!(
                        HAS_TARGET_FEATURES[i],
                        "only the last `cpufeatures::new!` entry may omit target features"
                    );
                    i += 1;
                }
            };

            /// Variant detected when none of the target feature sets is available.
            const FALLBACK: Features = {
                let all = [$(Features::$variant),+];
                all[all.len() - 1]
            };

            // Value stored in `STORAGE` until CPU feature detection has been performed.
            //
            // `Features` is `#[repr(u8)]` and does not use explicit discriminants, so its tags are
            // exactly `0..=FALLBACK` and the value right past the last variant can not collide
            // with any of them.
            const UNINIT: u8 = FALLBACK as u8 + 1;

            // Every `Features` tag has to stay below `UNINIT`, otherwise `init_get` could not
            // tell the uninitialized state apart and the transmutes below would be unsound.
            const _: () = {
                $(assert!((Features::$variant as u8) < UNINIT);)*
            };

            // Set when the detected variant follows from compile-time information alone, which
            // happens in two cases: the first declared set is enabled at compile time, as it is
            // probed first and therefore always wins, and the target having no runtime detection
            // at all, where no set beyond the enabled ones can ever be found. Either way no
            // storage is involved and `init_inner` never runs.
            const STATICALLY_DETECTED: Option<Features> = {
                // The last entry declares no target features, so it is always enabled and
                // terminates the search below.
                let enabled = [$(cfg!(all($($(target_feature = $tf,)+)?))),+];

                if enabled[0] || !$crate::__runtime_detection_available!() {
                    let mut i = 0;
                    while !enabled[i] {
                        i += 1;
                    }

                    Some([$(Features::$variant),+][i])
                } else {
                    None
                }
            };

            static STORAGE: AtomicU8 = AtomicU8::new(UNINIT);

            /// Initialization token
            #[derive(Copy, Clone, Debug)]
            pub struct InitToken(());

            impl InitToken {
                /// Initialize token, performing CPU feature detection.
                pub fn init() -> Self {
                    init()
                }

                /// Initialize token and return the detected target feature set.
                pub fn init_get() -> (Self, Features) {
                    init_get()
                }

                /// Get initialized value.
                #[inline(always)]
                pub fn get(&self) -> Features {
                    match STATICALLY_DETECTED {
                        Some(features) => features,
                        None => {
                            let val = STORAGE.load(Relaxed);

                            // SAFETY: `InitToken` can only be obtained from `init_get`, which
                            // stores the tag of a valid `Features` value into `STORAGE` before
                            // constructing the token, and the tag is never modified afterwards.
                            unsafe { core::mem::transmute::<u8, Features>(val) }
                        }
                    }
                }
            }

            #[cold]
            fn init_inner() -> Features {
                let res = 'detect: {
                    $($(
                        if $crate::__unless_target_features! {
                            $($tf),+ => { $crate::__detect_target_features!($($tf),+) }
                        } {
                            break 'detect Features::$variant;
                        }
                    )?)*

                    FALLBACK
                };

                STORAGE.store(res as u8, Relaxed);

                res
            }

            /// Get detected target feature set and initialization token, initializing underlying
            /// storage if needed.
            #[inline]
            pub fn init_get() -> (InitToken, Features) {
                let res = match STATICALLY_DETECTED {
                    Some(features) => features,
                    None => {
                        // Relaxed ordering is fine, as we only have a single atomic variable.
                        let val = STORAGE.load(Relaxed);

                        if val == UNINIT {
                            init_inner()
                        } else {
                            // SAFETY: `STORAGE` contains either `UNINIT`, which is handled above,
                            // or the tag of a valid `Features` value written by `init_inner`.
                            unsafe { core::mem::transmute::<u8, Features>(val) }
                        }
                    }
                };

                (InitToken(()), res)
            }

            /// Initialize underlying storage if needed and get initialization token.
            #[inline]
            pub fn init() -> InitToken {
                init_get().0
            }

            /// Initialize underlying storage if needed and get detected target feature set.
            #[inline]
            pub fn get() -> Features {
                init_get().1
            }
        }
    };
}
