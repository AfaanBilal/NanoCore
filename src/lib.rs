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

pub mod assembler;
pub mod cpu;
pub mod nanocore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    HLT, // Halt: HLT

    NOP, // No-operation

    LDI, // Load immediate: LDI Rx 123
    LDA, // Load from address: LDA Rx 0xF1
    LDR, // Load from address in register: LDR Rx Ry (Ry contains memory address)

    MOV, // Move data between registers: MOV Rx Ry

    STORE, // Store to address: ST Rx 0xF1

    PUSH, // Push register to stack: PUSH Rx
    POP,  // Pop register from stack: POP Rx

    ADD,  // Add: ADD Rx Ry (Rx = Rx + Ry)
    ADDI, // Add using immediate: ADD Rx 123 (Rx = Rx + 123)
    SUB,  // Subtract: SUB Rx Ry (Rx = Rx - Ry)
    SUBI, // Subtract using immediate: SUB Rx 123 (Rx = Rx - 123)

    INC, // Increment: INC Rx (Rx = Rx + 1)
    DEC, // Decrement: DEC Rx (Rx = Rx - 1)

    AND, // Logical AND: AND Rx Ry (Rx = Rx & Ry)
    OR,  // Logical OR: OR Rx Ry (Rx = Rx | Ry)
    XOR, // Logical XOR: XOR Rx Ry (Rx = Rx ^ Ry)
    NOT, // Logical NOT: NOT Rx (Rx = !Rx)
    CMP, // Comparison: CMP Rx Ry (Set zero flag if Rx = Ry, otherwise reset)

    JMP, // Unconditional Jump to address: JMP 0xAB
    JZ,  // Jump if zero flag set: JZ 0xAB
    JNZ, // Jump if zero flag not set: JNZ 0xAB

    SHL, // Shift left: SHL Rx (Rx = Rx << 1)
    SHR, // Shift right: SHR Rx (Rx = Rx >> 1)

    PRINT, // Print to output: PRINT Rx

    MUL,  // Multiply: MUL Rx Ry (Rx = Rx * Ry)
    MULI, // Multiply using immediate: MUL Rx 123 (Rx = Rx * 123)
    DIV,  // Divide: DIV Rx Ry (Rx = Rx / Ry)
    DIVI, // Divide using immediate: DIV Rx 123 (Rx = Rx / 123)
    MOD,  // Modulus: MOD Rx Ry (Rx = Rx % Ry)
    MODI, // Modulus using immediate: MOD Rx 123 (Rx = Rx % 123)

    CALL, // CALL a function: CALL raise_to_power
    RET,  // Return from a function
}

impl Op {
    pub fn instruction_len(&self) -> u8 {
        match self {
            Op::LDI
            | Op::LDA
            | Op::STORE
            | Op::ADDI
            | Op::SUBI
            | Op::MULI
            | Op::DIVI
            | Op::MODI => 3,
            Op::LDR | Op::MOV | Op::PUSH | Op::POP | Op::ADD | Op::SUB | Op::INC | Op::DEC => 2,
            Op::AND | Op::OR | Op::XOR | Op::NOT | Op::CMP | Op::SHL | Op::SHR => 2,
            Op::JMP | Op::JZ | Op::JNZ | Op::PRINT => 2,
            Op::MUL | Op::DIV | Op::MOD | Op::CALL => 2,
            _ => 1,
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = (*self).into();
        write!(f, "{s}")
    }
}

impl From<Op> for String {
    fn from(value: Op) -> Self {
        let op_str: &str = value.into();

        op_str.to_owned()
    }
}

impl From<Op> for &str {
    fn from(value: Op) -> Self {
        match value {
            Op::HLT => "HLT",
            Op::NOP => "NOP",

            Op::LDI => "LDI",
            Op::LDA => "LDA",
            Op::LDR => "LDR",

            Op::MOV => "MOV",
            Op::STORE => "STORE",

            Op::PUSH => "PUSH",
            Op::POP => "POP",

            Op::ADD => "ADD",
            Op::ADDI => "ADDI",
            Op::SUB => "SUB",
            Op::SUBI => "SUBI",

            Op::INC => "INC",
            Op::DEC => "DEC",

            Op::AND => "AND",
            Op::OR => "OR",
            Op::XOR => "XOR",
            Op::NOT => "NOT",
            Op::CMP => "CMP",

            Op::JMP => "JMP",
            Op::JZ => "JZ",
            Op::JNZ => "JNZ",

            Op::SHL => "SHL",
            Op::SHR => "SHR",

            Op::PRINT => "PRINT",

            Op::MUL => "MUL",
            Op::MULI => "MULI",
            Op::DIV => "DIV",
            Op::DIVI => "DIVI",
            Op::MOD => "MOD",
            Op::MODI => "MODI",

            Op::CALL => "CALL",
            Op::RET => "RET",
        }
    }
}

impl From<&str> for Op {
    fn from(value: &str) -> Self {
        match value {
            "HLT" => Op::HLT,
            "NOP" => Op::NOP,

            "LDI" => Op::LDI,
            "LDA" => Op::LDA,
            "LDR" => Op::LDR,

            "MOV" => Op::MOV,
            "STORE" => Op::STORE,

            "PUSH" => Op::PUSH,
            "POP" => Op::POP,

            "ADD" => Op::ADD,
            "ADDI" => Op::ADDI,
            "SUB" => Op::SUB,
            "SUBI" => Op::SUBI,

            "INC" => Op::INC,
            "DEC" => Op::DEC,

            "AND" => Op::AND,
            "OR" => Op::OR,
            "XOR" => Op::XOR,
            "NOT" => Op::NOT,
            "CMP" => Op::CMP,

            "JMP" => Op::JMP,
            "JZ" => Op::JZ,
            "JNZ" => Op::JNZ,

            "SHL" => Op::SHL,
            "SHR" => Op::SHR,

            "PRINT" => Op::PRINT,

            "MUL" => Op::MUL,
            "MULI" => Op::MULI,
            "DIV" => Op::DIV,
            "DIVI" => Op::DIVI,
            "MOD" => Op::MOD,
            "MODI" => Op::MODI,

            "CALL" => Op::CALL,
            "RET" => Op::RET,

            _ => panic!("Invalid operation: {value}"),
        }
    }
}

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Op::HLT,
            0x01 => Op::NOP,

            0x02 => Op::LDI,
            0x03 => Op::LDA,
            0x04 => Op::LDR,

            0x05 => Op::MOV,
            0x06 => Op::STORE,

            0x07 => Op::PUSH,
            0x08 => Op::POP,

            0x09 => Op::ADD,
            0x0A => Op::ADDI,
            0x0B => Op::SUB,
            0x0C => Op::SUBI,

            0x0D => Op::INC,
            0x0E => Op::DEC,

            0x0F => Op::AND,
            0x10 => Op::OR,
            0x11 => Op::XOR,
            0x12 => Op::NOT,
            0x13 => Op::CMP,

            0x14 => Op::SHL,
            0x15 => Op::SHR,

            0x16 => Op::JMP,
            0x17 => Op::JZ,
            0x18 => Op::JNZ,

            0x19 => Op::PRINT,

            0x1A => Op::MUL,
            0x1B => Op::MULI,
            0x1C => Op::DIV,
            0x1D => Op::DIVI,
            0x1E => Op::MOD,
            0x1F => Op::MODI,

            0x20 => Op::CALL,
            0x21 => Op::RET,

            _ => Op::NOP,
        }
    }
}

impl From<Op> for u8 {
    fn from(val: Op) -> Self {
        match val {
            Op::HLT => 0x00,
            Op::NOP => 0x01,

            Op::LDI => 0x02,
            Op::LDA => 0x03,
            Op::LDR => 0x04,

            Op::MOV => 0x05,
            Op::STORE => 0x06,

            Op::PUSH => 0x07,
            Op::POP => 0x08,

            Op::ADD => 0x09,
            Op::ADDI => 0x0A,
            Op::SUB => 0x0B,
            Op::SUBI => 0x0C,

            Op::INC => 0x0D,
            Op::DEC => 0x0E,

            Op::AND => 0x0F,
            Op::OR => 0x10,
            Op::XOR => 0x11,
            Op::NOT => 0x12,
            Op::CMP => 0x13,

            Op::SHL => 0x14,
            Op::SHR => 0x15,

            Op::JMP => 0x16,
            Op::JZ => 0x17,
            Op::JNZ => 0x18,

            Op::PRINT => 0x19,

            Op::MUL => 0x1A,
            Op::MULI => 0x1B,
            Op::DIV => 0x1C,
            Op::DIVI => 0x1D,
            Op::MOD => 0x1E,
            Op::MODI => 0x1F,

            Op::CALL => 0x20,
            Op::RET => 0x21,
        }
    }
}

pub fn start_color() {
    print!("\x1b[92;40m");
}

pub fn end_color() {
    print!("\x1b[0m");
}
