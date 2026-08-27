# [RustCrypto]: async `tokio::io`-compatibility wrappers for `digest`

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Async counterpart to [`digest-io`], providing `tokio::io`-compatible
wrappers for traits defined in the [`digest`] crate.

## Examples

Simultaneously reading and hashing file data asynchronously:
```rust,ignore
use digest_io_async::HashReader;
use sha2::Sha256;
use tokio::{fs::File, io};

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    // Create new hashing reader
    let f = File::open("Cargo.toml").await?;
    let mut reader = HashReader::<Sha256, File>::new(f);

    // Copy all data out of the file without buffering it up front
    let mut sink = Vec::new();
    io::copy(&mut reader, &mut sink).await?;

    // Get the resulting hash over the read data
    let hash = reader.finalize();
    println!("Hash: {hash:?}");
    Ok(())
}
```

Simultaneously hashing data and writing it to file asynchronously:
```rust,ignore
use digest_io_async::HashWriter;
use sha2::{Digest, Sha256};
use tokio::{fs::File, io::AsyncWriteExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    // Create new hashing writer
    let f = File::create("out.txt").await?;
    let mut writer = HashWriter::<Sha256, File>::new(f);

    // Write data to the file
    let data = b"Hello world!";
    writer.write_all(data).await?;

    // Get the resulting hash over written data
    let hash = writer.finalize();
    println!("{hash:?}");
    assert_eq!(hash, Sha256::digest(data));
    tokio::fs::remove_file("out.txt").await?;
    Ok(())
}
```

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/digest-io-async.svg
[crate-link]: https://crates.io/crates/digest-io-async
[docs-image]: https://docs.rs/digest-io-async/badge.svg
[docs-link]: https://docs.rs/digest-io-async/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/digest-io-async.yml/badge.svg?branch=master
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/digest-io-async.yml?query=branch:master

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
[`digest-io`]: https://docs.rs/digest-io
[`digest`]: https://docs.rs/digest
