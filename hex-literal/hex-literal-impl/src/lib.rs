extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree, Delimiter};
use proc_macro_hack::proc_macro_hack;

fn is_hex_char(c: &char) -> bool {
    match *c {
        '0'..='9' | 'a'..='f' | 'A'..='F' => true,
        _ => false,
    }
}

fn is_format_char(c: &char) -> bool {
    match *c {
        ' ' | '\r' | '\n' | '\t' => true,
        _ => false,
    }
}


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

#[proc_macro_hack]
pub fn hex(mut input: TokenStream) -> TokenStream {
    input = ignore_groups(input);
    let mut ts = input.into_iter();
    let input = match (ts.next(), ts.next()) {
        (Some(TokenTree::Literal(literal)), None) => literal.to_string(),
        _ => panic!("expected one string literal"),
    };

    let bytes = input.as_bytes();
    let n = bytes.len();
    // trim quote characters
    let input = &input[1..n-1];

    for (i, c) in input.chars().enumerate() {
        if !(is_hex_char(&c) || is_format_char(&c)) {
            panic!("invalid character (position {}): {:?})", i + 1, c);
        }
    };
    let n = input.chars().filter(is_hex_char).count() / 2;
    let mut s = String::with_capacity(2 + 7*n);

    s.push('[');
    let mut iter = input.chars().filter(is_hex_char);
    while let Some(c1) = iter.next() {
        if let Some(c2) = iter.next() {
            s += "0x";
            s.push(c1);
            s.push(c2);
            s += "u8,";
        } else {
            panic!("expected even number of hex characters");
        }
    }
    s.push(']');

    s.parse().unwrap()
}
