use crate::hfstol::header;



pub async fn load_analyzer_binary() {
    let result = header::read_hfstol_header();
    match result {
        Ok(result) => {
           println!("Loaded morphological analyzer file: {:?}", result);
        },
        Err(e) => eprintln!("Error loading morphological analyzer file: {}", e)
    }
}
