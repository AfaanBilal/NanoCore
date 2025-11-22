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

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum EmulatorError {
    InvalidOperand {
        op: String,
        expected: String,
        got: String,
    },
    StackOverflow {
        sp: u8,
    },
    StackUnderflow {
        sp: u8,
    },
    ProgramTooLarge {
        size: usize,
        start: u8,
        max: usize,
    },
    DivisionByZero {
        op: String,
    },
    IoError(String),
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOperand { op, expected, got } => {
                write!(f, "{} expects {} operand, got {}", op, expected, got)
            }
            Self::StackOverflow { sp } => {
                write!(f, "Stack overflow at SP={:#04X}", sp)
            }
            Self::StackUnderflow { sp } => {
                write!(f, "Stack underflow at SP={:#04X}", sp)
            }
            Self::ProgramTooLarge { size, start, max } => {
                write!(
                    f,
                    "Program too large: {} bytes starting at {:#04X} exceeds {}-byte memory limit",
                    size, start, max
                )
            }
            Self::DivisionByZero { op } => {
                write!(f, "{}: division by zero", op)
            }
            Self::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EmulatorError {}

#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerError {
    SyntaxError { line: usize, message: String },
    InvalidRegister { name: String, line: usize },
    InvalidValue { value: String, line: usize },
    InvalidHexAddress { value: String, line: usize },
    UndefinedLabel { label: String, line: usize },
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SyntaxError { line, message } => {
                write!(f, "Line {}: {}", line, message)
            }
            Self::InvalidRegister { name, line } => {
                write!(f, "Line {}: Invalid register '{}'", line, name)
            }
            Self::InvalidValue { value, line } => {
                write!(f, "Line {}: Invalid value '{}'", line, value)
            }
            Self::InvalidHexAddress { value, line } => {
                write!(f, "Line {}: Invalid hex address '{}'", line, value)
            }
            Self::UndefinedLabel { label, line } => {
                write!(f, "Line {}: Undefined label '{}'", line, label)
            }
        }
    }
}

impl std::error::Error for AssemblerError {}

pub type EmulatorResult<T> = std::result::Result<T, EmulatorError>;

pub type AssemblerResult<T> = std::result::Result<T, AssemblerError>;
