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

use crate::cpu::CPU;

#[derive(Default)]
pub struct Computer {
    pub cpu: CPU,
}

impl Computer {
    #[must_use]
    pub fn new() -> Self {
        Computer { cpu: CPU::new() }
    }

    pub fn load_program(&mut self, program: &[u8], start_address: u8) {
        if (start_address as usize + program.len()) > 256 {
            panic!(
                "Error: Program ({} bytes) starting at {:#04X} exceeds 256-byte memory limit!",
                program.len(),
                start_address
            );
        }

        for (i, &byte) in program.iter().enumerate() {
            self.cpu.memory[start_address.wrapping_add(i as u8) as usize] = byte;
        }

        self.cpu.pc = start_address;

        println!("Program loaded. PC set to {:#04X}", self.cpu.pc);
    }
}

#[derive(Debug)]
pub enum Operand {
    None,
    Reg(u8),
    RegImm(u8, u8),
    RegReg(u8, u8),
    Addr(u8),
}

#[derive(Debug)]
pub enum Op {
    HLT,
    LDI,
    INC,
    ADD,
    JMP,
    NOP,
    PRINT,
}

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        match value & 0xF0 {
            0x00 => Op::HLT,
            0x10 => Op::LDI,
            0x20 => Op::INC,
            0x30 => Op::ADD,
            0x40 => Op::JMP,
            0x50 => Op::PRINT,
            _ => Op::NOP,
        }
    }
}
