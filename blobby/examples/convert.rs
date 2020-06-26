//! Convert utility
use std::{env, error::Error, fs::File};
use std::io::{self, Write, BufRead, BufReader, BufWriter};
use blobby::{BlobIterator};

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

fn encode(reader: impl BufRead, mut writer: impl Write)
    -> io::Result<usize>
{
    use std::collections::HashMap;
    use std::collections::hash_map::Entry::{Occupied, Vacant};

    let mut blobs: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut buf = [0u8; 4];
    let mut pos = 0;
    let mut recs = 0;
    for line in reader.lines() {
        let blob = hex::decode(line?.as_str())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        match blobs.entry(blob) {
            Occupied(e) => {
                // TODO: be smarter about short blobs, e.g.
                // store 1 byte blob directly, if reference VLQ
                // takes more than one byte.
                let n = (*e.get() << 1) + 1;
                let vlq = encode_vlq(n , &mut buf);
                writer.write_all(vlq)?;
                pos += vlq.len();
            }
            Vacant(e) => {
                let n = e.key().len();
                let vlq = encode_vlq(n << 1, &mut buf);
                let delta = vlq.len() + n;
                writer.write_all(vlq)?;
                writer.write_all(e.key())?;
                if n != 0 {
                    e.insert(pos);
                }
                pos += delta;
            }
        }
        recs += 1;
    }
    Ok(recs)
}

fn decode<R: BufRead, W: Write>(mut reader: R, mut writer: W)
    -> io::Result<usize>
{
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    let res: Vec<_> = BlobIterator::new(&data).collect();
    for blob in res.iter() {
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
