//! Convert
extern crate hex;
extern crate blobby;
extern crate byteorder;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{Write, BufRead, BufReader, BufWriter};
use std::{u8, u16, u32};

use byteorder::{LE, WriteBytesExt};
use blobby::BlobIterator;

fn encode<R: BufRead, W: Write>(reader: R, mut writer: W)
    -> io::Result<usize>
{
    let mut res = Vec::new();
    for line in reader.lines() {
        let blob = hex::decode(line?.as_str())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        res.push(blob);
    }
    let n = match res.iter().map(|b| b.len()).max() {
        None => 1,
        Some(m) if m <= u8::MAX as usize => 1,
        Some(m) if m <= u16::MAX as usize => 2,
        Some(m) if m <= u32::MAX as usize => 4,
        _ => 8,
    };

    writer.write_all(b"blobby")?;
    writer.write_all(format!("{}", n).as_bytes())?;

    for blob in res.iter() {
        let s = blob.len();
        match n {
            1 => writer.write_all(&[s as u8])?,
            2 => writer.write_u16::<LE>(s as u16)?,
            4 => writer.write_u32::<LE>(s as u32)?,
            8 => writer.write_u64::<LE>(s as u64)?,
            _ => unreachable!(),
        }
        writer.write_all(blob)?;
    }
    Ok(res.len())
}

fn decode<R: BufRead, W: Write>(mut reader: R, mut writer: W)
    -> io::Result<usize>
{
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    let res: Vec<_> = BlobIterator::new(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .collect();
    for blob in res.iter() {
        writer.write_all(hex::encode(blob).as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(res.len())
}

fn main() -> Result<(), Box<Error>> {
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

    println!("Processed {} record(s)", n);;

    Ok(())
}
