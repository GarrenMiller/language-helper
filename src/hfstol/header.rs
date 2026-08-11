use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, BufReader, BufRead};
use std::error::Error;
use deku::prelude::*;

#[derive(Debug)]
struct HfstMetadata {
    name: String,
    value: String,
}

#[derive(Debug, DekuRead, DekuWrite)]
struct HfstPropertyHeader {
    num_input_symbols: u16,
    num_symbols: u16,
    transition_index_size: u32,
    transition_table_size: u32,
    num_states: u32,
    num_transitions: u32,
    multichar_symbol_length: u32,
    flags: u32,
    property_mask: u32
}

pub fn read_header() -> io::Result<()> {
    let file = File::open("hu.hfstol")?;
    let mut reader = BufReader::new(file);
    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;

    println!("Modern HFST format: {}", is_modern_hfst(&magic));

    let header_property = decode_hfst_metadata_header(&mut reader);
    println!("Metadata: {:?}", header_property);
    let hfst_header = decode_hfst_property_header(reader);
    println!("Header Props: {:?}", hfst_header);
    Ok(())
}

fn is_modern_hfst(bytes: &[u8]) -> bool {
    bytes.len() >= 5 && &bytes[0..5] == b"HFST\0"
}

fn decode_hfst_metadata_header(mut reader: impl BufRead + Seek) -> Result<Vec<HfstMetadata>, Box<dyn Error>> {
    let (index, props_str) = {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            eprintln!("The file buffer was empty when decoding properties");
        }
        let index = u16::from_le_bytes([buffer[0], buffer[1]]);
        let properties = &buffer[3..index as usize];
        (index, str::from_utf8(properties))
    };

    let split = props_str?.split('\0').collect::<Vec<&str>>(); // Remove separators

    let result = split 
        .chunks(2)
        .filter_map(|pair| match pair {
            [name, value] => Some(HfstMetadata { 
                name: name.to_string(),
                value: value.to_string() 
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    reader.consume(index as usize + 3);
    return Ok(result);
} 

fn decode_hfst_property_header(mut reader: impl BufRead + Seek) -> Result<HfstPropertyHeader, Box<dyn Error>> {
    let buffer = reader.fill_buf()?;
    let ((remaining_bytes, bit_offset), result) = HfstPropertyHeader::from_bytes((&buffer[..32], 0)).unwrap();
    return Ok(result);
}
