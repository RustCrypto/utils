#[macro_use]
extern crate proc_macro_hack;

#[inline(always)]
fn is_hex_char(c: &char) -> bool {
    match *c {
        '0'...'9' | 'a'...'f' | 'A'...'F' => true,
        _ => false,
    }
}

#[inline(always)]
fn is_format_char(c: &char) -> bool {
    match *c {
        ' ' | '\r' | '\n' | '\t' => true,
        _ => false,
    }
}

proc_macro_expr_impl! {
    pub fn hex_impl(input: &str) -> String {
        let bytes = input.as_bytes();
        let n = bytes.len();
        if bytes[0] != b'"' || bytes[n-1] != b'"' {
            panic!("expected string literal as an input");
        }
        let input = &input[1..n-1];

        input.chars().for_each(|c| {
            if !(is_hex_char(&c) || is_format_char(&c)) {
                panic!("invalid character: {:?}", c);
            }
        });
        let n = input.chars().filter(is_hex_char).count() / 2;
        let mut s = String::with_capacity(2 + 7*n);

        s.push('[');
        let mut iter = input.chars().filter(is_hex_char);
        loop {
            let c1 = match iter.next() { Some(c) => c, None => break };
            let c2 = iter.next().unwrap_or_else(|| panic!(
                "expected even number of hex character"));
            s += "0x";
            s.push(c1);
            s.push(c2);
            s += "u8,";
        }
        s.push(']');

        s
    }
}
