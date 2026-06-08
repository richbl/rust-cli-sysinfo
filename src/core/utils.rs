use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// `read_first_line()` reads the first line of a file at the given path
///
pub fn read_first_line(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut line = String::new();

    io::BufReader::new(file).read_line(&mut line).ok()?;

    let len = line.trim_end().len();
    line.truncate(len);

    Some(line)
}

/// `read_hex_u16()` reads a hexadecimal value from a file and returns it as a u16
///
pub fn read_hex_u16(path: &Path) -> Option<u16> {
    let contents = fs::read_to_string(path).ok()?;
    let value = contents.trim().trim_start_matches("0x");

    u16::from_str_radix(value, 16).ok()
}
