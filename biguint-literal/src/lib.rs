//! Crate 
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

use proc_macro::{Delimiter, TokenStream, TokenTree};

/// Strips any outer `Delimiter::None` groups from the input,
/// returning a `TokenStream` consisting of the innermost
/// non-empty-group `TokenTree`.
/// This is used to handle a proc macro being invoked
/// by a `macro_rules!` expansion.
/// See https://github.com/rust-lang/rust/issues/72545 for background
fn ignore_groups(mut input: TokenStream) -> TokenStream {
    let mut tokens = input.clone().into_iter();
    loop {
        if let Some(TokenTree::Group(group)) = tokens.next() {
            if group.delimiter() == Delimiter::None {
                input = group.stream();
                continue;
            }
        }
        return input;
    }
}

fn parse_bytes(input: TokenStream) -> Vec<u8> {
    let mut ts = ignore_groups(input).into_iter();
    let input_str = match (ts.next(), ts.next()) {
        (Some(TokenTree::Literal(literal)), None) => literal.to_string(),
        _ => panic!("expected single string literal"),
    };
    let buf: Vec<u8> = input_str.into();

    let buf = match buf.as_slice() {
        [b'"', b @ .., b'"'] => b,
        _ => panic!("expected single string literal"),
    };

    let mut iter = buf
        .iter()
        .filter(|&c| match c {
            b' ' | b'\r' | b'\n' | b'\t' => false,
            _ => true,
        })
        .map(|&c| match c {
            b'0'..=b'9' => c - 48,
            b'A'..=b'F' => c - 55,
            b'a'..=b'f' => c - 87,
            _ => panic!("encountered invalid character"),
        });
    let mut bytes = Vec::new();
    loop {
        let b = match iter.next() {
            Some(v) => v << 4,
            None => break,
        };
        match iter.next() {
            Some(v) => bytes.push(b + v),
            None => panic!("expected even number of characters"),
        }
    }
    bytes
}

fn assemble(bytes: &[u8], width: usize) -> String {
    let mut s = String::new();
    for chunk in bytes.chunks_exact(width).rev() {
        s.push_str("0x");
        for b in chunk {
            s.push_str(&format!("{:02X}", b));
        }
        s.push_str(", ");
    }
    s
}

/// Macro for converting string literal containing hex-encoded string
/// to an array containing decoded bytes
#[proc_macro]
pub fn hex_biguint(input: TokenStream) -> TokenStream {
    let bytes = parse_bytes(input);
    if bytes.len() % 8 != 0 {
        panic!("number of bytes must be mutliple of 8");
    }
    let n = bytes.len();
    let code = format!(
        "unsafe {{
            #[cfg(target_pointer_width = \"16\")]
            let t = core::mem::transmute::<[u16; {}], generic_array::GenericArray<u16, _>>([{}]);
            #[cfg(target_pointer_width = \"32\")]
            let t = core::mem::transmute::<[u32; {}], generic_array::GenericArray<u32, _>>([{}]);
            #[cfg(target_pointer_width = \"64\")]
            let t = core::mem::transmute::<[u64; {}], generic_array::GenericArray<u64, _>>([{}]);
            t
        }}",
        n / 2, assemble(&bytes, 2),
        n / 4, assemble(&bytes, 4),
        n / 8, assemble(&bytes, 8),
    );
    code.parse().unwrap()
}
