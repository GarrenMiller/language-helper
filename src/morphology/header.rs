use std::fs::File;
use std::io::{self, Read, Seek, BufReader, BufRead};
use std::error::Error;

#[derive(Debug)]
struct HfstMetadata {
    name: String,
    value: String,
}

pub fn read_header() -> io::Result<()> {
    let file = File::open("hu.hfstol")?;
    let mut reader = BufReader::new(file);
    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;

    println!("Modern HFST format: {}", is_modern_hfst(&magic));

    let header_property = decode_hfst_metadata_header(reader);

    println!("Header property name: {:?}", header_property);
    Ok(())
}

fn is_modern_hfst(bytes: &[u8]) -> bool {
    bytes.len() >= 5 && &bytes[0..5] == b"HFST\0"
}

fn decode_hfst_metadata_header(mut reader: impl BufRead + Seek) -> Result<Vec<HfstMetadata>, Box<dyn Error>> {
    let buffer = reader.fill_buf()?;
    if buffer.is_empty() {
        eprintln!("The file buffer was empty when decoding properties");
    }

    let length = u16::from_le_bytes([buffer[0], buffer[1]]);
    let offset = length + 3; // Skip null byte separator
    
    let properties = &buffer[3..offset as usize]; 
    let props_str = str::from_utf8(properties);
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
    reader.consume(offset as usize);
    return Ok(result);
} 
