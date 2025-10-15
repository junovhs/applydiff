use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

const LINE_COUNT: usize = 50_000;

fn main() {
    println!("Generating large files for test LF01-Replace-Start...");

    // ---- Create the 'before' file ----
    let before_path = Path::new("tests/LF01-Replace-Start/before/large_file.txt");
    let before_file = fs::File::create(before_path).expect("Could not create before file");
    let mut writer = BufWriter::new(before_file);

    writeln!(writer, "--- START OF FILE ---").unwrap(); // The line we will target
    for i in 1..LINE_COUNT {
        writeln!(writer, "line {}", i).unwrap();
    }
    writer.flush().unwrap();
    println!("  ✓ Created {}", before_path.display());


    // ---- Create the 'after' file ----
    let after_path = Path::new("tests/LF01-Replace-Start/after/large_file.txt");
    let after_file = fs::File::create(after_path).expect("Could not create after file");
    let mut writer = BufWriter::new(after_file);

    writeln!(writer, "--- PATCHED START OF FILE ---").unwrap(); // The expected result after patching
    for i in 1..LINE_COUNT {
        writeln!(writer, "line {}", i).unwrap();
    }
    writer.flush().unwrap();
    println!("  ✓ Created {}", after_path.display());
    println!("Done.");
}