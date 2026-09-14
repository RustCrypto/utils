#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(missing_docs, unused_qualifications)]

//! ## Usage
//!
//! ```
//! use dit::Dit;
//!
//! let dit = Dit::init();
//! assert!(!dit.is_enabled());
//! let _guard = dit.enable();
//! assert!(dit.is_enabled());
//! ```

#[cfg(target_arch = "aarch64")]
use core::arch::asm;

#[cfg(target_arch = "aarch64")]
cpufeatures::new!(dit_supported, "dit");

/// Data-Independent Timing: support for enabling features of AArch64 CPUs which improve
/// constant-time operation.
pub struct Dit {
    #[cfg(target_arch = "aarch64")]
    supported: dit_supported::InitToken,
}

impl Dit {
    /// Initialize Data-Independent Timing using runtime CPU feature detection.
    #[cfg(target_arch = "aarch64")]
    pub fn init() -> Self {
        Self {
            supported: dit_supported::init(),
        }
    }

    /// Enable Data-Independent Timing (if available).
    ///
    /// Returns an RAII guard that will return DIT to its previous state when dropped.
    #[cfg(target_arch = "aarch64")]
    #[must_use]
    pub fn enable(&self) -> Guard<'_> {
        let was_enabled = if self.is_supported() {
            unsafe { set_dit_enabled() }
        } else {
            false
        };

        Guard {
            dit: self,
            was_enabled,
        }
    }

    /// Check if DIT has been enabled.
    pub fn is_enabled(&self) -> bool {
        #[cfg(target_arch = "aarch64")]
        if self.is_supported() {
            return unsafe { get_dit_enabled() };
        }

        false
    }

    /// Check if DIT is supported by this CPU.
    pub fn is_supported(&self) -> bool {
        #[cfg(target_arch = "aarch64")]
        return self.supported.get();
        #[cfg(not(target_arch = "aarch64"))]
        false
    }
}

/// RAII guard which returns DIT to its previous state when dropped.
#[cfg(target_arch = "aarch64")]
pub struct Guard<'a> {
    /// DIT implementation.
    dit: &'a Dit,

    /// Previous DIT state before it was enabled.
    was_enabled: bool,
}

#[cfg(target_arch = "aarch64")]
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if self.dit.supported.get() {
            unsafe { restore_dit(self.was_enabled) }
        }
    }
}

/// Detect if DIT is enabled for the current thread by checking the processor state register.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "dit")]
unsafe fn get_dit_enabled() -> bool {
    let mut dit: u64;
    unsafe {
        asm!(
            "mrs {dit}, DIT",
            dit = out(reg) dit,
            options(nomem, nostack, preserves_flags)
        );
    }
    (dit >> 24) & 1 != 0
}

/// Enable DIT for the current thread.
///
/// Returns the previous DIT state prior to enabling DIT.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "dit")]
unsafe fn set_dit_enabled() -> bool {
    unsafe {
        let was_enabled = get_dit_enabled();
        asm!("msr DIT, #1", options(nomem, nostack, preserves_flags));
        was_enabled
    }
}

/// Restore DIT state depending on the enabled bit.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "dit")]
unsafe fn restore_dit(enabled: bool) {
    if !enabled {
        // Disable DIT
        unsafe { asm!("msr DIT, #0", options(nomem, nostack, preserves_flags)) };
    }
}

#[cfg(target_arch = "aarch64")]
#[cfg(test)]
mod tests {
    use super::{Dit, get_dit_enabled, restore_dit, set_dit_enabled};
    cpufeatures::new!(dit_supported, "dit");

    #[test]
    fn high_level_api() {
        let dit = Dit::init();
        assert!(dit.is_supported());

        {
            assert!(!dit.is_enabled());
            let _guard = dit.enable();
            assert!(dit.is_enabled());

            // Test nested usage
            {
                let _guard2 = dit.enable();
                assert!(dit.is_enabled());
            }

            assert!(dit.is_enabled());
        }

        assert!(!dit.is_enabled());
    }

    #[test]
    fn asm_wrappers() {
        let dit_token = dit_supported::init();
        if !dit_token.get() {
            panic!("DIT is not available on this CPU");
        }

        let dit_enabled = unsafe { get_dit_enabled() };
        assert!(!dit_enabled);

        let was_enabled = unsafe { set_dit_enabled() };
        assert!(!was_enabled);
        let dit_enabled = unsafe { get_dit_enabled() };
        assert!(dit_enabled);

        unsafe { restore_dit(true) };
        let dit_enabled = unsafe { get_dit_enabled() };
        assert!(dit_enabled);

        unsafe { restore_dit(false) };
        let dit_enabled = unsafe { get_dit_enabled() };
        assert!(!dit_enabled);
    }
}
