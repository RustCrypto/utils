#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(not(target_arch = "aarch64"))]
compile_error!("This crate only builds on `aarch64` targets");

use core::arch::asm;

/// Detect if DIT is enabled for the current thread by checking the processor state register.
#[target_feature(enable = "dit")]
pub unsafe fn get_dit_enabled() -> bool {
    let mut dit: u64;
    asm!(
        "mrs {dit}, DIT",
        dit = out(reg) dit,
        options(nomem, nostack, preserves_flags)
    );
    (dit >> 24) & 1 != 0
}

/// Enable DIT for the current thread.
#[target_feature(enable = "dit")]
pub unsafe fn set_dit_enabled() {
    asm!("msr DIT, #1", options(nomem, nostack, preserves_flags));
}

/// Restore DIT state depending on the enabled bit.
#[target_feature(enable = "dit")]
pub unsafe fn restore_dit(enabled: bool) {
    if !enabled {
        // Disable DIT
        asm!("msr DIT, #0", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(test)]
mod tests {
    use super::{get_dit_enabled, restore_dit, set_dit_enabled};
    cpufeatures::new!(dit_supported, "dit");

    #[test]
    fn get() {
        let dit_token = dit_supported::init();
        if !dit_token.get() {
            panic!("DIT is not available on this CPU");
        }

        let dit_enabled = unsafe { get_dit_enabled() };
        assert!(!dit_enabled);

        unsafe { set_dit_enabled() };
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
