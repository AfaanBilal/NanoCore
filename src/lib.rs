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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    HLT,
    LDI,
    INC,
    ADD,
    SUB,
    JMP,
    JZ,
    JNZ,
    PRINT,
    SHL,
    SHR,
    NOP,
}

impl Op {
    pub fn instruction_len(&self) -> u8 {
        match self {
            Op::LDI | Op::ADD | Op::SUB | Op::JMP | Op::JZ | Op::JNZ => 2,
            _ => 1,
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::HLT => "HLT",
                Op::LDI => "LDI",
                Op::INC => "INC",
                Op::ADD => "ADD",
                Op::SUB => "SUB",
                Op::JMP => "JMP",
                Op::JZ => "JZ",
                Op::JNZ => "JNZ",
                Op::PRINT => "PRINT",
                Op::SHL => "SHL",
                Op::SHR => "SHR",
                Op::NOP => "NOP",
            }
        )
    }
}

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        let high = value & 0xF0;

        match value {
            0x00 => Op::HLT,
            _ if high == 0x10 => Op::LDI,
            _ if high == 0x20 => Op::INC,
            0x30 => Op::ADD,
            0x31 => Op::SUB,
            0x40 => Op::JMP,
            0x41 => Op::JZ,
            0x42 => Op::JNZ,
            _ if high == 0x50 => Op::PRINT,
            _ if high == 0x60 => Op::SHL,
            _ if high == 0x70 => Op::SHR,
            _ => Op::NOP,
        }
    }
}

impl From<&str> for Op {
    fn from(value: &str) -> Self {
        match value {
            "HLT" => Op::HLT,
            "LDI" => Op::LDI,
            "INC" => Op::INC,
            "ADD" => Op::ADD,
            "SUB" => Op::SUB,
            "JMP" => Op::JMP,
            "JZ" => Op::JZ,
            "JNZ" => Op::JNZ,
            "SHL" => Op::SHL,
            "SHR" => Op::SHR,
            "PRINT" => Op::PRINT,
            "NOP" => Op::NOP,
            _ => panic!("Invalid operation: {value}"),
        }
    }
}
