use std::fs::File;
use std::io::{self, Read, Seek, BufReader};

#[derive(Debug)]
struct HeaderProperty {
    name: String,
    value: String,
}

pub fn read_header() -> io::Result<()> {
    let file = File::open("hu.hfstol")?;
    let mut reader = BufReader::new(file);
    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;

    println!("Modern HFST format: {}", is_modern_hfst(&magic));

    let header_property = decode_properties_section(reader);

    println!("Header property name: {:?}", header_property);
    Ok(())
}

fn is_modern_hfst(bytes: &[u8]) -> bool {
    bytes.len() >= 5 && &bytes[0..5] == b"HFST\0"
}

fn decode_properties_section<R: Read + Seek>(mut reader: BufReader<R>) {
    let mut length_bytes = [0u8; 2]; 
    let _ = reader.read(&mut length_bytes);
    let length = u16::from_le_bytes(length_bytes); 
    println!("property section length: {}", length);
    let _ = reader.seek_relative(1).unwrap();
    let mut properties = vec![0u8; length.into()];
    let _ = reader.read_exact(&mut properties);
    let result = String::from_utf8(properties);
    match result {
        Ok(result) => {
            let pairs = result.split('\0').collect::<Vec<&str>>();
            for pair in pairs.chunks(2) {
                println!("{:?}", pair)
            }
        },
        Err(e) => println!("Something went wrong: {:?}", e),
    }

} 
