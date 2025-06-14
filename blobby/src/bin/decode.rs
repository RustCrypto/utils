//! Encoding utility
use blobby::BlobIterator;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::{env, error::Error, fs::File};

fn encode_hex(data: &[u8]) -> String {
    let mut res = String::with_capacity(2 * data.len());
    for &byte in data {
        res.push_str(&format!("{byte:02X}"));
    }
    res
}

fn decode<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    let res = BlobIterator::new(&data)
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid blobby data: {e:?}"),
            )
        })?
        .collect::<Vec<_>>();
    for blob in res.iter() {
        let blob = blob.map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid blobby data: {e:?}"),
            )
        })?;
        writer.write_all(encode_hex(blob).as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(res.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!(
            "Blobby decoding utility.\n\
            Usage: decode <input blb file> <output text file>"
        );
        return Ok(());
    }

    let in_path = args[0].as_str();
    let out_path = args[1].as_str();
    let in_file = BufReader::new(File::open(in_path)?);
    let out_file = BufWriter::new(File::create(out_path)?);

    let n = decode(in_file, out_file)?;
    println!("Processed {n} record(s)");

    Ok(())
}
