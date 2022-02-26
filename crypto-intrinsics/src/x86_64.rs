//! `x86_64` intrinsics

use core::arch::asm;

/// Move if zero.
///
/// Uses a `test` instruction to check if the given `condition` value is
/// equal to zero, then calls `cmovz` (a.k.a. `cmove`) to conditionally move
/// `src` to `dst` when `condition` is equal to zero.
#[inline(always)]
pub fn cmovz(condition: u64, src: u64, dst: &mut u64) {
    unsafe {
        asm! {
            "test {0}, {0}",
            "cmovz {1}, {2}",
            in(reg) condition,
            inlateout(reg) *dst,
            in(reg) src
        };
    }
}

/// Move if not zero.
///
/// Uses a `test` instruction to check if the given `condition` value is not
/// equal to zero, then calls `cmovnz` (a.k.a. `cmovne`) to conditionally move
/// `src` to `dst` when `condition` is nonzero.
#[inline(always)]
pub fn cmovnz(condition: u64, src: u64, dst: &mut u64) {
    unsafe {
        asm! {
            "test {0}, {0}",
            "cmovnz {1}, {2}",
            in(reg) condition,
            inlateout(reg) *dst,
            in(reg) src
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmovz_works() {
        let mut n = 24;
        cmovz(42, 42, &mut n);
        assert_eq!(n, 24);
        cmovz(0, 42, &mut n);
        assert_eq!(n, 42);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 24;
        cmovnz(0, 42, &mut n);
        assert_eq!(n, 24);
        cmovnz(42, 42, &mut n);
        assert_eq!(n, 42);
    }
}
