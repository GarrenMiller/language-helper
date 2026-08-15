use std::io::{BufRead, Seek};
use std::error::Error;
use crate::hfstol::header::HfstolProperties;

pub fn parse_symbols(mut reader: impl BufRead + Seek, hfstol_properties: HfstolProperties) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>), Box<dyn Error>> {
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
