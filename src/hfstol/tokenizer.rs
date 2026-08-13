use std::io::{BufRead, Seek};
use std::error::Error;
use crate::hfstol::header::HfstolProperties;

pub fn get_alphabet(mut reader: impl BufRead + Seek, hfstol_properties: HfstolProperties) -> Result<(), Box<dyn Error>> {
    let mut _count = 0;
    // let mut alphabet = Vec::new();

    let buffer = reader.fill_buf()?;

    let index = buffer.iter()
        .enumerate()
        .filter(|&(_, &b)| b == 0x00)
        .map(|(idx, _)| idx)
        .nth(hfstol_properties.num_input_symbols as usize)
        .unwrap();

    let alphabet_string = String::from_utf8(buffer[..index as usize].to_vec()).unwrap();
    let split_alphabet = alphabet_string
        .split("\0")
        .collect::<Vec<&str>>()
        .retain(|&s| !s.is_empty() && !s.contains("@"));

    println!("Alphabet string: {:?}", split_alphabet);

    Ok(())
}
