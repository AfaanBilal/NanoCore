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

    c.assemble(&asm);

    print!("Assembled. Writing to bin.");

    fs::write(&args.output, &c.program)?;

    println!("\nDone.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::identity_op)]
    fn test_assemble() {
        let mut c = Assembler::default();
        c.assemble(
            "LDI R0 253
                 LDI R1 65
                 PRINT R1
                 ADD R2 R1
                 SUB R2 R0
                 INC R0
                 JZ 0x0F
                 INC R1
                 JMP 0x04
                 HLT",
        );

        assert_eq!(
            &c.program,
            &[
                0x10 | 0x00,
                0b1111_1101, // LDI R0 253
                0x10 | 0x01,
                0x41,        // LDI R1 65 ('A')
                0x50 | 0x01, // PRINT R1
                0x30,
                (0x02 << 4) | 0x01, // ADD R2 R1
                0x31,
                (0x02 << 4) | 0x00, // SUB R2 R0
                0x20 | 0x00,        // INC R0
                0x41,
                0x0F,        // JZ 0x0F (HLT)
                0x20 | 0x01, // INC R1
                0x40,
                0x04, // JMP 0x04 (-> PRINT R1)
                0x00, // HLT
            ]
        )
    }
}
