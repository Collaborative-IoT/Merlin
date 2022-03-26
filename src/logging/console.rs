use ansi_term::Colour;
use anyhow::Result;
use std::fs::File;
use std::io::Read;

pub fn log_success(data: &str) {
    println!("[{}]:{}", Colour::Green.paint("SUCCESS"), data);
}

pub fn log_failure(data: &str) {
    println!("[{}]:{}", Colour::Red.paint("FAILURE"), data);
}

pub fn log_event(data: &str) {
    println!("[{}]:{}", Colour::Purple.paint("EVENT"), data);
}

/// Log our ascii knight to console
pub fn log_start(path: String) -> Result<()> {
    let mut file_result = File::open(path)?;
    let mut string_data = String::new();
    file_result.read_to_string(&mut string_data)?;
    println!("{}", string_data);
    Ok(())
}
