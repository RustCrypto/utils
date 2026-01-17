use crate::{CtAssign, CtEq, CtOption};
use core::ops::AddAssign;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

#[cfg(doc)]
use core::ops::Index;

/// Constant-time lookup by index, similar to the [`Index`] trait, but returning an owned result in
/// constant-time.
pub trait CtLookup<Idx> {
    /// Output type returned by the lookup operation.
    type Output: CtAssign;

    /// Attempt to retrieve the item at the given `index`, either returning it or the [`CtOption`]
    /// equivalent of [`None`] if the `index` was out-of-bounds.
    fn ct_lookup(&self, index: Idx) -> CtOption<Self::Output>;
}

impl<T, Idx> CtLookup<Idx> for [T]
where
    T: CtAssign + Default,
    Idx: AddAssign + CtEq + Default + From<u8>,
{
    type Output = T;

    #[inline]
    fn ct_lookup(&self, index: Idx) -> CtOption<T> {
        let mut ret = CtOption::none();
        let mut i = Idx::default();

        for item in self {
            ret.insert_if(item, i.ct_eq(&index));

            // TODO(tarcieri): ideally we'd prevent overflow here but there's no core `CheckedAdd`
            i += Idx::from(1u8);
        }

        ret
    }
}

impl<T, Idx, const N: usize> CtLookup<Idx> for [T; N]
where
    T: CtAssign + Default,
    Idx: AddAssign + CtEq + Default + From<u8>,
{
    type Output = T;

    #[inline]
    fn ct_lookup(&self, index: Idx) -> CtOption<T> {
        self.as_slice().ct_lookup(index)
    }
}

#[cfg(feature = "alloc")]
impl<T, Idx> CtLookup<Idx> for Box<[T]>
where
    T: CtAssign + Default,
    Idx: AddAssign + CtEq + Default + From<u8>,
{
    type Output = T;

    #[inline]
    fn ct_lookup(&self, index: Idx) -> CtOption<T> {
        (**self).ct_lookup(index)
    }
}

#[cfg(feature = "alloc")]
impl<T, Idx> CtLookup<Idx> for Vec<T>
where
    T: CtAssign + Default,
    Idx: AddAssign + CtEq + Default + From<u8>,
{
    type Output = T;

    #[inline]
    fn ct_lookup(&self, index: Idx) -> CtOption<T> {
        self.as_slice().ct_lookup(index)
    }
}

#[cfg(test)]
mod tests {
    mod array {
        use crate::CtLookup;

        const EXAMPLE: [u8; 3] = [1, 2, 3];

        #[test]
        fn ct_lookup_u32() {
            assert_eq!(EXAMPLE.ct_lookup(0u32).unwrap(), 1);
            assert_eq!(EXAMPLE.ct_lookup(1u32).unwrap(), 2);
            assert_eq!(EXAMPLE.ct_lookup(2u32).unwrap(), 3);
            assert!(EXAMPLE.ct_lookup(3u32).is_none().to_bool());
            assert!(EXAMPLE.ct_lookup(4u32).is_none().to_bool());
        }

        // usize only has a `CtEq` impl on 32-bit and 64-bit targets currently
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        #[test]
        fn ct_lookup_usize() {
            assert_eq!(EXAMPLE.ct_lookup(0usize).unwrap(), 1);
            assert_eq!(EXAMPLE.ct_lookup(1usize).unwrap(), 2);
            assert_eq!(EXAMPLE.ct_lookup(2usize).unwrap(), 3);
            assert!(EXAMPLE.ct_lookup(3usize).is_none().to_bool());
            assert!(EXAMPLE.ct_lookup(4usize).is_none().to_bool());
        }
    }

    mod slice {
        use crate::CtLookup;

        const EXAMPLE: &[u8] = &[1, 2, 3];

        #[test]
        fn ct_lookup_u32() {
            assert_eq!(EXAMPLE.ct_lookup(0u32).unwrap(), 1);
            assert_eq!(EXAMPLE.ct_lookup(1u32).unwrap(), 2);
            assert_eq!(EXAMPLE.ct_lookup(2u32).unwrap(), 3);
            assert!(EXAMPLE.ct_lookup(3u32).is_none().to_bool());
            assert!(EXAMPLE.ct_lookup(4u32).is_none().to_bool());
        }

        // usize only has a `CtEq` impl on 32-bit and 64-bit targets currently
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        #[test]
        fn ct_lookup_usize() {
            assert_eq!(EXAMPLE.ct_lookup(0usize).unwrap(), 1);
            assert_eq!(EXAMPLE.ct_lookup(1usize).unwrap(), 2);
            assert_eq!(EXAMPLE.ct_lookup(2usize).unwrap(), 3);
            assert!(EXAMPLE.ct_lookup(3usize).is_none().to_bool());
            assert!(EXAMPLE.ct_lookup(4usize).is_none().to_bool());
        }
    }
}
