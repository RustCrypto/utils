//! Tests for previous bugs in the implementation.

use cmov::CmovEq;

#[test]
fn u64_cmoveq() {
    let n = 0x8200_0000_0000_0000u64;
    let mut cond = 0u8;
    n.cmoveq(&0, 1u8, &mut cond);

    // 0x8200_0000_0000_0000 is not equal to 0
    assert_eq!(cond, 0);
}
