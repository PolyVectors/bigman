use bigman::compiler::compiler;
use bigman::computer;

use std::error;
use std::fs;

use eyre::Result;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut computer = computer::Computer::new();
    computer.load_program(compiler::compile(fs::read_to_string("./main.asm")?.as_str()));

    Ok(())
}
