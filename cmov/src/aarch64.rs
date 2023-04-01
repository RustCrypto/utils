use crate::{Cmov, CmovEq, Condition};
use core::arch::asm;

macro_rules! csel {
    ($csel:expr, $dst:expr, $src:expr, $condition:expr) => {
        unsafe {
            asm! {
                "cmp {0:w}, 0",
                $csel,
                in(reg) $condition,
                inlateout(reg) *$dst,
                in(reg) *$src,
                in(reg) *$dst,
                options(pure, nomem, nostack),
            };
        }
    };
}

macro_rules! csel_eq {
    ($instruction:expr, $lhs:expr, $rhs:expr, $condition:expr, $dst:expr) => {
        let mut tmp = *$dst as u16;
        unsafe {
            asm! {
                "eor {0:w}, {1:w}, {2:w}",
                "cmp {0:w}, 0",
                $instruction,
                out(reg) _,
                in(reg) *$lhs,
                in(reg) *$rhs,
                inlateout(reg) tmp,
                in(reg) $condition as u16,
                in(reg) tmp,
                options(pure, nomem, nostack),
            };
        };

        *$dst = tmp as u8;
    };
}

impl Cmov for u16 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        csel!("csel {1:w}, {2:w}, {3:w}, NE", self, value, condition);
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        csel!("csel {1:w}, {2:w}, {3:w}, EQ", self, value, condition);
    }
}

impl CmovEq for u16 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        csel_eq!("csel {3:w}, {4:w}, {5:w}, NE", self, rhs, input, output);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        csel_eq!("csel {3:w}, {4:w}, {5:w}, EQ", self, rhs, input, output);
    }
}

impl Cmov for u32 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        csel!("csel {1:w}, {2:w}, {3:w}, NE", self, value, condition);
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        csel!("csel {1:w}, {2:w}, {3:w}, EQ", self, value, condition);
    }
}

impl CmovEq for u32 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        csel_eq!("csel {3:w}, {4:w}, {5:w}, NE", self, rhs, input, output);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        csel_eq!("csel {3:w}, {4:w}, {5:w}, EQ", self, rhs, input, output);
    }
}

impl Cmov for u64 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        csel!("csel {1:x}, {2:x}, {3:x}, NE", self, value, condition);
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        csel!("csel {1:x}, {2:x}, {3:x}, EQ", self, value, condition);
    }
}

impl CmovEq for u64 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        csel_eq!("csel {3:w}, {4:w}, {5:w}, NE", self, rhs, input, output);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        csel_eq!("csel {3:w}, {4:w}, {5:w}, EQ", self, rhs, input, output);
    }
}
