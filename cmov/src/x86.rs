//! From "AMD64 Architecture Programmer’s Manual, Volume 1: Application Programming"
//! (Rev. 3.23 - October 2020) page 46:
//!
//! <https://www.amd.com/system/files/TechDocs/24592.pdf>
//!
//! > The CMOVcc instructions conditionally copy a word, doubleword, or
//! > quadword from a register or memory location to a register location.
//! > The source and destination must be of the same size.

use crate::{Cmov, Condition};
use core::arch::asm;

macro_rules! cmov {
    ($instruction:expr, $dst:expr, $src:expr, $condition:expr) => {
        unsafe {
            asm! {
                "test {0}, {0}",
                $instruction,
                in(reg_byte) $condition,
                inlateout(reg) *$dst,
                in(reg) $src,
                options(pure, nomem, nostack),
            };
        }
    };
}

impl Cmov for u16 {
    #[inline(always)]
    fn cmovz(&mut self, value: Self, condition: Condition) {
        cmov!("cmovz {1:e}, {2:e}", self, value, condition);
    }

    #[inline(always)]
    fn cmovnz(&mut self, value: Self, condition: Condition) {
        cmov!("cmovnz {1:e}, {2:e}", self, value, condition);
    }
}

impl Cmov for u32 {
    #[inline(always)]
    fn cmovz(&mut self, value: Self, condition: Condition) {
        cmov!("cmovz {1:e}, {2:e}", self, value, condition);
    }

    #[inline(always)]
    fn cmovnz(&mut self, value: Self, condition: Condition) {
        cmov!("cmovnz {1:e}, {2:e}", self, value, condition);
    }
}

impl Cmov for u64 {
    #[inline(always)]
    fn cmovz(&mut self, value: Self, condition: Condition) {
        cmov!("cmovz {1:r}, {2:r}", self, value, condition);
    }

    #[inline(always)]
    fn cmovnz(&mut self, value: Self, condition: Condition) {
        cmov!("cmovnz {1:r}, {2:r}", self, value, condition);
    }
}
