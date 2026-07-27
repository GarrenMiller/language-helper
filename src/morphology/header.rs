use std::fs::File;
use std::io::{self, Read};

pub fn read_header() -> io::Result<()> {
    let mut file = File::open("src/morphology/dog.hfstol")?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    println!("Modern HFST format: {}", is_modern_hfst(&magic));
    Ok(())
}

fn is_modern_hfst(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && &bytes[0..4] == b"HFST"
}
