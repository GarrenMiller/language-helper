use std::fs::File;
use std::io::{self, Read, Cursor, BufReader};

#[derive(Debug)]
struct HeaderProperty {
    name: String,
    value: String,
}

pub fn read_header() -> io::Result<()> {
    let file = File::open("src/morphology/dog.hfstol")?;
    let mut reader = BufReader::new(file);
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;

    println!("Modern HFST format: {}", is_modern_hfst(&magic));

    let mut data = [0u8; 2048];
    reader.read_exact(&mut data)?;
    let header_property = decode_magic_properties(&data);
    
    println!("Header property name: {:?}", header_property);
    Ok(())
}

fn is_modern_hfst(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && &bytes[0..4] == b"HFST"
}

fn decode_magic_properties(bytes: &[u8]) -> Result<HeaderProperty, String> {
    // TOOD: will need to pass in bytes at the right start, not sure how to find end
    // These are u16 name_len, name, u16 value_len, value
    let mut cursor = Cursor::new(bytes);
    let mut name_len_bytes = [0u8; 2]; 
    let _ = cursor.read(&mut name_len_bytes);
    let name_len = u16::from_le_bytes(name_len_bytes); 
    let mut name_bytes = vec![0u8; name_len.into()];
    let _ = cursor.read_exact(&mut name_bytes);
    let name = String::from_utf8(name_bytes);
    let mut value_len_bytes = [0u8; 2];
    let _ = cursor.read_exact(&mut value_len_bytes);
    let value_len = u16::from_le_bytes(value_len_bytes);
    let mut value_bytes = vec![0u8; value_len.into()];
    let _ = cursor.read_exact(&mut value_bytes);
    let value = String::from_utf8(value_bytes);

    match (name, value) {
        (Ok(name), Ok(value)) => Ok(HeaderProperty{ name: name, value: value }),
        _ => Err("Something went wrong trying to parse a value".to_string()),
    }
} 
