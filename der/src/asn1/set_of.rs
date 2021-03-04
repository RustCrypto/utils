//! ASN.1 `SET OF` support.

use crate::{
    Any, ByteSlice, Decodable, Decoder, Encodable, Encoder, Error, ErrorKind, Length, Result, Tag,
    Tagged,
};
use core::{convert::TryFrom, marker::PhantomData};

/// ASN.1 `SET OF` denotes a collection of zero or more occurrences of a
/// given type.
///
/// When encoded as DER, `SET OF` is lexicographically ordered.
pub trait SetOf<'a, T>: Decodable<'a> + Encodable
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    /// Iterator over the elements of the set
    type Iter: Iterator<Item = T>;

    /// Iterate over the elements of the set
    fn elements(&self) -> Self::Iter;
}

/// ASN.1 `SET OF` backed by a byte slice containing serialized DER.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    /// DER-encoded byte slice
    inner: ByteSlice<'a>,

    /// Set element type
    element_type: PhantomData<T>,
}

impl<'a, T> SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    /// Create a new [`SetOfRef`] from a slice.
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        let inner = ByteSlice::new(slice).map_err(|_| ErrorKind::Length { tag: Self::TAG })?;

        let mut decoder = Decoder::new(slice);
        let mut last_value = None;

        // Validate that we can decode all elements in the slice, and that they
        // are lexicographically ordered according to DER's rules
        while !decoder.is_finished() {
            let value: T = decoder.decode()?;

            if let Some(last) = last_value.as_ref() {
                if last >= &value {
                    return Err(ErrorKind::Noncanonical.into());
                }
            }

            last_value = Some(value);
        }

        Ok(Self {
            inner,
            element_type: PhantomData,
        })
    }

    /// Borrow the inner byte sequence.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }
}

impl<'a, T> AsRef<[u8]> for SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a, T> TryFrom<Any<'a>> for SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Tag::Set)?;
        Self::new(any.as_bytes())
    }
}

impl<'a, T> From<SetOfRef<'a, T>> for Any<'a>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    fn from(set: SetOfRef<'a, T>) -> Any<'a> {
        Any {
            tag: Tag::Set,
            value: set.inner,
        }
    }
}

impl<'a, T> Encodable for SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    fn encoded_len(&self) -> Result<Length> {
        Any::from(self.clone()).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(self.clone()).encode(encoder)
    }
}

impl<'a, T> Tagged for SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    const TAG: Tag = Tag::Set;
}

impl<'a, T> SetOf<'a, T> for SetOfRef<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    type Iter = SetOfRefIter<'a, T>;

    fn elements(&self) -> Self::Iter {
        SetOfRefIter::new(self)
    }
}

/// Iterator over the elements of an [`SetOfRef`].
pub struct SetOfRefIter<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    /// Decoder which iterates over the elements of the message
    decoder: Decoder<'a>,

    /// Element type
    element_type: PhantomData<T>,
}

impl<'a, T> SetOfRefIter<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    pub(crate) fn new(set: &SetOfRef<'a, T>) -> Self {
        Self {
            decoder: Decoder::new(set.as_bytes()),
            element_type: PhantomData,
        }
    }
}

impl<'a, T> Iterator for SetOfRefIter<'a, T>
where
    T: Clone + Decodable<'a> + Encodable + Ord,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.decoder.is_finished() {
            None
        } else {
            Some(
                self.decoder
                    .decode()
                    .expect("SetOfRef decodable invariant violated"),
            )
        }
    }
}
