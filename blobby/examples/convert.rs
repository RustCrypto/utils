//! Convert utility
use blobby::BlobIterator;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::{env, error::Error, fs::File};

const NEXT_MASK: u8 = 0b1000_0000;
const VAL_MASK: u8 = 0b0111_1111;

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

fn encode(reader: impl BufRead, mut writer: impl Write) -> io::Result<usize> {
    let mut blobs = Vec::new();
    for line in reader.lines() {
        let blob = hex::decode(line?.as_str())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        blobs.push(blob);
    }

    let mut idx_map = HashMap::new();
    for blob in blobs.iter().filter(|b| b.len() != 0) {
        let v = idx_map.entry(blob.as_slice()).or_insert(0);
        *v += 1;
    }

    let mut idx: Vec<&[u8]> = idx_map
        .iter()
        .filter(|(_, &v)| v > 1)
        .map(|(&k, _)| k)
        .collect();
    idx.sort_by_key(|e| {
        let k = match e {
            &[0] => 2,
            &[1] => 1,
            _ => 0,
        };
        (k, idx_map.get(e).unwrap())
    });
    idx.reverse();

    let rev_idx: HashMap<&[u8], usize> = idx.iter().enumerate().map(|(i, &e)| (e, i)).collect();

    println!("Index len: {:?}", idx.len());
    let mut buf = [0u8; 4];
    writer.write_all(encode_vlq(idx.len(), &mut buf))?;
    for e in idx {
        writer.write_all(encode_vlq(e.len(), &mut buf))?;
        writer.write_all(e)?;
    }

    for blob in blobs.iter() {
        if let Some(dup_pos) = rev_idx.get(blob.as_slice()) {
            let n = (dup_pos << 1) + 1usize;
            writer.write_all(encode_vlq(n, &mut buf))?;
        } else {
            let n = blob.len() << 1;
            writer.write_all(encode_vlq(n, &mut buf))?;
            writer.write_all(blob)?;
        }
    }

    Ok(blobs.len())
}

fn decode<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    let res: Vec<_> = BlobIterator::new(&data)
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid blobby data: {:?}", e),
            )
        })?
        .collect();
    for blob in res.iter() {
        let blob = blob.map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid blobby data: {:?}", e),
            )
        })?;
        writer.write_all(hex::encode(blob).as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(res.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let is_encode = match args[0].as_str() {
        "encode" => true,
        "decode" => false,
        _ => Err("unknown mode")?,
    };
    let in_path = args[1].as_str();
    let out_path = args[2].as_str();
    let in_file = BufReader::new(File::open(in_path)?);
    let out_file = BufWriter::new(File::create(out_path)?);

    let n = if is_encode {
        encode(in_file, out_file)?
    } else {
        decode(in_file, out_file)?
    };

    println!("Processed {} record(s)", n);

    Ok(())
}
