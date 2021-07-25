//! Convert utility
use blobby::{encode_blobs, BlobIterator};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::{env, error::Error, fs::File};

fn encode(reader: impl BufRead, mut writer: impl Write) -> io::Result<usize> {
    let mut blobs = Vec::new();
    for line in reader.lines() {
        let blob = hex::decode(line?.as_str())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        blobs.push(blob);
    }
    let (data, idx_len) = encode_blobs(&blobs);
    let data_len = data.len();
    println!("Index len: {:?}", idx_len);
    writer.write_all(&data).map(|_| data_len)
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
