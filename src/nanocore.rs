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

use std::fmt::Display;

use crate::cpu::CPU;

#[derive(Default)]
pub struct NanoCore {
    pub cpu: CPU,
}

impl NanoCore {
    #[must_use]
    pub fn new() -> Self {
        NanoCore { cpu: CPU::new() }
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

        println!("Program loaded.");
    }

    pub fn run(&mut self) {
        println!("\n === NanoCore Start === \n");

        let mut cycle = 0;

        while !self.cpu.is_halted {
            cycle += 1;
            print!("\nCycle: {cycle:03} | ");
            self.cpu.print_state();

            if cycle >= 20 {
                break;
            }

            self.cycle();
        }

        println!("\n === NanoCore Halt === \n");
    }

    pub fn cycle(&mut self) {
        let (op, operands, instruction_len) = self.fetch_decode();

        let pc_override = self.execute(op, operands);

        if self.cpu.is_halted {
            return;
        }

        if !pc_override {
            self.cpu.pc = self.cpu.pc.wrapping_add(instruction_len);
        }
    }

    pub fn fetch_decode(&self) -> (Op, Operands, u8) {
        // FETCH
        let opcode = self.cpu.memory[self.cpu.pc as usize];
        let mut instruction_len = 1;

        // DECODE
        let op: Op = opcode.into();

        let operands = match op {
            Op::HLT | Op::NOP => Operands::None,
            Op::LDI => {
                instruction_len = 2;
                let reg = opcode & 0b0000_0111;
                let value = self.cpu.memory[self.cpu.pc.wrapping_add(1) as usize];

                Operands::RegImm(reg, value)
            }
            Op::INC => {
                let reg = opcode & 0b0000_0111;

                Operands::Reg(reg)
            }
            Op::ADD | Op::SUB => {
                instruction_len = 2;
                let value = self.cpu.memory[self.cpu.pc.wrapping_add(1) as usize];
                let rd = (value >> 4) & 0b0000_0111;
                let rs = value & 0b0000_0111;

                Operands::RegReg(rd, rs)
            }
            Op::JMP | Op::JZ | Op::JNZ => {
                instruction_len = 2;
                let addr = self.cpu.memory[self.cpu.pc.wrapping_add(1) as usize];

                Operands::Addr(addr)
            }
            Op::PRINT => {
                let reg = opcode & 0b0000_0111;

                Operands::Reg(reg)
            }
        };

        (op, operands, instruction_len)
    }

    pub fn execute(&mut self, op: Op, operands: Operands) -> bool {
        let mut pc_override = false;

        match op {
            Op::HLT => {
                self.cpu.is_halted = true;

                println!("-> HLT");
            }
            Op::LDI => {
                let Operands::RegImm(reg, value) = operands else {
                    panic!("Invalid!");
                };

                self.cpu.registers[reg as usize] = value;
                self.cpu.update_zn_flags(value);

                println!("-> LDI R{reg}: {value:#04X}");
            }
            Op::INC => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[reg as usize].wrapping_add(1);
                self.cpu.registers[reg as usize] = value;
                self.cpu.update_zn_flags(value);

                println!("-> INC R{reg}: {:#04X}", self.cpu.registers[reg as usize]);
            }
            Op::ADD | Op::SUB => {
                let Operands::RegReg(rd, rs) = operands else {
                    panic!("Invalid!");
                };

                let v1 = self.cpu.registers[rd as usize];
                let v2 = self.cpu.registers[rs as usize];

                let (result, carry) = match op {
                    Op::ADD => v1.overflowing_add(v2),
                    Op::SUB => v1.overflowing_sub(v2),
                    _ => unreachable!(),
                };

                self.cpu.registers[rd as usize] = result;
                self.cpu.update_zn_flags(result);

                if carry {
                    self.cpu.set_flag(CPU::FLAG_C);
                } else {
                    self.cpu.clear_flag(CPU::FLAG_C);
                }

                println!(
                    "-> {op} R{rd}, R{rs}: {v1} ({v1:#04X}) {} {v2} ({v2:#04X}) = {result} ({result:#04X})",
                    if op == Op::ADD { "+" } else { "-" }
                );
            }
            Op::JMP => {
                let Operands::Addr(a) = operands else {
                    panic!("Invalid!");
                };

                self.cpu.pc = a;
                pc_override = true;

                println!("-> JMP {a:#04X}");
            }
            Op::JZ | Op::JNZ => {
                let Operands::Addr(a) = operands else {
                    panic!("Invalid!");
                };

                if self.cpu.get_flag(CPU::FLAG_Z) == (op == Op::JZ) {
                    self.cpu.pc = a;
                    pc_override = true;

                    println!("-> {op} {a:#04X}");
                } else {
                    println!("-> {op} {a:#04X} (SKIP)");
                }
            }
            Op::PRINT => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[reg as usize];

                println!("-> PRINT R{reg}: '{}' ({value})", value as char);

                println!("{}", value as char);
            }
            Op::NOP => {
                println!("-> NOP");
            }
        }

        pc_override
    }
}

#[derive(Debug)]
pub enum Operands {
    None,
    Reg(u8),
    RegImm(u8, u8),
    RegReg(u8, u8),
    Addr(u8),
}

#[derive(Debug, PartialEq, Eq)]
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
    NOP,
}

impl Display for Op {
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
                Op::NOP => "NOP",
                Op::PRINT => "PRINT",
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
            _ => Op::NOP,
        }
    }
}
