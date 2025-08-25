//! ARM64 CPU feature detection support.
//!
//! Unfortunately ARM instructions to detect CPU features cannot be called from
//! unprivileged userspace code, so this implementation relies on OS-specific
//! APIs for feature detection.

// Evaluate the given `$body` expression any of the supplied target features
// are not enabled. Otherwise returns true.
#[macro_export]
#[doc(hidden)]
macro_rules! __can_detect {
    ($($tf:tt),+) => { true };
}

// Apple platform's runtime detection of target CPU features using `sysctlbyname`.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {{
        $($crate::check!($tf) & )+ true
    }};
}

// Apple OS (macOS, iOS, watchOS, and tvOS) `check!` macro.
//
// NOTE: several of these instructions (e.g. `aes`, `sha2`) can be assumed to
// be present on all Apple ARM64 hardware.
//
// Newer CPU instructions now have nodes within sysctl's `hw.optional`
// namespace, however the ones that do not can safely be assumed to be
// present on all Apple ARM64 devices, now and for the foreseeable future.
//
// See discussion on this issue for more information:
// <https://github.com/RustCrypto/utils/issues/378>
#[macro_export]
#[doc(hidden)]
macro_rules! check {
    ("aes") => {
        true
    };
    ("dit") => {
        // https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms#Enable-DIT-for-constant-time-cryptographic-operations
        unsafe {
            $crate::aarch64_apple::sysctlbyname(b"hw.optional.arm.FEAT_DIT\0")
        }
    };
    ("sha2") => {
        true
    };
    ("sha3") => {
        unsafe {
            // `sha3` target feature implies SHA-512 as well
            $crate::aarch64_apple::sysctlbyname(b"hw.optional.armv8_2_sha512\0")
                && $crate::aarch64_appleaarch64::sysctlbyname(b"hw.optional.armv8_2_sha3\0")
        }
    };
    ("sm4") => {
        false
    };
}

/// Apple helper function for calling `sysctlbyname`.
pub unsafe fn sysctlbyname(name: &[u8]) -> bool {
    assert_eq!(
        name.last().cloned(),
        Some(0),
        "name is not NUL terminated: {:?}",
        name
    );

    let mut value: u32 = 0;
    let mut size = core::mem::size_of::<u32>();

    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr() as *const i8,
            &mut value as *mut _ as *mut libc::c_void,
            &mut size,
            core::ptr::null_mut(),
            0,
        )
    };

    assert_eq!(size, 4, "unexpected sysctlbyname(3) result size");
    assert_eq!(rc, 0, "sysctlbyname returned error code: {}", rc);
    value != 0
}
