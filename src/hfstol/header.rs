use std::fs::File;
use std::io::{Read, BufReader, BufRead};
use std::error::Error;
use deku::prelude::*;

#[derive(Debug)]
pub struct HfstolMetadata {
    name: String,
    value: String,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct HfstolProperties {
    pub num_input_symbols: u16,
    pub num_symbols: u16,
    transition_index_size: u32,
    transition_table_size: u32,
    num_states: u32,
    num_transitions: u32,
    multichar_symbol_length: u32,
    flags: u32,
    property_mask: u32
}

pub fn read_hfstol_header(reader: &mut BufReader<File>) -> Result<(Vec<HfstolMetadata>, HfstolProperties), Box<dyn Error>> {
    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;

    let is_modern = is_modern_hfstol(&magic);

    if !is_modern {
        return Err("Could not parse the HFST-OL file as version 3.1+; it's probably an older version that's not supported.".into());
    }

    let header_property = decode_hfstol_metadata(reader)?;
    let hfst_header = decode_hfstol_properties(reader)?;
    Ok((header_property, hfst_header))
}

fn is_modern_hfstol(bytes: &[u8]) -> bool {
    bytes.len() >= 5 && &bytes[0..5] == b"HFST\0"
}

fn decode_hfstol_metadata(reader: &mut BufReader<File>) -> Result<Vec<HfstolMetadata>, Box<dyn Error>> {
    let (index, props_str) = {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            return Err("The file buffer was empty when decoding metadata; is the file correct?".into())
        }
        let index = u16::from_le_bytes([buffer[0], buffer[1]]);
        let properties = &buffer[3..index as usize];
        (index, str::from_utf8(properties))
    };

    let split = props_str?.split('\0').collect::<Vec<&str>>(); // Remove separators

    let result = split 
        .chunks(2)
        .filter_map(|pair| match pair {
            [name, value] => Some(HfstolMetadata { 
                name: name.to_string(),
                value: value.to_string() 
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    reader.consume(index as usize + 3);
    return Ok(result);
} 

fn decode_hfstol_properties(reader: &mut BufReader<File>) -> Result<HfstolProperties, Box<dyn Error>> {
    let buffer = reader.fill_buf()?;
    let (_, result) = HfstolProperties::from_bytes((&buffer[..32], 0)).unwrap();
    reader.consume(32);
    return Ok(result);
}
