/// Zeroize `N` bytes of stack space.
///
/// Most algorithm implementations use stack to store temporary data.
/// This temporaries may contain sensitive information (e.g. cryptgraphic keys)
/// and can stay on stack after the computation is finished. If an attacker
/// is able for some reasons to read stack data freely, it may result in
/// leaking of the sensitive data.
///
/// # WARNING
/// This function requires you to estimate how much stack space is used by your
/// sensitive computation. This can be done by tools like [`cargo-call-stack`],
/// but note that stack usage depends on optimization level and compiler flags.
///
/// [`cargo-call-stack`]: https://github.com/japaric/cargo-call-stack
///
/// Additionally, you must annotate your sensitive function with `#[inline(never)]`.
///
/// For example, the following example **DOES NOT** erase stack properly:
/// ```ignore
/// pub fn encrypt_data(cipher: &Cipher, data: &mut [u8]) {
///     cipher.encrypt(data);
///     zeroize::zeroize_stack::<65_536>();
/// }
/// ```
/// The `cipher.encrypt` method may get inlined and `zeroize_stack` will erase
/// stack memory above stack frame reserved by `encrypt_data`. In other words,
/// it will **NOT** erase stack memory used by `cipher.encrypt`.
///
/// You should wrap your computation in the following way:
/// ```ignore
/// #[inline(never)]
/// fn encrypt_data_inner(cipher: &Cipher, data: &mut [u8]) {
///     cipher.encrypt(data);
/// }
///
/// pub fn encrypt_data(cipher: &Cipher, data: &mut [u8]) {
///     cipher.encrypt(data);
///     zeroize::zeroize_stack::<65_536>();
/// }
/// ```
/// Finally, note that `#[inline(never)]` is just a hint and may be ignored
/// by the compiler. It works properly in practice, but such stack zeroization
/// should be considered as "best effort" and in cases where it's not enough
/// you should inspect the generated binary to verify that you got a desired
/// codegen.
#[inline(never)]
pub fn zeroize_stack<const N: usize>() {
    let buf = [0u8; N];
    crate::optimization_barrier(&buf);
}
