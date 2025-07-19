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

use crate::{assembler::Assembler, nanocore::NanoCore};

pub mod assembler;
pub mod cpu;
pub mod nanocore;

fn main() {
    #[allow(clippy::identity_op)]
    let program: &[u8] = &[
        0x10 | 0x00,
        253, // LDI R0 253
        0x10 | 0x01,
        0x41,        // LDI R1 65 ('A')
        0x50 | 0x01, // PRINT R1
        0x30,
        (0x02 << 4) | 0x01, // ADD R2 R1
        0x31,
        (0x02 << 4) | 0x00, // SUB R2 R0
        0x20 | 0x00,        // INC R0
        0x41,
        0x11,        // JZ 0x11 (HLT)
        0x20 | 0x01, // INC R1
        0x60 | 0x01, // SHL R1
        0x70 | 0x01, // SHR R1
        0x40,
        0x04, // JMP 0x04 (-> PRINT R1)
        0x00, // HLT
    ];

    println!("Program: ");
    Assembler::print_program(program);
    println!();

    let mut nano = NanoCore::new();
    nano.load_program(program, 0x00);
    nano.run();

    let mut c = Assembler::default();
    c.assemble(
        "
    LDI R0 253
    LDI R1 65
    PRINT R1
    ADD R2 R1
    SUB R2 R0
    INC R0
    JZ 0x11
    INC R1
    SHL R1
    SHR R1
    JMP 0x04
    HLT
    ",
    );
    println!("Binary: ");
    Assembler::print_program(&c.program);
    println!();

    let mut nano = NanoCore::new();
    nano.load_program(&c.program, 0x00);
    nano.run();
}
