//! # `NanoCore`
//!
//! (c) 2025 Afaan Bilal <https://afaan.dev>
//!
//! `NanoCore` is a meticulously crafted emulator for a custom, true 8-bit CPU.
//!
//! Designed with extreme minimalism in mind, this CPU operates within a strict
//! 256-byte memory space, with all registers, the Program Counter (PC), and
//! the Stack Pointer (SP) being 8-bit.
//!
//! This project serves as an educational exercise in understanding the
//! fundamental principles of computer architecture, low-level instruction
//! set design, memory management under severe constraints, and assembly
//! language programming.
//!

use std::fs;

use clap::Parser;
use nanocore::{assembler::Assembler, end_color, start_color};

#[derive(Parser, Debug)]
#[command(name = "assembler")]
#[command(about = "Assembles NanoCore ASM (.nca) file into binary output (.ncb)", long_about = None)]
struct Args {
    /// Path to the source assembly file
    #[arg(short, long)]
    input: String,

    /// Path to the output binary file
    #[arg(short, long, default_value = "out.ncb")]
    output: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let asm = fs::read_to_string(&args.input)?;
    print!("\nAssembling:\n  Input: ");
    start_color();
    print!("{}", args.input);
    end_color();
    print!("\n Output: ");
    start_color();
    print!("{}", args.output);
    end_color();
    println!();

    let mut c = Assembler::default();

    if let Err(e) = c.assemble(&asm) {
        eprintln!("Error assembling '{}': {}", args.input, e);
        std::process::exit(1);
    }

    print!("Assembled. Writing to bin.");

    fs::write(&args.output, &c.program)?;

    println!("\nDone.");

    Ok(())
}
