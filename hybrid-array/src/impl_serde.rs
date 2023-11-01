// This file was modified from
// https://github.com/fizyk20/generic-array/blob/0e2a03714b05bb7a737a677f8df77d6360d19c99/src/impl_serde.rs

use crate::{Array, ArraySize};
use core::{fmt, marker::PhantomData};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

impl<T, N: ArraySize> Serialize for Array<T, N>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(N::USIZE)?;
        for el in self {
            tup.serialize_element(el)?;
        }

        tup.end()
    }
}

struct ArrayVisitor<T, N> {
    _t: PhantomData<T>,
    _n: PhantomData<N>,
}

// to avoid extra computation when testing for extra elements in the sequence
struct Dummy;
impl<'de> Deserialize<'de> for Dummy {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Dummy)
    }
}

impl<'de, T, N: ArraySize> Visitor<'de> for ArrayVisitor<T, N>
where
    T: Deserialize<'de>,
{
    type Value = Array<T, N>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "struct Array<T, U{}>", N::USIZE)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Array<T, N>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Check the length in advance
        match seq.size_hint() {
            Some(n) if n != N::USIZE => {
                return Err(de::Error::invalid_length(n, &self));
            }
            _ => {}
        }

        // Deserialize the array
        let arr = Array::try_from_fn(|idx| {
            let next_elem_opt = seq.next_element()?;
            next_elem_opt.ok_or(de::Error::invalid_length(idx, &self))
        });

        // If there's a value allegedly remaining, and deserializing it doesn't fail, then that's a
        // length mismatch error
        if seq.size_hint() != Some(0) && seq.next_element::<Dummy>()?.is_some() {
            Err(de::Error::invalid_length(N::USIZE + 1, &self))
        } else {
            arr
        }
    }
}

impl<'de, T, N: ArraySize> Deserialize<'de> for Array<T, N>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Array<T, N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = ArrayVisitor {
            _t: PhantomData,
            _n: PhantomData,
        };
        deserializer.deserialize_tuple(N::USIZE, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let array = Array::<u8, typenum::U2>::default();
        let serialized = bincode::serialize(&array);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_deserialize() {
        let mut array = Array::<u8, typenum::U2>::default();
        array[0] = 1;
        array[1] = 2;
        let serialized = bincode::serialize(&array).unwrap();
        let deserialized = bincode::deserialize::<Array<u8, typenum::U2>>(&serialized);
        assert!(deserialized.is_ok());
        let array = deserialized.unwrap();
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
    }

    #[test]
    fn test_serialized_size() {
        let array = Array::<u8, typenum::U1>::default();
        let size = bincode::serialized_size(&array).unwrap();
        assert_eq!(size, 1);
    }

    #[test]
    #[should_panic]
    fn test_too_many() {
        let serialized = "[1, 2, 3, 4, 5]";
        let _ = serde_json::from_str::<Array<u8, typenum::U4>>(serialized).unwrap();
    }
}
