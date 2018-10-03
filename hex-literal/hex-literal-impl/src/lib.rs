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
            return "compile_error!(\"expected string literal\")".to_string();
        }
        let input = &input[1..n-1];

        for (i, c) in input.chars().enumerate() {
            if !(is_hex_char(&c) || is_format_char(&c)) {
                return format!("compile_error!(\"\
                    invalid character (position {}): {:?}\")", i + 1, c);
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
                return "compile_error!(\"\
                    expected even number of hex characters\")".to_string();
            }
        }
        s.push(']');

        s
    }
}
