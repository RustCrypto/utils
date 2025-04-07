# [RustCrypto]: `std::io`-compatibility wrappers for `digest`

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Wrappers for compatibility between `std::io` and `digest` traits.

## Examples

Updating hash function by writing data into it:

```rust
use digest_io::IoWrapper;
use sha2::{Digest, Sha256};
use std::{fs::File, io};

fn main() -> io::Result<()> {
    // Wrap SHA-256 hash function
    let mut hasher = IoWrapper(Sha256::new());

    // Write contents of the file to the wrapped hasher
    let mut f = File::open("Cargo.toml")?;
    io::copy(&mut f, &mut hasher)?;

    // Get the resulting hash of the file data
    let hash = hasher.0.finalize();

    println!("{hash:?}");
    Ok(())
}
```

Reading data from a XOF reader:

```rust
use digest_io::IoWrapper;
use sha3::{Shake128, digest::ExtendableOutput};
use std::{fs::File, io};

fn read_array(r: &mut impl io::Read) -> io::Result<[u8; 64]> {
    let mut buf = [0u8; 64];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

fn main() -> io::Result<()> {
    // Create XOF reader
    let mut hasher = IoWrapper(Shake128::default());
    let mut f = File::open("Cargo.toml")?;
    io::copy(&mut f, &mut hasher)?;
    let reader = hasher.0.finalize_xof();

    // Wrap the reader and read data from it
    let mut reader = IoWrapper(reader);
    let buf = read_array(&mut reader)?;
    println!("{buf:?}");

    Ok(())
}
```

Simultaneously reading and hashing file data:
```rust
use digest_io::HashReader;
use sha2::Sha256;
use std::{
    fs::File,
    io::{self, Read},
};

fn main() -> io::Result<()> {
    // Create new hashing reader
    let f = File::open("Cargo.toml")?;
    let mut reader = HashReader::<Sha256, File>::new(f);

    // Read all data from the file
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    // Get the resulting hash over read data
    let hash = reader.finalize();
    println!("Data: {buf:?}");
    println!("Hash: {hash:?}");
    Ok(())
}
```

Simultaneously hashing data and writing it to file:
```rust
use digest_io::HashWriter;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Write},
};

fn main() -> io::Result<()> {
    // Create new hashing reader
    let f = File::create("out.txt")?;
    let mut writer = HashWriter::<Sha256, File>::new(f);

    // Write data to the file
    let data = b"Hello world!";
    writer.write_all(data)?;

    // Get the resulting hash over written data
    let hash = writer.finalize();
    println!("{hash:?}");
    assert_eq!(hash, Sha256::digest(data));
    std::fs::remove_file("out.txt")?;
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

[crate-image]: https://img.shields.io/crates/v/digest-io.svg
[crate-link]: https://crates.io/crates/digest-io
[docs-image]: https://docs.rs/digest-io/badge.svg
[docs-link]: https://docs.rs/digest-io/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/digest-io.yml/badge.svg?branch=master
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/digest-io.yml?query=branch:master

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
