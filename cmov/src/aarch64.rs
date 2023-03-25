use crate::{Cmov, Condition};
use core::arch::asm;

macro_rules! csel {
    ($cmp:expr, $csel:expr, $dst:expr, $src:expr, $condition:expr) => {
        unsafe {
            asm! {
                $cmp,
                $csel,
                in(reg) $condition,
                inlateout(reg) *$dst,
                in(reg) $src,
                in(reg) *$dst,
                options(pure, nomem, nostack),
            };
        }
    };
}

impl Cmov for u16 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        csel!(
            "cmp {0:w}, 0",
            "csel {1:w}, {2:w}, {3:w}, NE",
            self,
            *value,
            condition
        );
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        csel!(
            "cmp {0:w}, 0",
            "csel {1:w}, {2:w}, {3:w}, EQ",
            self,
            *value,
            condition
        );
    }
}

impl Cmov for u32 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        csel!(
            "cmp {0:w}, 0",
            "csel {1:w}, {2:w}, {3:w}, NE",
            self,
            *value,
            condition
        );
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        csel!(
            "cmp {0:w}, 0",
            "csel {1:w}, {2:w}, {3:w}, EQ",
            self,
            *value,
            condition
        );
    }
}

impl Cmov for u64 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        csel!(
            "cmp {0:x}, 0",
            "csel {1:x}, {2:x}, {3:x}, NE",
            self,
            *value,
            condition
        );
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        csel!(
            "cmp {0:x}, 0",
            "csel {1:x}, {2:x}, {3:x}, EQ",
            self,
            *value,
            condition
        );
    }
}
