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

use std::fs;

use crate::nanocore::NanoCore;

pub mod cpu;
pub mod nanocore;

fn main() {
    let bin = std::env::args().nth(1).expect("Missing filename.");
    let bytes = fs::read(bin).unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&bytes, 0x00);
    nano.run();
}
