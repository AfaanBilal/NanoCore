#![deny(clippy::all)]
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

use clap::Parser;
use std::fs;

use nanocore::{assembler::Assembler, nanocore::NanoCore};

#[derive(Parser, Debug)]
#[command(name = "nanocore")]
#[command(about = "Run NanoCore", long_about = None)]
struct Args {
    /// Path to the source assembly file / compiled bin file
    #[arg(index = 1)]
    input: String,

    /// Print output
    #[arg(short, long, default_value_t = true)]
    print: bool,

    /// Print state
    #[arg(short = 's', long, default_value_t = false)]
    print_state: bool,

    /// Print instructions
    #[arg(short = 'i', long, default_value_t = false)]
    print_instructions: bool,
}

fn main() {
    let args = Args::parse();

    let bytes = if args.input.ends_with(".nca") {
        let asm = fs::read_to_string(&args.input).unwrap();

        if args.print_state {
            println!("Assembling {}", &args.input);
        }

        let mut c = Assembler::default();
        c.assemble(&asm);

        c.program
    } else {
        fs::read(args.input).unwrap()
    };

    let mut nano = NanoCore::new();
    nano.print = args.print;
    nano.print_state = args.print_state;
    nano.print_instructions = args.print_instructions;

    nano.load_program(&bytes, 0x00);
    nano.run();
}
