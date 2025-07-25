use super::{NEXT_MASK, VAL_MASK};

/// Write a git-flavoured VLQ value into `buf`.
///
/// Returns the slice within `buf` that holds the value.
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
///  - count of index entries=N
///  - N x index entries, each encoded as:
///      - size L of index entry (VLQ)
///      - index blob contents (L bytes)
///  - repeating encoded blobs, each encoded as:
///      - VLQ value that is either:
///         - (J << 1) & 0x01: indicates this blob is index entry J
///         - (L << 1) & 0x00: indicates an explicit blob of len L
///      - (in the latter case) explicit blob contents (L bytes)
pub fn encode_blobs<'a, I, T>(blobs: &'a I) -> (alloc::vec::Vec<u8>, usize)
where
    &'a I: IntoIterator<Item = &'a T>,
    T: AsRef<[u8]> + 'a,
{
    use alloc::{collections::BTreeMap, vec::Vec};

    let mut idx_map = BTreeMap::new();
    blobs
        .into_iter()
        .map(|v| v.as_ref())
        .filter(|blob| !blob.is_empty())
        .for_each(|blob| {
            let v = idx_map.entry(blob.as_ref()).or_insert(0);
            *v += 1;
        });

    let mut idx: Vec<&[u8]> = idx_map
        .iter()
        .filter(|&(_, &v)| v > 1)
        .map(|(&k, _)| k)
        .collect();
    idx.sort_by_key(|e| {
        let k = match e {
            [0] => 2,
            [1] => 1,
            _ => 0,
        };
        (k, idx_map.get(e).unwrap())
    });
    idx.reverse();
    let idx_len = idx.len();

    let rev_idx: BTreeMap<&[u8], usize> = idx.iter().enumerate().map(|(i, &e)| (e, i)).collect();

    let mut out_buf = Vec::new();
    let mut buf = [0u8; 4];
    out_buf.extend_from_slice(encode_vlq(idx.len(), &mut buf));
    for e in idx {
        out_buf.extend_from_slice(encode_vlq(e.len(), &mut buf));
        out_buf.extend_from_slice(e);
    }

    for blob in blobs.into_iter().map(|v| v.as_ref()) {
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
