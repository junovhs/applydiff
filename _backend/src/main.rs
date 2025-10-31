#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

// The main binary now just calls the library's main function.
fn main() {
    applydiff_backend::main();
}