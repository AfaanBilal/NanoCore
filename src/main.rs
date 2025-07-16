#![deny(clippy::all)]
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

use crate::computer::Computer;

pub mod computer;
pub mod cpu;

fn main() {
    let program: [u8; 5] = [
        0x10 | 0x01,
        0x41,        // LDI R1 65 'A'
        0x20 | 0x01, // INC R2
        0x50 | 0x01, // PRINT R1
        0x00,        // HLT
    ];

    let mut c = Computer::new();

    c.load_program(&program, 0x00);
    c.run();
}
