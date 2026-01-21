# [RustCrypto]: CPU bits selection

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Compile-time detection heuristics for the optimal word size to use for the
target CPU, which in some cases may differ from its address size a.k.a.
`target_pointer_width`.

Implemented as `macro_rules!`.

[Documentation][docs-link]

## Example

Below is a basic example of how you can use the `cpubits!` macro:

```rust
cpubits::cpubits! {
    16 => { pub type Word = u16; }
    32 => { pub type Word = u32; }
    64 => { pub type Word = u64; }
}
```

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

Includes portions from the `cfg-if` crate, which are also dual-licensed Apache 2.0 + MIT.
Copyright (c) 2014 Alex Crichton.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/cpubits.svg
[crate-link]: https://crates.io/crates/cpubits
[docs-image]: https://docs.rs/cpubits/badge.svg
[docs-link]: https://docs.rs/cpubits/
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/cpubits.yml/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/cpubits.yml
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
