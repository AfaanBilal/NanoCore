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

use crate::nanocore::Op;

#[derive(Default)]
pub struct Compiler {
    pub asm: String,
    pub compiled: Vec<u8>,
}

impl Compiler {
    pub fn compile(&mut self, asm: &str) {
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
                Op::HLT => self.compiled.push(0x00),
                Op::LDI => {
                    let register = Self::register(parts[1]);
                    self.compiled.push(0x10 | register);
                    self.compiled
                        .push(parts[2].parse::<u8>().expect("Invalid value"));
                }
                Op::INC => {
                    let register = Self::register(parts[1]);
                    self.compiled.push(0x20 | register);
                }
                Op::ADD | Op::SUB => {
                    let rd = Self::register(parts[1]);
                    let rs = Self::register(parts[2]);

                    match op {
                        Op::ADD => self.compiled.push(0x30),
                        Op::SUB => self.compiled.push(0x31),
                        _ => unreachable!(),
                    }

                    self.compiled.push((rd << 4) | rs);
                }
                Op::JMP | Op::JZ | Op::JNZ => {
                    let addr =
                        hex::decode(parts[1].strip_prefix("0x").expect("Invalid hex address"))
                            .expect("Expected address")[0];

                    match op {
                        Op::JMP => self.compiled.push(0x40),
                        Op::JZ => self.compiled.push(0x41),
                        Op::JNZ => self.compiled.push(0x42),
                        _ => unreachable!(),
                    }

                    self.compiled.push(addr);
                }
                Op::PRINT => {
                    let register = Self::register(parts[1]);
                    self.compiled.push(0x50 | register);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::identity_op)]
    fn test_compile() {
        let mut c = Compiler::default();
        c.compile(
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
            &c.compiled,
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
