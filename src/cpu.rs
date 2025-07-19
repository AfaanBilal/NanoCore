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

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub registers: [u8; 16],
    pub pc: u8,
    pub sp: u8,
    pub memory: [u8; 256],
    pub flags: u8,
    pub is_halted: bool,
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    #[must_use]
    pub fn new() -> Self {
        CPU {
            registers: [0; 16],
            pc: 0,
            sp: 0xFF,
            memory: [0; 256],
            flags: 0,
            is_halted: false,
        }
    }

    pub const FLAG_Z: u8 = 0b0000_0001;
    pub const FLAG_C: u8 = 0b0000_0010;
    pub const FLAG_N: u8 = 0b0000_0100;
    pub const FLAG_Y: u8 = 0b0000_1000;

    pub fn set_flag(&mut self, bit: u8) {
        self.flags |= bit;
    }

    pub fn clear_flag(&mut self, bit: u8) {
        self.flags &= !bit;
    }

    pub fn get_flag(&self, bit: u8) -> bool {
        (self.flags & bit) != 0
    }

    pub fn update_zn_flags(&mut self, result: u8) {
        match result {
            0 => self.set_flag(Self::FLAG_Z),
            _ => self.clear_flag(Self::FLAG_Z),
        }

        match result & 0x80 {
            0 => self.clear_flag(Self::FLAG_N),
            _ => self.set_flag(Self::FLAG_N), // MSB non-zero
        }
    }

    pub fn print_state(&self, cycle: u8) {
        println!();

        Self::start_color();
        print!("┌{}┐", "─".repeat(self.registers.len() * 6 - 1));
        Self::end_color();

        println!();

        Self::start_color();
        print!(
            "│ Cycle: {cycle:03} / PC: {:03} / Flags: {:08b} {}│",
            self.pc,
            self.flags,
            " ".repeat(self.registers.len() * 6 - 41)
        );
        Self::end_color();

        println!();

        Self::start_color();
        print!("├");
        for i in 0..self.registers.len() {
            print!(
                "─────{}",
                if i == self.registers.len() - 1 {
                    "┤"
                } else {
                    "┬"
                }
            );
        }
        Self::end_color();

        println!();

        Self::start_color();
        print!("│");
        for i in 0..self.registers.len() {
            print!(" R{i} {}│", if i < 10 { " " } else { "" });
        }
        Self::end_color();

        println!();

        Self::start_color();
        print!("│");
        for i in 0..self.registers.len() {
            print!(" {:03} │", self.registers[i]);
        }
        Self::end_color();

        println!();

        Self::start_color();
        print!("└");
        for i in 0..self.registers.len() {
            print!(
                "─────{}",
                if i == self.registers.len() - 1 {
                    "┘"
                } else {
                    "┴"
                }
            );
        }
        Self::end_color();

        println!();
    }

    pub fn start_color() {
        print!("\x1b[92;40m");
    }

    pub fn end_color() {
        print!("\x1b[0m");
    }
}
