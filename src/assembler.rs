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

use std::collections::HashMap;

use crate::Op;

#[derive(Default)]
pub struct Assembler {
    pub asm: String,
    pub labels: HashMap<String, u8>,
    pub program: Vec<u8>,
}

impl Assembler {
    pub fn assemble(&mut self, asm: &str) {
        self.asm = asm.to_owned();

        self.map_labels();

        let lines = self.asm.lines();

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with(";") {
                continue;
            }

            if Self::is_label(line) {
                continue;
            }

            let parts = line.split(" ").collect::<Vec<&str>>();
            let op: Op = parts[0].into();
            let opcode: u8 = op.into();

            match op {
                Op::HLT => self.program.push(opcode),
                Op::NOP => {}
                Op::LDI | Op::ADDI | Op::SUBI => {
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1]));
                    self.program.push(Self::from_value_str(parts[2]));
                }
                Op::LDA | Op::STO => {
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1]));
                    self.program.push(Self::from_hex_str(parts[2]));
                }
                Op::LDR | Op::MOV | Op::ADD | Op::SUB | Op::AND | Op::OR | Op::XOR | Op::CMP => {
                    self.program.push(opcode);
                    self.program
                        .push(Self::register(parts[1]) << 4 | Self::register(parts[2]));
                }
                Op::PUSH
                | Op::POP
                | Op::INC
                | Op::DEC
                | Op::NOT
                | Op::SHL
                | Op::SHR
                | Op::PRINT => {
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1]));
                }
                Op::JMP | Op::JZ | Op::JNZ => {
                    let addr = if self.labels.contains_key(parts[1]) {
                        *self.labels.get(parts[1]).unwrap()
                    } else {
                        Self::from_hex_str(parts[1])
                    };

                    self.program.push(opcode);
                    self.program.push(addr);
                }
            }
        }
    }

    pub fn map_labels(&mut self) {
        let lines = self.asm.lines();

        let mut addr: u8 = 0;

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with(";") {
                continue;
            }

            if Self::is_label(line) {
                self.labels
                    .insert(line.strip_suffix(':').unwrap().to_owned(), addr);
                continue;
            }

            let parts = line.split(" ").collect::<Vec<&str>>();

            let op: Op = parts[0].into();
            addr += op.instruction_len();
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

    pub fn from_value_str(v: &str) -> u8 {
        v.parse::<u8>().expect("Invalid value")
    }

    pub fn from_hex_str(v: &str) -> u8 {
        hex::decode(v.strip_prefix("0x").expect("Invalid hex address")).expect("Expected address")
            [0]
    }

    pub fn is_label(l: &str) -> bool {
        l.ends_with(':')
    }

    pub fn print_program(p: &[u8]) {
        for byte in p {
            println!("{byte:#04X} : {:04b} {:04b} ", byte >> 4, byte & 0x0F);
        }
    }
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
