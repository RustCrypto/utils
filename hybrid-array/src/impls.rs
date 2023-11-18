use super::{Array, ArrayOps, ArraySize, IntoArray};

#[cfg(feature = "zeroize")]
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(feature = "zeroize")]
impl<T, U> Zeroize for Array<T, U>
where
    T: Zeroize,
    U: ArraySize,
{
    fn zeroize(&mut self) {
        self.0.as_mut().iter_mut().zeroize()
    }
}

#[cfg(feature = "zeroize")]
impl<T, U> ZeroizeOnDrop for Array<T, U>
where
    T: ZeroizeOnDrop,
    U: ArraySize,
{
}

macro_rules! impl_array_size {
    ($($len:expr => $ty:ident),+) => {
        $(
            impl<T> ArrayOps<T, $len> for Array<T, typenum::$ty> {
                const SIZE: usize = $len;
                type Size = typenum::$ty;

                #[inline]
                fn as_core_array(&self) -> &[T; $len] {
                    &self.0
                }

                #[inline]
                fn as_mut_core_array(&mut self) -> &mut [T; $len] {
                    &mut self.0
                }

                #[inline]
                fn from_core_array(arr: [T; $len]) -> Self {
                    Self(arr)
                }

                #[inline]
                fn ref_from_core_array(array_ref: &[T; $len]) -> &Self {
                    // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; $len]`
                    unsafe { &*(array_ref.as_ptr() as *const Self) }
                }

                #[inline]
                fn ref_from_mut_core_array(array_ref: &mut [T; $len]) -> &mut Self {
                    // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; $len]`
                    unsafe { &mut *(array_ref.as_mut_ptr() as *mut Self) }
                }

                #[inline]
                fn map_to_core_array<F, U>(self, f: F) -> [U; $len]
                where
                    F: FnMut(T) -> U
                {
                    self.0.map(f)
                }
            }

            unsafe impl ArraySize for typenum::$ty {
                type ArrayType<T> = [T; $len];
            }

            impl<T> From<Array<T, typenum::$ty>> for [T; $len] {
                fn from(arr: Array<T, typenum::$ty>) -> [T; $len] {
                    arr.0
                }
            }

            impl<T> IntoArray<T> for [T; $len] {
                type Size = typenum::$ty;

                fn into_hybrid_array(self) -> Array<T, Self::Size> {
                    Array::from_core_array(self)
                }
            }
        )+
     };
}

impl_array_size! {
    0 => U0,
    1 => U1,
    2 => U2,
    3 => U3,
    4 => U4,
    5 => U5,
    6 => U6,
    7 => U7,
    8 => U8,
    9 => U9,
    10 => U10,
    11 => U11,
    12 => U12,
    13 => U13,
    14 => U14,
    15 => U15,
    16 => U16,
    17 => U17,
    18 => U18,
    19 => U19,
    20 => U20,
    21 => U21,
    22 => U22,
    23 => U23,
    24 => U24,
    25 => U25,
    26 => U26,
    27 => U27,
    28 => U28,
    29 => U29,
    30 => U30,
    31 => U31,
    32 => U32,
    33 => U33,
    34 => U34,
    35 => U35,
    36 => U36,
    37 => U37,
    38 => U38,
    39 => U39,
    40 => U40,
    41 => U41,
    42 => U42,
    43 => U43,
    44 => U44,
    45 => U45,
    46 => U46,
    47 => U47,
    48 => U48,
    49 => U49,
    50 => U50,
    51 => U51,
    52 => U52,
    53 => U53,
    54 => U54,
    55 => U55,
    56 => U56,
    57 => U57,
    58 => U58,
    59 => U59,
    60 => U60,
    61 => U61,
    62 => U62,
    63 => U63,
    64 => U64,
    96 => U96,
    128 => U128,
    192 => U192,
    256 => U256,
    384 => U384,
    448 => U448,
    512 => U512,
    768 => U768,
    896 => U896,
    1024 => U1024,
    2048 => U2048,
    4096 => U4096,
    8192 => U8192
}
