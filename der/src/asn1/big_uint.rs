//! "Big" ASN.1 `INTEGER` types.

use crate::{
    Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged,
};
use core::{convert::TryFrom, marker::PhantomData};
use typenum::Unsigned;

/// "Big" unsigned ASN.1 `INTEGER` type.
///
/// Provides direct access to the underlying big endian bytes which comprise an
/// unsigned integer value.
///
/// Intended for use cases like very large integers that are used in
/// cryptographic applications (e.g. keys, signatures).
///
/// Generic over a `Size` value (e.g. [`der::consts::U64`][`typenum::U64`]),
/// indicating the size of an integer in bytes.
///
/// Currently supported sizes are 1 - 512 bytes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(docsrs, doc(cfg(feature = "big-uint")))]
pub struct BigUInt<'a, N: BigUIntSize> {
    /// Inner value
    inner: ByteSlice<'a>,

    /// Integer size in bytes
    size: PhantomData<N>,
}

impl<'a, N: BigUIntSize> BigUInt<'a, N> {
    /// Create a new [`BigUInt`] from a byte slice.
    ///
    /// Slice may be less than or equal to `N` bytes.
    pub fn new(mut bytes: &'a [u8]) -> Result<Self> {
        // Remove leading zeroes
        while bytes.get(0).cloned() == Some(0) {
            bytes = &bytes[1..];
        }

        if bytes.len() > N::to_usize() {
            return Err(ErrorKind::Length { tag: Self::TAG }.into());
        }

        ByteSlice::new(bytes)
            .map(|inner| Self {
                inner,
                size: PhantomData,
            })
            .map_err(|_| ErrorKind::Length { tag: Self::TAG }.into())
    }

    /// Borrow the inner byte slice which contains the least significant bytes
    /// of a big endian integer value with all leading zeros stripped, and may
    /// be any length from empty (i.e. zero) to `N` bytes.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    /// Get the length of this [`BigUInt`] in bytes.
    pub fn len(&self) -> Length {
        self.inner.len()
    }

    /// Is the inner byte slice empty?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the length of the inner integer value when encoded.
    fn inner_len(self) -> Result<Length> {
        self.len()
            + match self.inner.as_ref().get(0).cloned() {
                Some(n) if n >= 0x80 => 1u8, // Needs leading `0`
                None => 1u8,                 // Needs leading `0`
                _ => 0u8,                    // No leading `0`
            }
    }

    /// Get the ASN.1 DER [`Header`] for this [`BigUint`] value
    fn header(self) -> Result<Header> {
        Ok(Header {
            tag: Tag::Integer,
            length: self.inner_len()?,
        })
    }
}

impl<'a, N: BigUIntSize> From<&BigUInt<'a, N>> for BigUInt<'a, N> {
    fn from(value: &BigUInt<'a, N>) -> BigUInt<'a, N> {
        *value
    }
}

impl<'a, N: BigUIntSize> TryFrom<Any<'a>> for BigUInt<'a, N> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BigUInt<'a, N>> {
        any.tag().assert_eq(Tag::Integer)?;
        let mut bytes = any.as_bytes();

        // Disallow a leading byte which would overflow a signed
        // ASN.1 integer (since this is a "uint" type).
        // We expect all such cases to have a leading `0x00` byte
        // (see comment below)
        if let Some(byte) = bytes.get(0).cloned() {
            if byte > 0x80 {
                return Err(ErrorKind::Value { tag: Self::TAG }.into());
            }
        }

        // The `INTEGER` type always encodes a signed value, so for unsigned
        // values the leading `0x00` byte may need to be removed.
        // TODO(tarcieri): validate leading 0 byte was required
        if bytes.len() > N::to_usize() {
            if bytes.len() != N::to_usize().checked_add(1).unwrap() {
                return Err(ErrorKind::Length { tag: Self::TAG }.into());
            }

            if bytes.get(0).cloned() != Some(0) {
                return Err(ErrorKind::Value { tag: Self::TAG }.into());
            }

            bytes = &bytes[1..];
        }

        Self::new(bytes)
    }
}

impl<'a, N: BigUIntSize> Encodable for BigUInt<'a, N> {
    fn encoded_len(&self) -> Result<Length> {
        self.header()?.encoded_len()? + self.inner_len()?
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.header()?.encode(encoder)?;

        // Add leading `0x00` byte if required
        if self.inner_len()? > self.len() {
            encoder.byte(0)?;
        }

        encoder.bytes(self.as_bytes())
    }
}

impl<'a, N: BigUIntSize> Tagged for BigUInt<'a, N> {
    const TAG: Tag = Tag::Integer;
}

/// Marker trait for allowed [`BigUInt`] sizes.
#[cfg_attr(docsrs, doc(cfg(feature = "big-uint")))]
pub trait BigUIntSize: Unsigned {}

macro_rules! impl_size {
    ($($int:ident),+) => {
        $(impl BigUIntSize for typenum::consts::$int {})+
    };
}

// Sizes supported by the current implementation (1 - 512 bytes)
impl_size!(
    U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U13, U14, U15, U16, U17, U18, U19, U20, U21,
    U22, U23, U24, U25, U26, U27, U28, U29, U30, U31, U32, U33, U34, U35, U36, U37, U38, U39, U40,
    U41, U42, U43, U44, U45, U46, U47, U48, U49, U50, U51, U52, U53, U54, U55, U56, U57, U58, U59,
    U60, U61, U62, U63, U64, U65, U66, U67, U68, U69, U70, U71, U72, U73, U74, U75, U76, U77, U78,
    U79, U80, U81, U82, U83, U84, U85, U86, U87, U88, U89, U90, U91, U92, U93, U94, U95, U96, U97,
    U98, U99, U100, U101, U102, U103, U104, U105, U106, U107, U108, U109, U110, U111, U112, U113,
    U114, U115, U116, U117, U118, U119, U120, U121, U122, U123, U124, U125, U126, U127, U128, U129,
    U130, U131, U132, U133, U134, U135, U136, U137, U138, U139, U140, U141, U142, U143, U144, U145,
    U146, U147, U148, U149, U150, U151, U152, U153, U154, U155, U156, U157, U158, U159, U160, U161,
    U162, U163, U164, U165, U166, U167, U168, U169, U170, U171, U172, U173, U174, U175, U176, U177,
    U178, U179, U180, U181, U182, U183, U184, U185, U186, U187, U188, U189, U190, U191, U192, U193,
    U194, U195, U196, U197, U198, U199, U200, U201, U202, U203, U204, U205, U206, U207, U208, U209,
    U210, U211, U212, U213, U214, U215, U216, U217, U218, U219, U220, U221, U222, U223, U224, U225,
    U226, U227, U228, U229, U230, U231, U232, U233, U234, U235, U236, U237, U238, U239, U240, U241,
    U242, U243, U244, U245, U246, U247, U248, U249, U250, U251, U252, U253, U254, U255, U256, U257,
    U258, U259, U260, U261, U262, U263, U264, U265, U266, U267, U268, U269, U270, U271, U272, U273,
    U274, U275, U276, U277, U278, U279, U280, U281, U282, U283, U284, U285, U286, U287, U288, U289,
    U290, U291, U292, U293, U294, U295, U296, U297, U298, U299, U300, U301, U302, U303, U304, U305,
    U306, U307, U308, U309, U310, U311, U312, U313, U314, U315, U316, U317, U318, U319, U320, U321,
    U322, U323, U324, U325, U326, U327, U328, U329, U330, U331, U332, U333, U334, U335, U336, U337,
    U338, U339, U340, U341, U342, U343, U344, U345, U346, U347, U348, U349, U350, U351, U352, U353,
    U354, U355, U356, U357, U358, U359, U360, U361, U362, U363, U364, U365, U366, U367, U368, U369,
    U370, U371, U372, U373, U374, U375, U376, U377, U378, U379, U380, U381, U382, U383, U384, U385,
    U386, U387, U388, U389, U390, U391, U392, U393, U394, U395, U396, U397, U398, U399, U400, U401,
    U402, U403, U404, U405, U406, U407, U408, U409, U410, U411, U412, U413, U414, U415, U416, U417,
    U418, U419, U420, U421, U422, U423, U424, U425, U426, U427, U428, U429, U430, U431, U432, U433,
    U434, U435, U436, U437, U438, U439, U440, U441, U442, U443, U444, U445, U446, U447, U448, U449,
    U450, U451, U452, U453, U454, U455, U456, U457, U458, U459, U460, U461, U462, U463, U464, U465,
    U466, U467, U468, U469, U470, U471, U472, U473, U474, U475, U476, U477, U478, U479, U480, U481,
    U482, U483, U484, U485, U486, U487, U488, U489, U490, U491, U492, U493, U494, U495, U496, U497,
    U498, U499, U500, U501, U502, U503, U504, U505, U506, U507, U508, U509, U510, U511, U512
);

#[cfg(test)]
mod tests {
    use super::BigUInt;
    use crate::{Any, ErrorKind, Result, Tag};
    use core::convert::TryInto;

    // TODO(tarcieri): tests for different integer sizes
    type BigU1<'a> = BigUInt<'a, typenum::U1>;

    /// Parse a `BitU1` from an ASN.1 `Any` value to test decoding behaviors.
    fn parse_bigu1_from_any(bytes: &[u8]) -> Result<BigU1<'_>> {
        Any::new(Tag::Integer, bytes)?.try_into()
    }

    #[test]
    fn decode_empty() {
        let x = parse_bigu1_from_any(&[]).unwrap();
        assert_eq!(x.as_bytes(), &[]);
    }

    #[test]
    fn decode_zero() {
        let x = parse_bigu1_from_any(&[0]).unwrap();
        assert_eq!(x.as_bytes(), &[]);
    }

    #[test]
    fn decode_leading_extra_zero() {
        let x = parse_bigu1_from_any(&[0x00, 0x81]).unwrap();
        assert_eq!(x.as_bytes(), &[0x81]);
    }

    #[test]
    fn reject_oversize_without_extra_zero() {
        let err = parse_bigu1_from_any(&[0x81]).err().unwrap();
        assert_eq!(err.kind(), ErrorKind::Value { tag: Tag::Integer });
    }
}
