use std::io::{BufRead, BufReader};
use std::fs::File;
use std::error::Error;
use std::fmt;
use crate::hfstol::header::HfstolProperties;


pub struct Tokenizer {
    pub alphabet: Vec<Vec<u8>>,
    pub special_symbols: Vec<Vec<u8>>,
}

impl Tokenizer {
    pub fn new(reader: BufReader<File> , hfstol_properties: HfstolProperties) -> Result<Tokenizer, Box<dyn Error>> {
        let (alphabet, special_symbols) = parse_symbols(reader, hfstol_properties)?;
        Ok(
            Self {
                alphabet,
                special_symbols
            }
        )
    }
}

impl fmt::Display for Tokenizer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alphabet = self
            .alphabet
            .iter()
            .map(|v| str::from_utf8(&v).expect("Could not convert alphabet symbol to string from  UTF-8 bytes"))
            .collect::<Vec<&str>>();

        let symbols = self
            .special_symbols
            .iter()
            .map(|v| str::from_utf8(&v).expect("Could not convert special symbol to string from UTF-8 bytes"))
            .collect::<Vec<&str>>();

        write!(
            f,
            "Alphabet: {:?}, special_symbols: {:?}",
            alphabet,
            symbols,
        )
    }
}

pub fn parse_symbols(mut reader: BufReader<File>, hfstol_properties: HfstolProperties) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>), Box<dyn Error>> {
    let mut _count = 0;
    let mut alphabet: Vec<Vec<u8>> = Vec::new();
    let mut special_symbols: Vec<Vec<u8>> = Vec::new();

    // First '@' byte
    {
        let buffer = reader.fill_buf()?;
        let start = buffer.iter().position(|&b| b == b'@').unwrap();
        reader.consume(start);
    }

    loop {
        if _count == hfstol_properties.num_symbols {
            println!("Finished parsing symbols");
            break;
        }
        let buffer = reader.fill_buf()?;
        let null_byte = buffer.iter().position(|&b| b == 0).unwrap();

        let symbol = buffer[..null_byte].to_vec();

        if symbol[0] == b'@' || symbol[0] == b'^' {
            special_symbols.push(symbol);
        }
        else {
            alphabet.push(buffer[..null_byte].to_vec());
        }

        reader.consume(null_byte + 1);
        _count += 1;
    }

    Ok((alphabet, special_symbols))
}

pub fn parse_transition_index_table(mut reader: BufReader<File>) -> Result<(), Box<dyn Error>> {
    let buffer = reader.fill_buf()?;
    println!("First byte is: {:?}", buffer[0]);
    Ok(())
}
