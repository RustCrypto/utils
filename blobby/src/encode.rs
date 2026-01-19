use super::{NEXT_MASK, VAL_MASK};

/// Write a git-flavoured VLQ value into `buf`.
///
/// Returns the slice within `buf` that holds the value.
#[allow(clippy::cast_possible_truncation)]
fn encode_vlq(mut val: usize, buf: &mut [u8; 4]) -> &[u8] {
    macro_rules! step {
        ($n:expr) => {
            buf[$n] = if $n == 3 {
                (val & (VAL_MASK as usize)) as u8
            } else {
                val -= 1;
                NEXT_MASK | (val & (VAL_MASK as usize)) as u8
            };
            val >>= 7;
            if val == 0 {
                return &buf[$n..];
            }
        };
    }

    step!(3);
    step!(2);
    step!(1);
    step!(0);
    panic!("integer is too big")
}

/// Encode the given collection of binary blobs in .blb format into `writer`.
/// Returns the encoded data together with a count of the number of blobs included in the index.
///
/// The encoded file format is:
///  - number of blobs in the file = N
///  - number of deduplicated index entries = M
///  - M x index entries encoded as:
///      - size L of index entry (VLQ)
///      - index blob contents (L bytes)
///  - N x blobs encoded as:
///      - VLQ value that is either:
///         - (J << 1) & 0x01: indicates this blob is index entry J
///         - (L << 1) & 0x00: indicates an explicit blob of len L
///      - (in the latter case) explicit blob contents (L bytes)
#[allow(clippy::missing_panics_doc, clippy::unwrap_used)]
pub fn encode_blobs<T>(blobs: &[T]) -> (alloc::vec::Vec<u8>, usize)
where
    T: AsRef<[u8]>,
{
    use alloc::{collections::BTreeMap, vec::Vec};

    let mut dedup_map = BTreeMap::new();
    blobs
        .iter()
        .map(AsRef::as_ref)
        .filter(|blob| !blob.is_empty())
        .for_each(|blob| {
            let v = dedup_map.entry(blob.as_ref()).or_insert(0);
            *v += 1;
        });

    let mut dedup_list: Vec<&[u8]> = dedup_map
        .iter()
        .filter(|&(_, &v)| v > 1)
        .map(|(&k, _)| k)
        .collect();
    dedup_list.sort_by_key(|e| {
        let k = match e {
            [0] => 2,
            [1] => 1,
            _ => 0,
        };
        (k, dedup_map.get(e).unwrap())
    });
    dedup_list.reverse();
    let idx_len = dedup_list.len();

    let rev_idx: BTreeMap<&[u8], usize> = dedup_list
        .iter()
        .enumerate()
        .map(|(i, &e)| (e, i))
        .collect();

    let mut out_buf = Vec::new();
    let mut buf = [0u8; 4];

    out_buf.extend_from_slice(encode_vlq(blobs.len(), &mut buf));
    out_buf.extend_from_slice(encode_vlq(dedup_list.len(), &mut buf));

    for e in dedup_list {
        out_buf.extend_from_slice(encode_vlq(e.len(), &mut buf));
        out_buf.extend_from_slice(e);
    }

    for blob in blobs.iter().map(AsRef::as_ref) {
        if let Some(dup_pos) = rev_idx.get(blob) {
            let n = (dup_pos << 1) + 1usize;
            out_buf.extend_from_slice(encode_vlq(n, &mut buf));
        } else {
            let n = blob.len() << 1;
            out_buf.extend_from_slice(encode_vlq(n, &mut buf));
            out_buf.extend_from_slice(blob);
        }
    }

    (out_buf, idx_len)
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::unwrap_used)]
mod tests {
    use crate::{Error, NEXT_MASK, VAL_MASK, decode::read_vlq};

    fn encode_vlq(mut val: usize, buf: &mut [u8; 4]) -> &[u8] {
        macro_rules! step {
            ($n:expr) => {
                buf[$n] = if $n == 3 {
                    (val & (VAL_MASK as usize)) as u8
                } else {
                    val -= 1;
                    NEXT_MASK | (val & (VAL_MASK as usize)) as u8
                };
                val >>= 7;
                if val == 0 {
                    return &buf[$n..];
                }
            };
        }

        step!(3);
        step!(2);
        step!(1);
        step!(0);
        panic!("integer is too big")
    }

    #[test]
    fn encode_decode() {
        let mut buf = [0u8; 4];
        for val in 0..=270549119 {
            let mut res = encode_vlq(val, &mut buf);
            let val_res = read_vlq(&mut res).unwrap();
            assert_eq!(val, val_res);
            assert!(res.is_empty());
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_vlq() {
        let mut example_buf: &[u8] = &[
            0b0000_0000, // 0
            0b0000_0010, // 2
            0b0111_1111, // 127
            0b1000_0000, 0b0000_0000, // 128
            0b1111_1111, 0b0111_1111, // 16511
            0b1000_0000, 0b1000_0000, 0b0000_0000, // 16512
            0b1111_1111, 0b1111_1111, 0b0111_1111, // 2113663
            0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0000, // 2113664
            0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111, // 270549119
            0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111,
        ];

        let targets = [
            (0, 1),
            (2, 1),
            (127, 1),
            (128, 2),
            (16511, 2),
            (16512, 3),
            (2113663, 3),
            (2113664, 4),
            (270549119, 4),
        ];

        let mut buf = [0u8; 4];

        let mut rem_len = example_buf.len();

        for (target_val, target_size) in targets {
            assert_eq!(encode_vlq(target_val, &mut buf), &example_buf[..target_size]);

            let val = read_vlq(&mut example_buf).unwrap();
            assert_eq!(val, target_val);

            rem_len -= target_size;
            assert_eq!(example_buf.len(), rem_len);
            
        }
        
        // Only VLQ values of up to 4 bytes are supported
        assert_eq!(read_vlq(&mut example_buf), Err(Error::InvalidVlq));
    }
}
