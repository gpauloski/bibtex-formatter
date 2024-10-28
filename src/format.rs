use crate::models::Entry;
use crate::Result;
use std::fs::File;
use std::io::Write;

pub fn print_entries(entries: &[Entry]) {
    for (i, entry) in entries.iter().enumerate() {
        if i != 0 {
            println!();
        }
        println!("{}", entry);
    }
}

pub fn write_entries(entries: &[Entry], filepath: &str) -> Result<()> {
    let mut file = File::create(filepath)?;

    for (i, entry) in entries.iter().enumerate() {
        if i != 0 {
            file.write_all("\n\n".as_bytes())?;
        }
        write!(file, "{}", entry)?;
    }

    Ok(())
}
