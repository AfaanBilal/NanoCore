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
    pub fn assemble(&mut self, asm: &str) -> crate::AssemblerResult<()> {
        self.asm = asm.to_owned();

        self.map_constants()?;
        self.map_labels()?;

        let lines = self.asm.lines();

        for (line_idx, line) in lines.enumerate() {
            let line_num = line_idx + 1;
            let line = line.trim();
            // Strip comment
            let line = if let Some(idx) = line.find(';') {
                line[..idx].trim()
            } else {
                line
            };

            if line.is_empty() {
                continue;
            }

            if Self::is_label(line) || Self::is_constant(line) {
                continue;
            }

            if line.starts_with(".DB") {
                let parts = line.split_whitespace().collect::<Vec<&str>>();
                for part in parts.iter().skip(1) {
                    self.program.push(self.resolve_number(part, line_num)?);
                }
                continue;
            }

            if line.starts_with(".STRING") {
                let start = line.find('"').ok_or(crate::AssemblerError::SyntaxError {
                    line: line_num,
                    message: "Expected start of string".to_string(),
                })? + 1;
                let end = line.rfind('"').ok_or(crate::AssemblerError::SyntaxError {
                    line: line_num,
                    message: "Expected end of string".to_string(),
                })?;
                let content = &line[start..end];
                for byte in content.bytes() {
                    self.program.push(byte);
                }
                continue;
            }

            let parts = line.split_whitespace().collect::<Vec<&str>>();
            if parts.is_empty() {
                continue;
            }

            let op: Op = match Op::try_from(parts[0]) {
                Ok(op) => op,
                Err(crate::AssemblerError::SyntaxError { message, .. }) => {
                    return Err(crate::AssemblerError::SyntaxError {
                        line: line_num,
                        message,
                    });
                }
                Err(e) => return Err(e),
            };

            let opcode: u8 = op.into();

            match op {
                Op::HLT | Op::RET => self.program.push(opcode),
                Op::NOP => {}
                Op::LDI | Op::ADDI | Op::SUBI | Op::MULI | Op::DIVI | Op::MODI => {
                    if parts.len() < 3 {
                        return Err(crate::AssemblerError::SyntaxError {
                            line: line_num,
                            message: format!("{} requires 2 arguments", op),
                        });
                    }
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1], line_num)?);
                    self.program.push(self.resolve_number(parts[2], line_num)?);
                }
                Op::LDA | Op::STORE => {
                    if parts.len() < 3 {
                        return Err(crate::AssemblerError::SyntaxError {
                            line: line_num,
                            message: format!("{} requires 2 arguments", op),
                        });
                    }
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1], line_num)?);
                    self.program.push(self.resolve_number(parts[2], line_num)?);
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
                | Op::MOD
                | Op::STR => {
                    if parts.len() < 3 {
                        return Err(crate::AssemblerError::SyntaxError {
                            line: line_num,
                            message: format!("{} requires 2 arguments", op),
                        });
                    }
                    self.program.push(opcode);
                    self.program.push(
                        Self::register(parts[1], line_num)? << 4
                            | Self::register(parts[2], line_num)?,
                    );
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
                    if parts.len() < 2 {
                        return Err(crate::AssemblerError::SyntaxError {
                            line: line_num,
                            message: format!("{} requires 1 argument", op),
                        });
                    }
                    self.program.push(opcode);
                    self.program.push(Self::register(parts[1], line_num)?);
                }
                Op::JMP | Op::JZ | Op::JNZ | Op::CALL => {
                    if parts.len() < 2 {
                        return Err(crate::AssemblerError::SyntaxError {
                            line: line_num,
                            message: format!("{} requires 1 argument", op),
                        });
                    }
                    let addr = if self.labels.contains_key(parts[1]) {
                        *self.labels.get(parts[1]).unwrap()
                    } else {
                        self.resolve_number(parts[1], line_num)?
                    };

                    self.program.push(opcode);
                    self.program.push(addr);
                }
            }
        }
        Ok(())
    }

    pub fn map_labels(&mut self) -> crate::AssemblerResult<()> {
        let lines = self.asm.lines();

        let mut addr: u8 = 0;

        for (line_idx, line) in lines.enumerate() {
            let line_num = line_idx + 1;
            let line = line.trim();

            // Strip comment
            let line = if let Some(idx) = line.find(';') {
                line[..idx].trim()
            } else {
                line
            };

            if line.is_empty() || Self::is_constant(line) {
                continue;
            }

            if line.starts_with(".DB") {
                let parts = line.split_whitespace().collect::<Vec<&str>>();
                addr = addr.wrapping_add((parts.len() - 1) as u8);
                continue;
            }

            if line.starts_with(".STRING") {
                let start = line.find('"').ok_or(crate::AssemblerError::SyntaxError {
                    line: line_num,
                    message: "Expected start of string".to_string(),
                })? + 1;
                let end = line.rfind('"').ok_or(crate::AssemblerError::SyntaxError {
                    line: line_num,
                    message: "Expected end of string".to_string(),
                })?;
                addr = addr.wrapping_add((end - start) as u8);
                continue;
            }

            if Self::is_label(line) {
                self.labels
                    .insert(line.trim_end_matches(':').to_owned(), addr);
                continue;
            }

            let parts = line.split_whitespace().collect::<Vec<&str>>();
            if parts.is_empty() {
                continue;
            }

            let op: Op = match Op::try_from(parts[0]) {
                Ok(op) => op,
                Err(crate::AssemblerError::SyntaxError { message, .. }) => {
                    return Err(crate::AssemblerError::SyntaxError {
                        line: line_num,
                        message,
                    });
                }
                Err(e) => return Err(e),
            };

            addr = addr.wrapping_add(op.instruction_len());
        }
        Ok(())
    }

    pub fn map_constants(&mut self) -> crate::AssemblerResult<()> {
        let lines = self.asm.lines();

        for (line_idx, line) in lines.enumerate() {
            let line_num = line_idx + 1;
            let line = line.trim();

            // Strip comment
            let line = if let Some(idx) = line.find(';') {
                line[..idx].trim()
            } else {
                line
            };

            if !Self::is_constant(line) {
                continue;
            }

            let parts = line.split_whitespace().collect::<Vec<&str>>();
            if parts.len() < 3 {
                return Err(crate::AssemblerError::SyntaxError {
                    line: line_num,
                    message: "Invalid constant definition".to_string(),
                });
            }

            let name = parts[1];
            let value = if parts[2].starts_with("0x") {
                Self::from_hex_str(parts[2], line_num)?
            } else {
                Self::from_value_str(parts[2], line_num)?
            };

            self.constants.insert(name.to_owned(), value);
        }
        Ok(())
    }

    pub fn resolve_number(&self, v: &str, line: usize) -> crate::AssemblerResult<u8> {
        if let Some(value) = self.constants.get(v) {
            return Ok(*value);
        }

        if v.starts_with("0x") {
            Self::from_hex_str(v, line)
        } else {
            Self::from_value_str(v, line)
        }
    }

    pub fn register(r: &str, line: usize) -> crate::AssemblerResult<u8> {
        let register = r
            .strip_prefix("R")
            .ok_or(crate::AssemblerError::InvalidRegister {
                line,
                name: r.to_string(),
            })?
            .parse::<u8>()
            .map_err(|_| crate::AssemblerError::InvalidRegister {
                line,
                name: r.to_string(),
            })?;

        if register > 15 {
            return Err(crate::AssemblerError::InvalidRegister {
                line,
                name: r.to_string(),
            });
        }

        Ok(register)
    }

    pub fn from_value_str(v: &str, line: usize) -> crate::AssemblerResult<u8> {
        v.parse::<u8>()
            .map_err(|_| crate::AssemblerError::InvalidValue {
                line,
                value: v.to_string(),
            })
    }

    pub fn from_hex_str(v: &str, line: usize) -> crate::AssemblerResult<u8> {
        let bytes = hex::decode(v.strip_prefix("0x").unwrap_or(v)).map_err(|_| {
            crate::AssemblerError::InvalidHexAddress {
                line,
                value: v.to_string(),
            }
        })?;

        if bytes.is_empty() {
            return Err(crate::AssemblerError::InvalidValue {
                line,
                value: v.to_string(),
            });
        }

        Ok(bytes[0])
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
        let _ = c.assemble(
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
        )
        .unwrap();

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
        )
        .unwrap();

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
        )
        .unwrap();

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
        )
        .unwrap();
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
        )
        .unwrap();

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
        )
        .unwrap();

        assert_eq!(&c.program, &[Op::LDI.into(), 0, 10, Op::JMP.into(), 0x10,])
    }

    #[test]
    fn test_assemble_input() {
        let mut c = Assembler::default();
        c.assemble(
            "IN R0
             PRINT R0",
        )
        .unwrap();

        assert_eq!(&c.program, &[Op::IN.into(), 0, Op::PRINT.into(), 0,])
    }

    #[test]
    fn test_assemble_indirect_jumps() {
        let mut c = Assembler::default();
        c.assemble(
            "JMPR R0
             CALLR R1",
        )
        .unwrap();

        assert_eq!(&c.program, &[Op::JMPR.into(), 0, Op::CALLR.into(), 1,])
    }

    #[test]
    fn test_assemble_data_directives() {
        let mut c = Assembler::default();
        c.assemble(
            ".DB 0x01 0x02 10
             .STRING \"ABC\"
             LDI R0 0xFF",
        )
        .unwrap();

        assert_eq!(
            &c.program,
            &[
                0x01,
                0x02,
                10, // .DB
                0x41,
                0x42,
                0x43, // .STRING "ABC"
                Op::LDI.into(),
                0,
                0xFF // LDI R0 0xFF
            ]
        )
    }

    #[test]
    fn test_assemble_str() {
        let mut c = Assembler::default();
        c.assemble("STR R0 R1").unwrap();

        assert_eq!(&c.program, &[Op::STR.into(), 0x01])
    }
}
