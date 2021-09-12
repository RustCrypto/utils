use crate::limb::{Inner, SignedInner, BIT_SIZE};

const HI_BIT: usize = BIT_SIZE - 1;

/// Returns all 1's if `a`!=0 or 0 if a==0
#[inline]
pub(crate) const fn is_nonzero(a: Inner) -> Inner {
    let a = a as SignedInner;
    ((a | -a) >> HI_BIT) as Inner
}

/// Return `a` if `c`!=0 or `b` if `c`==0
#[inline]
pub(crate) const fn ct_select(a: Inner, b: Inner, c: Inner) -> Inner {
    a ^ (c & (a ^ b))
}
