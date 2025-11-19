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

use crate::{Op, cpu::CPU, end_color, start_color};

#[derive(Debug, Default)]
pub struct NanoCore {
    pub cpu: CPU,
    pub cycle: u16,
    pub current_instruction: String,
    pub current_instruction_bin: String,
    pub current_skipped: bool,
    pub instruction_log: Vec<String>,
    pub output: String,

    pub print: bool,
    pub print_state: bool,
    pub print_instructions: bool,
}

impl NanoCore {
    pub const MAX_CYCLES: u16 = 1024;

    #[must_use]
    pub fn new() -> Self {
        NanoCore {
            cpu: CPU::new(),
            cycle: 0,
            current_instruction: String::new(),
            current_instruction_bin: String::new(),
            current_skipped: false,
            instruction_log: vec![],
            output: String::new(),

            print: false,
            print_state: false,
            print_instructions: false,
        }
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
    }

    pub fn run(&mut self) {
        self.print_colored(&format!(
            "{}  NanoCore Start  {}",
            "━".repeat(45),
            "━".repeat(50)
        ));

        while !self.cpu.is_halted {
            if self.print_state {
                self.cpu.print_state(self.cycle);
            }

            if self.cycle >= Self::MAX_CYCLES {
                println!("\n== FORCE HALT - max cycles ==\n");
                break;
            }

            self.cycle();
        }

        self.print_colored(&format!(
            "{}  NanoCore Halt  {}",
            "━".repeat(46),
            "━".repeat(50)
        ));
    }

    pub fn print_colored(&self, s: &str) {
        if !self.print_state {
            return;
        }

        println!();
        start_color();
        print!("{s}");
        end_color();
        println!();
    }

    pub fn cycle(&mut self) {
        let (op, operands) = self.fetch_decode();

        let pc_override = self.execute(op, operands);

        if self.cpu.is_halted {
            return;
        }

        if !pc_override {
            self.cpu.pc = self.cpu.pc.wrapping_add(op.instruction_len());
        }

        self.cycle += 1;
    }

    pub fn fetch_decode(&mut self) -> (Op, Operands) {
        // FETCH
        let opcode = self.cpu.memory[self.cpu.pc as usize];
        let byte_2 = self.cpu.memory[self.cpu.pc.wrapping_add(1) as usize];
        let byte_3 = self.cpu.memory[self.cpu.pc.wrapping_add(2) as usize];

        self.current_instruction_bin = format!("{opcode:08b} │ {opcode:#04X} │ {opcode:03}");

        // DECODE
        let op: Op = opcode.into();

        let operands = match op {
            Op::HLT | Op::NOP | Op::RET => Operands::None,
            Op::LDI | Op::ADDI | Op::SUBI | Op::MULI | Op::DIVI | Op::MODI => {
                Operands::RegImm(byte_2, byte_3)
            }
            Op::LDA | Op::STORE => Operands::RegAddr(byte_2, byte_3),
            Op::PUSH | Op::POP | Op::INC | Op::DEC | Op::NOT | Op::SHL | Op::SHR | Op::ROL | Op::ROR | Op::PRINT => {
                Operands::Reg(byte_2)
            }
            Op::LDR
            | Op::MOV
            | Op::ADD
            | Op::SUB
            | Op::AND
            | Op::OR
            | Op::XOR
            | Op::CMP
            | Op::MUL
            | Op::DIV
            | Op::MOD => Operands::RegReg((byte_2 >> 4) & 0x0F, byte_2 & 0x0F),
            Op::JMP | Op::JZ | Op::JNZ | Op::CALL => Operands::Addr(byte_2),
        };

        (op, operands)
    }

    pub fn execute(&mut self, op: Op, operands: Operands) -> bool {
        let mut pc_override = false;

        if !self.current_instruction.is_empty() {
            self.instruction_log.push(format!(
                "{} {}",
                self.current_instruction,
                if self.current_skipped { "(SKIP)" } else { "" }
            ));
        }

        self.current_skipped = false;

        match op {
            Op::HLT => {
                self.cpu.is_halted = true;
                self.current_instruction = op.into();
            }
            Op::NOP => {
                self.current_instruction = op.into();
            }
            Op::LDI => {
                let Operands::RegImm(reg, value) = operands else {
                    panic!("Invalid!");
                };

                self.cpu.registers[reg as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction = format!("{op}   R{reg} {value:#04X}| ({value:03})");
            }
            Op::LDA => {
                let Operands::RegAddr(reg, addr) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.memory[addr as usize];
                self.cpu.registers[reg as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction = format!("{op}   R{reg} {addr:#04X}| ({value:03})");
            }
            Op::STORE => {
                let Operands::RegAddr(reg, addr) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[reg as usize];
                self.cpu.memory[addr as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction = format!("{op}   R{reg} {addr:#04X}| ({value:03})");
            }
            Op::LDR => {
                let Operands::RegReg(rd, rs) = operands else {
                    panic!("Invalid!");
                };

                let addr = self.cpu.registers[rs as usize];
                let value = self.cpu.memory[addr as usize];
                self.cpu.registers[rd as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction = format!("{op}   R{rd} R{rs}| {addr:#04X} ({value:03})");
            }
            Op::MOV => {
                let Operands::RegReg(rd, rs) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[rs as usize];
                self.cpu.registers[rd as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction = format!("{op}   R{rd} R{rs}| ({value:03})");
            }
            Op::PUSH => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid");
                };

                if self.cpu.sp == CPU::STACK_MIN {
                    panic!("Error: Stack Overflow SP: {}", self.cpu.sp);
                }

                let value = self.cpu.registers[reg as usize];

                self.cpu.memory[self.cpu.sp as usize] = value;
                self.cpu.sp = self.cpu.sp.wrapping_sub(1);
                self.cpu.update_zn_flags(value);

                self.current_instruction =
                    format!("PUSH  R{reg}| ({value:03}) (SP: {:#04X})", self.cpu.sp);
            }
            Op::POP => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid");
                };

                if self.cpu.sp == CPU::STACK_MAX {
                    panic!("Error: Stack Underflow SP: {}", self.cpu.sp);
                }

                self.cpu.sp = self.cpu.sp.wrapping_add(1);

                let value = self.cpu.memory[self.cpu.sp as usize];

                self.cpu.registers[reg as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction =
                    format!("POP   R{reg}| ({value:03}) (SP: {:#04X})", self.cpu.sp);
            }
            Op::INC | Op::DEC => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid!");
                };

                let value = if op == Op::INC {
                    self.cpu.registers[reg as usize].wrapping_add(1)
                } else {
                    self.cpu.registers[reg as usize].wrapping_sub(1)
                };

                self.cpu.registers[reg as usize] = value;
                self.cpu.update_zn_flags(value);

                self.current_instruction =
                    format!("{op}   R{reg}| {:#04X}", self.cpu.registers[reg as usize]);
            }
            Op::ADD | Op::SUB | Op::MUL | Op::DIV | Op::MOD => {
                let Operands::RegReg(rd, rs) = operands else {
                    panic!("Invalid!");
                };

                let v1 = self.cpu.registers[rd as usize];
                let v2 = self.cpu.registers[rs as usize];

                let (result, carry) = match op {
                    Op::ADD => v1.overflowing_add(v2),
                    Op::SUB => v1.overflowing_sub(v2),
                    Op::MUL => v1.overflowing_mul(v2),
                    Op::DIV => v1.overflowing_div(v2),
                    Op::MOD => v1.overflowing_rem(v2),
                    _ => unreachable!(),
                };

                self.cpu.registers[rd as usize] = result;
                self.cpu.update_zn_flags(result);

                if carry {
                    self.cpu.set_flag(CPU::FLAG_C);
                } else {
                    self.cpu.clear_flag(CPU::FLAG_C);
                }

                self.current_instruction = format!(
                    "{op}   R{rd} R{rs}| {v1:03} ({v1:#04X}) {} {v2:03} ({v2:#04X}) = {result:03} ({result:#04X})",
                    if op == Op::ADD { "+" } else { "-" }
                );
            }
            Op::ADDI | Op::SUBI | Op::MULI | Op::DIVI | Op::MODI => {
                let Operands::RegImm(reg, v2) = operands else {
                    panic!("Invalid!");
                };

                let v1 = self.cpu.registers[reg as usize];

                let (result, carry) = match op {
                    Op::ADDI => v1.overflowing_add(v2),
                    Op::SUBI => v1.overflowing_sub(v2),
                    Op::MULI => v1.overflowing_mul(v2),
                    Op::DIVI => v1.overflowing_div(v2),
                    Op::MODI => v1.overflowing_rem(v2),
                    _ => unreachable!(),
                };

                self.cpu.registers[reg as usize] = result;
                self.cpu.update_zn_flags(result);

                if carry {
                    self.cpu.set_flag(CPU::FLAG_C);
                } else {
                    self.cpu.clear_flag(CPU::FLAG_C);
                }

                self.current_instruction = format!(
                    "{op}   R{reg} {v2:03}| {v1:03} ({v1:#04X}) {} {v2:03} ({v2:#04X}) = {result:03} ({result:#04X})",
                    if op == Op::ADD { "+" } else { "-" }
                );
            }
            Op::AND | Op::OR | Op::XOR | Op::CMP => {
                let Operands::RegReg(rd, rs) = operands else {
                    panic!("Invalid!");
                };

                let v1 = self.cpu.registers[rs as usize];
                let v2 = self.cpu.registers[rd as usize];

                let result = match op {
                    Op::AND => v1 & v2,
                    Op::OR => v1 | v2,
                    Op::XOR => v1 ^ v2,
                    Op::CMP => {
                        if v1 == v2 {
                            0
                        } else if v1 > v2 {
                            1
                        } else {
                            2
                        }
                    }
                    _ => unreachable!(),
                };

                self.cpu.registers[rd as usize] = result;
                self.cpu.update_zn_flags(result);

                self.current_instruction = format!(
                    "{op}   R{rd} R{rs}| {v1:03} ({v1:#04X}) {} {v2:03} ({v2:#04X}) = {result:03} ({result:#04X})",
                    match op {
                        Op::AND => "&",
                        Op::OR => "|",
                        Op::XOR => "^",
                        Op::CMP => "==",
                        _ => unreachable!(),
                    }
                );
            }
            Op::NOT => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[reg as usize];
                let result = !value;

                self.cpu.registers[reg as usize] = result;
                self.cpu.update_zn_flags(result);

                self.current_instruction = format!(
                    "{op}   R{reg}| ! {value:03} ({value:#04X}) ({value:08b}) = {result:03} ({result:#04X}) ({result:08b})",
                );
            }
            Op::JMP => {
                let Operands::Addr(a) = operands else {
                    panic!("Invalid!");
                };

                self.cpu.pc = a;
                pc_override = true;

                self.current_instruction =
                    format!("JMP   {a:#04X}| Mem({:#04X})", self.cpu.memory[a as usize]);
            }
            Op::JZ | Op::JNZ => {
                let Operands::Addr(a) = operands else {
                    panic!("Invalid!");
                };

                self.current_instruction = format!(
                    "{op}{}   {a:#04X}| Z({}) Mem({:#04X})",
                    if op == Op::JZ { " " } else { "" },
                    self.cpu.get_flag(CPU::FLAG_Z) as u8,
                    self.cpu.memory[a as usize],
                );

                if self.cpu.get_flag(CPU::FLAG_Z) == (op == Op::JZ) {
                    self.cpu.pc = a;
                    pc_override = true;
                } else {
                    self.current_skipped = true;

                    if self.print_instructions {
                        print!("(SKIP) ");
                    }
                }
            }
            Op::PRINT => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[reg as usize];

                self.current_instruction = format!(
                    "{op} R{reg}| '{}' ({value:03}) ({value:#04X})",
                    value as char
                );
                self.output.push(value as char);

                if self.print {
                    print!("{}", value as char);
                }
            }
            Op::SHL | Op::SHR | Op::ROL | Op::ROR => {
                let Operands::Reg(reg) = operands else {
                    panic!("Invalid!");
                };

                let value = self.cpu.registers[reg as usize];

                let (result, carry) = match op {
                    Op::SHL => value.overflowing_shl(1),
                    Op::SHR => value.overflowing_shr(1),
                    Op::ROL => (value.rotate_left(1), (value & 0x80) != 0),
                    Op::ROR => (value.rotate_right(1), (value & 0x01) != 0),
                    _ => unreachable!(),
                };

                self.cpu.registers[reg as usize] = result;
                self.cpu.update_zn_flags(result);

                if carry {
                    self.cpu.set_flag(CPU::FLAG_C);
                } else {
                    self.cpu.clear_flag(CPU::FLAG_C);
                }

                self.current_instruction = format!(
                    "{op}   R{reg}| {value:03} ({value:08b}) {} 1 = {result:03} ({result:08b})",
                    match op {
                        Op::SHL => "<<",
                        Op::SHR => ">>",
                        Op::ROL => "ROL",
                        Op::ROR => "ROR",
                        _ => unreachable!(),
                    }
                );
            }
            Op::CALL => {
                let Operands::Addr(a) = operands else {
                    panic!("Invalid!");
                };

                if self.cpu.sp == CPU::STACK_MIN {
                    panic!("Error: Stack Overflow SP: {}", self.cpu.sp);
                }

                self.cpu.memory[self.cpu.sp as usize] = self.cpu.pc.wrapping_add(2);
                self.cpu.sp = self.cpu.sp.wrapping_sub(1);

                self.cpu.pc = a;
                pc_override = true;

                self.current_instruction =
                    format!("CALL  {a:#04X}| Mem({:#04X})", self.cpu.memory[a as usize]);
            }
            Op::RET => {
                if self.cpu.sp == CPU::STACK_MAX {
                    panic!("Error: Stack Underflow SP: {}", self.cpu.sp);
                }

                self.cpu.sp = self.cpu.sp.wrapping_add(1);
                self.cpu.pc = self.cpu.memory[self.cpu.sp as usize];
                pc_override = true;

                self.current_instruction = format!(
                    "RET  | {:#04X} Mem({:#04X})",
                    self.cpu.pc, self.cpu.memory[self.cpu.pc as usize]
                );
            }
        }

        if self.print_instructions {
            println!("-> {}", self.current_instruction);
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
    RegAddr(u8, u8),
    Addr(u8),
}
