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

pub const fn parse_dedup_len(mut data: &[u8]) -> Result<usize, Error> {
    read_vlq(&mut data)
}

pub const fn parse_items_len(mut data: &[u8]) -> Result<usize, Error> {
    let dedup_index_len = try_read_vlq!(data);

    let mut i = 0;
    while i < dedup_index_len {
        let m = try_read_vlq!(data);
        let split = data.split_at(m);
        data = split.1;
        i += 1;
    }

    let mut i = 0;
    loop {
        if data.is_empty() {
            return Ok(i);
        }
        let val = try_read_vlq!(data);
        // the least significant bit is used as a flag
        let is_ref = (val & 1) != 0;
        let val = val >> 1;
        if is_ref {
            if val >= dedup_index_len {
                return Err(Error::InvalidIndex);
            }
        } else {
            if val > data.len() {
                return Err(Error::UnexpectedEnd);
            }
            let split = data.split_at(val);
            data = split.1;
        };
        i += 1;
    }
}

/// Parse blobby data into an array.
pub const fn parse_into_array<const ITEMS: usize, const DEDUP_LEN: usize>(
    mut data: &[u8],
) -> Result<[&[u8]; ITEMS], Error> {
    if try_read_vlq!(data) != DEDUP_LEN {
        return Err(Error::BadArrayLen);
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

    let mut res: [&[u8]; ITEMS] = [&[]; ITEMS];

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
#[cfg(feature = "alloc")]
pub fn parse_into_vec(mut data: &[u8]) -> Result<alloc::vec::Vec<&[u8]>, Error> {
    use alloc::{vec, vec::Vec};

    let dedup_len = try_read_vlq!(data);

    let mut dedup_index: Vec<&[u8]> = vec![&[]; dedup_len];

    let mut i = 0;
    while i < dedup_index.len() {
        let m = try_read_vlq!(data);
        let split = data.split_at(m);
        dedup_index[i] = split.0;
        data = split.1;
        i += 1;
    }

    let items_len = parse_items_len(data)?;
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

#[macro_export]
macro_rules! parse_into_slice {
    ($data:expr) => {{
        const ITEMS_LEN: usize = {
            match $crate::parse_items_len($data) {
                Ok(v) => v,
                Err(_) => panic!("Failed to parse items len"),
            }
        };
        const DEDUP_LEN: usize = {
            match $crate::parse_dedup_len($data) {
                Ok(v) => v,
                Err(_) => panic!("Failed to parse dedup len"),
            }
        };
        const ITEMS: [&[u8]; ITEMS_LEN] = {
            match $crate::parse_into_array::<ITEMS_LEN, DEDUP_LEN>($data) {
                Ok(v) => v,
                Err(_) => panic!("Failed to parse items"),
            }
        };
        ITEMS.as_slice()
    }};
}

#[macro_export]
macro_rules! parse_into_structs {
    (
        $data:expr;
        $static_vis:vis static $items_name:ident;
        $ty_vis:vis struct $item:ident {
            $($field:ident),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq)]
        $ty_vis struct $item {
            pub $($field : &'static [u8]),*
        }

        $crate::parse_into_structs!(
            $data;
            $static_vis static $items_name;
            existing struct $item {
                $($field),*
            }
        );
    };

    (
        $data:expr;
        $static_vis:vis static $items_name:ident;
        existing struct $item:ident {
            $($field:ident),* $(,)?
        }
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
