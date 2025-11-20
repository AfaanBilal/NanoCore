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
    pub constants: HashMap<String, u8>,
    pub program: Vec<u8>,
}

impl Assembler {
    pub fn assemble(&mut self, asm: &str) {
        self.asm = asm.to_owned();

        self.map_constants();
        self.map_labels();

        let lines = self.asm.lines();

        for line in lines {
            let line = line.trim();
            if line.is_empty() || Self::is_comment(line) {
                continue;
            }

            if Self::is_label(line) || Self::is_constant(line) {
                continue;
            }

            let parts = line.split(" ").collect::<Vec<&str>>();
            let op: Op = parts[0].into();
            let opcode: u8 = op.into();

            match op {
                Op::HLT | Op::RET => self.program.push(opcode),
                Op::NOP => {}
                Op::LDI | Op::ADDI | Op::SUBI | Op::MULI | Op::DIVI | Op::MODI => {
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1]));
                    self.program.push(self.resolve_number(parts[2]));
                }
                Op::LDA | Op::STORE => {
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1]));
                    self.program.push(self.resolve_number(parts[2]));
                }
                Op::LDR
                | Op::MOV
                | Op::ADD
                | Op::SUB
                | Op::AND
                | Op::OR
                | Op::XOR
                | Op::CMP
                | Op::MUL
                | Op::DIV
                | Op::MOD => {
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
                | Op::ROL
                | Op::ROR
                | Op::IN
                | Op::JMPR
                | Op::CALLR
                | Op::PRINT => {
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1]));
                }
                Op::JMP | Op::JZ | Op::JNZ | Op::CALL => {
                    let addr = if self.labels.contains_key(parts[1]) {
                        *self.labels.get(parts[1]).unwrap()
                    } else {
                        self.resolve_number(parts[1])
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
            if line.is_empty() || Self::is_comment(line) || Self::is_constant(line) {
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

    pub fn map_constants(&mut self) {
        let lines = self.asm.lines();

        for line in lines {
            let line = line.trim();
            if !Self::is_constant(line) {
                continue;
            }

            let parts = line.split(" ").collect::<Vec<&str>>();
            let name = parts[1];
            let value = if parts[2].starts_with("0x") {
                Self::from_hex_str(parts[2])
            } else {
                Self::from_value_str(parts[2])
            };

            self.constants.insert(name.to_owned(), value);
        }
    }

    pub fn resolve_number(&self, v: &str) -> u8 {
        if let Some(value) = self.constants.get(v) {
            return *value;
        }

        if v.starts_with("0x") {
            Self::from_hex_str(v)
        } else {
            Self::from_value_str(v)
        }
    }

    pub fn register(r: &str) -> u8 {
        let register = r
            .strip_prefix("R")
            .expect("Expected register R0..R15")
            .parse::<u8>()
            .expect("Expected register R0..R15");

        if register > 15 {
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

    pub fn is_constant(l: &str) -> bool {
        l.starts_with(".CONST")
    }

    pub fn is_comment(l: &str) -> bool {
        l.starts_with(";")
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
                 JZ 0x18
                 INC R1
                 SHL R1
                 SHR R1
                 JMP 0x06
                 HLT",
        );

        assert_eq!(
            &c.program,
            &[
                0x02, 0x00, 0xFD, 0x02, 0x01, 0x41, 0x19, 0x01, 0x09, 0x21, 0x0B, 0x20, 0x0D, 0x00,
                0x17, 0x18, 0x0D, 0x01, 0x14, 0x01, 0x15, 0x01, 0x16, 0x06, 0x00,
            ]
        )
    }

    #[test]
    fn test_assemble_rotate() {
        let mut c = Assembler::default();
        c.assemble(
            "LDI R0 1
             ROL R0
             ROR R0
             HLT",
        );

        assert_eq!(
            &c.program,
            &[
                Op::LDI.into(),
                0,
                1,
                Op::ROL.into(),
                0,
                Op::ROR.into(),
                0,
                Op::HLT.into()
            ]
        )
    }

    #[test]
    fn test_assemble_stack() {
        let mut c = Assembler::default();
        c.assemble(
            "PUSH R0
             POP R1",
        );

        assert_eq!(&c.program, &[Op::PUSH.into(), 0, Op::POP.into(), 1,])
    }

    #[test]
    fn test_assemble_arithmetic() {
        let mut c = Assembler::default();
        c.assemble(
            "ADD R0 R1
             SUB R0 R1
             MUL R0 R1
             DIV R0 R1
             MOD R0 R1
             ADDI R0 1
             SUBI R0 1
             MULI R0 1
             DIVI R0 1
             MODI R0 1",
        );

        assert_eq!(
            &c.program,
            &[
                Op::ADD.into(),
                0x01,
                Op::SUB.into(),
                0x01,
                Op::MUL.into(),
                0x01,
                Op::DIV.into(),
                0x01,
                Op::MOD.into(),
                0x01,
                Op::ADDI.into(),
                0,
                1,
                Op::SUBI.into(),
                0,
                1,
                Op::MULI.into(),
                0,
                1,
                Op::DIVI.into(),
                0,
                1,
                Op::MODI.into(),
                0,
                1,
            ]
        )
    }

    #[test]
    fn test_assemble_control_flow() {
        let mut c = Assembler::default();
        c.assemble(
            "JMP 0x10
             JZ 0x10
             JNZ 0x10
             CALL 0x10
             RET",
        );

        assert_eq!(
            &c.program,
            &[
                Op::JMP.into(),
                0x10,
                Op::JZ.into(),
                0x10,
                Op::JNZ.into(),
                0x10,
                Op::CALL.into(),
                0x10,
                Op::RET.into(),
            ]
        )
    }

    #[test]
    fn test_assemble_logic() {
        let mut c = Assembler::default();
        c.assemble(
            "AND R0 R1
             OR R0 R1
             XOR R0 R1
             NOT R0
             CMP R0 R1",
        );

        assert_eq!(
            &c.program,
            &[
                Op::AND.into(),
                0x01,
                Op::OR.into(),
                0x01,
                Op::XOR.into(),
                0x01,
                Op::NOT.into(),
                0,
                Op::CMP.into(),
                0x01,
            ]
        )
    }

    #[test]
    fn test_assemble_constants() {
        let mut c = Assembler::default();
        c.assemble(
            ".CONST VAL 10
             .CONST ADDR 0x10
             LDI R0 VAL
             JMP ADDR",
        );

        assert_eq!(&c.program, &[Op::LDI.into(), 0, 10, Op::JMP.into(), 0x10,])
    }

    #[test]
    fn test_assemble_input() {
        let mut c = Assembler::default();
        c.assemble(
            "IN R0
             PRINT R0",
        );

        assert_eq!(&c.program, &[Op::IN.into(), 0, Op::PRINT.into(), 0,])
    }

    #[test]
    fn test_assemble_indirect_jumps() {
        let mut c = Assembler::default();
        c.assemble(
            "JMPR R0
             CALLR R1",
        );

        assert_eq!(&c.program, &[Op::JMPR.into(), 0, Op::CALLR.into(), 1,])
    }
}
