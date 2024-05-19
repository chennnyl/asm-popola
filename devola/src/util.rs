use std::fs::File;
use std::path::Path;
use std::io::Read;
pub fn read_from_file(path: &Path) -> String {
    let mut output = String::new();
    File::open(path).unwrap().read_to_string(&mut output).unwrap();

    output
}