use hex_literal::hex;

#[test]
fn single_line_comments() {
    assert_eq!(hex!("dd 03 // comment"), [0xdd, 0x03]);
    assert_eq!(
        hex!(
            "00 04 f0 // a comment here
            54 fe // another comment"
        ),
        [0x00, 0x04, 0xf0, 0x54, 0xfe]
    );
    assert_eq!(
        hex!(
            "// initial comment
            01 02"
        ),
        [0x01, 0x02]
    );
}

#[test]
fn block_comments() {
    assert_eq!(
        hex!("00 01 02 /* intervening comment */ 03 04"),
        [0x00, 0x01, 0x02, 0x03, 0x04]
    );
    assert_eq!(hex!("/* initial comment */ ff df dd"), [0xff, 0xdf, 0xdd]);
    assert_eq!(
        hex!(
            "8f ff 7d /*
            comment
            on
            several
            lines
            */
            d0 a3"
        ),
        [0x8f, 0xff, 0x7d, 0xd0, 0xa3]
    );
}
