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

/// Errors that can occur during emulation.
#[derive(Debug, Clone, PartialEq)]
pub enum EmulatorError {
    /// Operand type mismatch for an instruction.
    ///
    /// Occurs when an instruction receives the wrong type of operand,
    /// such as expecting a register but receiving an immediate value.
    InvalidOperand {
        op: String,
        expected: String,
        got: String,
    },
    /// Stack overflow - attempting to push beyond stack limit.
    StackOverflow { sp: u8 },
    /// Stack underflow - attempting to pop from empty stack.
    StackUnderflow { sp: u8 },
    /// Program too large to fit in memory.
    ///
    /// The 8-bit architecture limits total addressable memory to 256 bytes.
    ProgramTooLarge { size: usize, start: u8, max: usize },
    /// Division or modulus by zero.
    DivisionByZero { op: String },
    /// I/O operation error.
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

/// Errors that can occur during assembly.
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

/// Result type for emulator operations.
pub type EmulatorResult<T> = std::result::Result<T, EmulatorError>;

/// Result type for assembler operations.
pub type AssemblerResult<T> = std::result::Result<T, AssemblerError>;
