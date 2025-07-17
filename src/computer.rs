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
            Op::ADD => {
                let rd = (opcode >> 3) & 0b0000_0111;
                let rs = opcode & 0b0000_0111;

                Operands::RegReg(rd, rs)
            }
            Op::JMP => {
                instruction_len = 2;
                let addr = self.cpu.memory[self.cpu.pc.wrapping_add(1) as usize];

                Operands::Addr(addr)
            }
            Op::PRINT => {
                let reg = opcode & 0b0000_0111;

                Operands::Reg(reg)
            }
        };

        // EXECUTE
        let mut pc_override = false;

        match op {
            Op::HLT => {
                self.cpu.is_halted = true;

                println!("-> HLT");
                return;
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

                self.cpu.registers[reg as usize] += 1;

                println!("-> INC R{reg}: {:#04X}", self.cpu.registers[reg as usize]);
            }
            Op::ADD => {
                let Operands::RegReg(rd, rs) = operands else {
                    panic!("Invalid!");
                };

                let v1 = self.cpu.registers[rd as usize];
                let v2 = self.cpu.registers[rs as usize];

                let (result, carry) = v1.overflowing_add(v2);
                self.cpu.registers[rd as usize] = result;
                self.cpu.update_zn_flags(result);

                if carry {
                    self.cpu.set_flag(CPU::FLAG_C);
                } else {
                    self.cpu.clear_flag(CPU::FLAG_C);
                }

                println!("-> ADD R{rd}, R{rs}: {v1:#04X} + {v2:#04X} = {result:#04X}");
            }
            Op::JMP => {
                let Operands::Addr(a) = operands else {
                    panic!("Invalid!");
                };

                self.cpu.pc = a;
                pc_override = true;

                println!("-> JMP {a:#04X}");
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

        if !pc_override {
            self.cpu.pc = self.cpu.pc.wrapping_add(instruction_len);
        }
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
