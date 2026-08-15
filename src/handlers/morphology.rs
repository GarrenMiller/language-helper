use std::fs::File;
use std::io::{BufReader};

use crate::hfstol::header;
use crate::hfstol::tokenizer;



pub async fn load_analyzer_binary() {
    let file = File::open("hu.hfstol").unwrap();
    let mut reader = BufReader::new(file);
    let hfstol_header = header::read_hfstol_header(&mut reader).unwrap();
    println!("Header: {:?}", hfstol_header.1);
    let symbols = tokenizer::parse_symbols(reader, hfstol_header.1).unwrap();
    println!("Alphabet: {:?}", symbols.0);
    println!("Special symbosl: {:?}", symbols.1);
}
