# [RustCrypto]: CryptoPAN IP Address Anonymization

![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Anonymizes IP addresses using the CryptoPAN algorithm tightly based on the GO implementation by Yawning Angel (https://github.com/Yawning/cryptopan), which is based on the original reference implementation [paper by J. Fan, J. Xu, M. Ammar, and S. Moon. (https://ieeexplore.ieee.org/abstract/document/1181415)]

CryptoPAN is a prefix-preserving, 1-1 mapping algorithm that allows for consistent anonymization of IP addresses across datasets, provided that the same 256-bit key is used. 

IPv6 anonymization is supported, but it is not known if the code conforms to the reference implementation. 

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.41+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto


