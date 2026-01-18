//! Encoding utility
use core::error::Error;

#[cfg(not(feature = "alloc"))]
fn main() -> Result<(), Box<dyn Error>> {
    Err("The encode binary should be compiled with enabled `alloc` feature!".into())
}

#[cfg(feature = "alloc")]
fn main() -> Result<(), Box<dyn Error>> {
    use blobby::encode_blobs;
    use std::io::{self, BufRead, BufReader, BufWriter, Write};
    use std::{env, fs::File};

    fn decode_hex_char(b: u8) -> io::Result<u8> {
        let res = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => {
                let msg = "Invalid hex string: invalid byte {b}";
                return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
            }
        };
        Ok(res)
    }

    fn decode_hex(data: &str) -> io::Result<Vec<u8>> {
        if data.len() % 2 != 0 {
            let msg = "Invalid hex string: length is not even";
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }
        data.as_bytes()
            .chunks_exact(2)
            .map(|chunk| {
                let a = decode_hex_char(chunk[0])?;
                let b = decode_hex_char(chunk[1])?;
                Ok((a << 4) | b)
            })
            .collect()
    }

    fn encode(reader: impl BufRead, mut writer: impl Write) -> io::Result<usize> {
        let mut blobs = Vec::new();
        for line in reader.lines() {
            let blob = decode_hex(&line?)?;
            blobs.push(blob);
        }
        let (data, idx_len) = encode_blobs(&blobs);
        println!("Index len: {idx_len:?}");
        writer.write_all(&data)?;
        Ok(blobs.len())
    }

    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!(
            "Blobby encoding utility.\n\
            Usage: encode <input txt file> <output blb file>"
        );
        return Ok(());
    }

    let in_path = args[0].as_str();
    let out_path = args[1].as_str();
    let in_file = BufReader::new(File::open(in_path)?);
    let out_file = BufWriter::new(File::create(out_path)?);

    let n = encode(in_file, out_file)?;
    println!("Processed {n} record(s)");

    Ok(())
}
