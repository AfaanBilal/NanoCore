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
use nanocore::{Op, end_color, start_color};

#[derive(Default)]
pub struct Assembler {
    pub asm: String,
    pub program: Vec<u8>,
}

impl Assembler {
    pub fn assemble(&mut self, asm: &str) {
        self.asm = asm.to_owned();

        let lines = self.asm.lines();

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with(";") {
                continue;
            }

            let parts = line.split(" ").collect::<Vec<&str>>();
            let op: Op = parts[0].into();

            match op {
                Op::HLT => self.program.push(0x00),
                Op::LDI => {
                    let register = Self::register(parts[1]);
                    self.program.push(0x10 | register);
                    self.program
                        .push(parts[2].parse::<u8>().expect("Invalid value"));
                }
                Op::INC => {
                    let register = Self::register(parts[1]);
                    self.program.push(0x20 | register);
                }
                Op::ADD | Op::SUB => {
                    let rd = Self::register(parts[1]);
                    let rs = Self::register(parts[2]);

                    match op {
                        Op::ADD => self.program.push(0x30),
                        Op::SUB => self.program.push(0x31),
                        _ => unreachable!(),
                    }

                    self.program.push((rd << 4) | rs);
                }
                Op::JMP | Op::JZ | Op::JNZ => {
                    let addr =
                        hex::decode(parts[1].strip_prefix("0x").expect("Invalid hex address"))
                            .expect("Expected address")[0];

                    match op {
                        Op::JMP => self.program.push(0x40),
                        Op::JZ => self.program.push(0x41),
                        Op::JNZ => self.program.push(0x42),
                        _ => unreachable!(),
                    }

                    self.program.push(addr);
                }
                Op::PRINT => {
                    let register = Self::register(parts[1]);
                    self.program.push(0x50 | register);
                }
                Op::SHL | Op::SHR => {
                    let register = Self::register(parts[1]);

                    let opcode = match op {
                        Op::SHL => 0x60,
                        Op::SHR => 0x70,
                        _ => unreachable!(),
                    };

                    self.program.push(opcode | register);
                }
                Op::NOP => {}
            }
        }
    }

    pub fn register(r: &str) -> u8 {
        let register = r
            .strip_prefix("R")
            .expect("Expected register R0..R7")
            .parse::<u8>()
            .expect("Expected register R0..R7");

        if register > 7 {
            panic!("Invalid register {r}");
        }

        register
    }

    pub fn print_program(p: &[u8]) {
        for byte in p {
            println!("{byte:#04X} : {:04b} {:04b} ", byte >> 4, byte & 0x0F);
        }
    }
}

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

    fs::write(&args.output, c.program)?;

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
