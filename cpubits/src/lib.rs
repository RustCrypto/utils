#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

use std::fmt;

/// Number of bits which should be treated as a CPU's "word size".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CpuBits {
    /// Use 32-bit backends for this target.
    B32 = 32,

    /// Use 64-bit backends for this target.
    B64 = 64,
}

impl CpuBits {
    /// Configure the build by emitting `rustcrypto_cpubits` cfg attributes.
    ///
    /// This invokes [`Self::detect_with_cfg_overrides`] and emits the
    /// `rustcrypto_cpubits` cfg attributes, propagating the explicit selection
    /// if it's already been made.
    pub fn configure_build() {
        println!("{}", Self::detect_with_cfg_overrides());
    }

    /// Apply heuristics to autodetect an optimal number of bits for the
    /// current target.
    ///
    /// Note that this may return values greater than a CPU's native word size
    /// in the event that codegen for a larger value is expected to provide
    /// better performance.
    ///
    /// This function ignores `cfg(rustcrypto_cpubits)` overrides.
    pub fn detect() -> Self {
        // TODO(tarcieri): target-specific customizations for e.g. ARMV7, `wasm32`
        if cfg!(target_pointer_width = "64") {
            CpuBits::B64
        } else {
            CpuBits::B32
        }
    }

    /// Use `cfg(rustcrypto_cpubits)` overrides if configured, or otherwise
    /// fall back on [`CpuBits::detect`].
    ///
    /// Ignores `cfg(rustcrypto_cpubits)` if it is set to a value other than
    /// "32" or "64". This is a limitation of `cfg` attributes.
    #[allow(dead_code)]
    pub fn detect_with_cfg_overrides() -> Self {
        #[cfg(rustcrypto_cpubits = "32")]
        return CpuBits::B32;

        #[cfg(rustcrypto_cpubits = "64")]
        return CpuBits::B64;

        CpuBits::detect()
    }
}

impl fmt::Display for CpuBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "cargo:rustc-cfg=rustcrypto_cpubits=\"{}\"", *self as u8)
    }
}
