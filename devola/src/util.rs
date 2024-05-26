use std::fs::File;
use std::path::Path;
use std::io::Read;
use crate::parser;
use crate::vm::{Devola, DevolaError};

pub fn read_from_file(path: &Path) -> String {
    let mut output = String::new();
    File::open(path).unwrap().read_to_string(&mut output).unwrap();

    output
}
pub fn execute_file(path: &str) -> Result<Devola, DevolaError> {
    let file = Path::new(path);
    let code = read_from_file(file);

    let (code, symbols) = parser::text::compile(code, None).unwrap();

    let mut devola = Devola::new(code, Some(symbols));
    devola.enable_debug();

    devola.run()?;
    Ok(devola)
}

pub fn build_u16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}
pub fn break_u16(word: u16) -> (u8, u8) {
    ((word >> 8) as u8, (word & 0x00FF) as u8)
}