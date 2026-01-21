use super::{Error, NEXT_MASK, VAL_MASK};

pub(crate) const fn read_vlq(data: &mut &[u8]) -> Result<usize, Error> {
    let b = match data.split_first() {
        Some((&b, rest)) => {
            *data = rest;
            b
        }
        None => return Err(Error::UnexpectedEnd),
    };
    let mut next = b & NEXT_MASK;
    let mut val = (b & VAL_MASK) as usize;

    macro_rules! step {
        () => {
            if next == 0 {
                return Ok(val);
            }
            let b = match data.split_first() {
                Some((&b, rest)) => {
                    *data = rest;
                    b
                }
                None => return Err(Error::UnexpectedEnd),
            };

            next = b & NEXT_MASK;
            let t = (b & VAL_MASK) as usize;
            val = ((val + 1) << 7) + t;
        };
    }

    step!();
    step!();
    step!();

    if next != 0 {
        return Err(Error::InvalidVlq);
    }

    Ok(val)
}

macro_rules! try_read_vlq {
    ($data:expr) => {
        match read_vlq(&mut $data) {
            Ok(v) => v,
            Err(err) => return Err(err),
        }
    };
}

/// Blobby file header
#[derive(Clone, Copy, Debug)]
pub struct Header {
    /// Number of blobs stored in the file
    pub items_len: usize,
    /// Number of deduplicated blobs
    pub dedup_len: usize,
}

impl Header {
    /// Parse blobby header.
    ///
    /// # Errors
    /// If data could not be parsed successfully.
    pub const fn parse(data: &mut &[u8]) -> Result<Self, Error> {
        match (read_vlq(data), read_vlq(data)) {
            (Ok(items_len), Ok(dedup_len)) => Ok(Header {
                items_len,
                dedup_len,
            }),
            (Err(err), _) | (Ok(_), Err(err)) => Err(err),
        }
    }
}

/// Parse blobby data into an array.
///
/// # Errors
/// If data could not be parsed successfully.
pub const fn parse_into_array<const ITEMS_LEN: usize, const DEDUP_LEN: usize>(
    mut data: &[u8],
) -> Result<[&[u8]; ITEMS_LEN], Error> {
    match Header::parse(&mut data) {
        Ok(header) => {
            if header.items_len != ITEMS_LEN || header.dedup_len != DEDUP_LEN {
                return Err(Error::BadArrayLen);
            }
        }
        Err(err) => return Err(err),
    }

    let mut dedup_index: [&[u8]; DEDUP_LEN] = [&[]; DEDUP_LEN];

    let mut i = 0;
    while i < dedup_index.len() {
        let m = try_read_vlq!(data);
        let split = data.split_at(m);
        dedup_index[i] = split.0;
        data = split.1;
        i += 1;
    }

    let mut res: [&[u8]; ITEMS_LEN] = [&[]; ITEMS_LEN];

    let mut i = 0;
    while i < res.len() {
        let val = try_read_vlq!(data);
        // the least significant bit is used as a flag
        let is_ref = (val & 1) != 0;
        let val = val >> 1;
        res[i] = if is_ref {
            if val >= dedup_index.len() {
                return Err(Error::InvalidIndex);
            }
            dedup_index[val]
        } else {
            if val > data.len() {
                return Err(Error::UnexpectedEnd);
            }
            let split = data.split_at(val);
            data = split.1;
            split.0
        };
        i += 1;
    }

    if data.is_empty() {
        Ok(res)
    } else {
        Err(Error::BadArrayLen)
    }
}

/// Parse blobby data into a vector of slices.
///
/// # Errors
/// If data failed to parse successfully
#[cfg(feature = "alloc")]
#[allow(clippy::missing_panics_doc)]
pub fn parse_into_vec(mut data: &[u8]) -> Result<alloc::vec::Vec<&[u8]>, Error> {
    use alloc::{vec, vec::Vec};

    let Header {
        items_len,
        dedup_len,
    } = Header::parse(&mut data)?;

    let mut dedup_index: Vec<&[u8]> = vec![&[]; dedup_len];

    let mut i = 0;
    while i < dedup_index.len() {
        let m = try_read_vlq!(data);
        let split = data.split_at(m);
        dedup_index[i] = split.0;
        data = split.1;
        i += 1;
    }

    let mut res: Vec<&[u8]> = vec![&[]; items_len];

    let mut i = 0;
    while i < res.len() {
        let val = try_read_vlq!(data);
        // the least significant bit is used as a flag
        let is_ref = (val & 1) != 0;
        let val = val >> 1;
        res[i] = if is_ref {
            if val >= dedup_index.len() {
                return Err(Error::InvalidIndex);
            }
            dedup_index[val]
        } else {
            if val > data.len() {
                return Err(Error::UnexpectedEnd);
            }
            let split = data.split_at(val);
            data = split.1;
            split.0
        };
        i += 1;
    }

    assert!(data.is_empty());
    Ok(res)
}

/// Parse data into a slice.
#[macro_export]
macro_rules! parse_into_slice {
    ($data:expr) => {{
        const HEADER: $crate::Header = {
            let mut data: &[u8] = $data;
            match $crate::Header::parse(&mut data) {
                Ok(v) => v,
                Err(_) => panic!("Failed to parse items len"),
            }
        };
        const ITEMS: [&[u8]; { HEADER.items_len }] = {
            match $crate::parse_into_array::<{ HEADER.items_len }, { HEADER.dedup_len }>($data) {
                Ok(v) => v,
                Err(_) => panic!("Failed to parse items"),
            }
        };
        ITEMS.as_slice()
    }};
}

/// Parse data into structs.
#[macro_export]
macro_rules! parse_into_structs {
    (
        $data:expr;
        #[define_struct]
        $static_vis:vis static $items_name:ident: &[
            $ty_vis:vis $item:ident { $($field:ident),* $(,)? }
        ];
    ) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq)]
        $ty_vis struct $item {
            pub $($field : &'static [u8]),*
        }

        $crate::parse_into_structs!(
            $data;
            $static_vis static $items_name: &[
                $item { $($field),* }
            ];
        );
    };

    (
        $data:expr;
        $static_vis:vis static $items_name:ident: &[
            $item:ident { $($field:ident),* $(,)? }
        ];
    ) => {
        $static_vis static $items_name: &[$item] = {
            const RAW_ITEMS: &[&[u8]] = $crate::parse_into_slice!($data);

            const fn get_struct(items: &mut &[&'static [u8]]) -> $item {
                $item {
                    $($field: {
                        match items.split_first() {
                            Some((first, rest)) => {
                                *items = rest;
                                first
                            }
                            None => unreachable!(),
                        }
                    }),*
                }
            }

            const ITEM_FIELDS: usize = 0 $( + {
                let $field: (); let _ = $field;
                1
            })*;

            const ITEMS_LEN: usize = if RAW_ITEMS.len() % ITEM_FIELDS == 0 {
                RAW_ITEMS.len() / ITEM_FIELDS
            } else {
                panic!("Number of raw items is not multiple of number of fields in the struct");
            };

            const ITEMS: [$item; ITEMS_LEN] = {
                let mut res = [$item { $($field : &[]),* }; ITEMS_LEN];

                let mut raw_items = RAW_ITEMS;
                let mut i = 0;
                while i < res.len() {
                    res[i] = get_struct(&mut raw_items);
                    i += 1;
                }
                res
            };

            ITEMS.as_slice()
        };
    };
}
