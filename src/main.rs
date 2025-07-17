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

use crate::nanocore::NanoCore;

pub mod cpu;
pub mod nanocore;

fn main() {
    #[allow(clippy::identity_op)]
    let program: &[u8] = &[
        0x10 | 0x00,
        0b1111_1101, // LDI R0 253
        0x10 | 0x01,
        0x41,        // LDI R1 65 ('A')
        0x50 | 0x01, // PRINT R1
        0x20 | 0x00, // INC R0
        0x41,
        0x0B,        // JZ 0x0B (HLT)
        0x20 | 0x01, // INC R1
        0x40,
        0x04, // JMP 0x04 (-> PRINT R1)
        0x00, // HLT
    ];

    println!("Program: ");
    for byte in program {
        println!("{byte:#04X} : {:04b} {:04b} ", byte >> 4, byte & 0x0F);
    }
    println!();

    let mut c = NanoCore::new();

    c.load_program(program, 0x00);
    c.run();
}
