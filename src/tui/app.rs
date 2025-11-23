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

use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;
use std::{
    io,
    time::{Duration, Instant},
};

use crate::{cpu::CPU, nanocore::NanoCore};

use super::{events, ui};

#[derive(Debug)]
pub struct App {
    pub exit: bool,

    pub nano_core: NanoCore,

    pub filename: String,
    pub program: Vec<u8>,

    pub running: bool,
    pub tick_rate: Duration,
    pub last_tick: Instant,

    pub breakpoints: Vec<u8>,
    pub editing_breakpoint: Option<String>,

    pub mem_view_start: u8,
    pub mem_view_start_editing: Option<String>,

    pub stack_view_start: u8,
    pub stack_view_start_editing: Option<String>,
}

impl App {
    pub fn new(filename: String, program: Vec<u8>) -> Self {
        Self {
            exit: false,
            nano_core: NanoCore::new(),
            filename,
            program,
            running: false,
            tick_rate: Duration::from_millis(100),
            last_tick: Instant::now(),
            breakpoints: vec![],
            editing_breakpoint: None,
            mem_view_start: 0,
            mem_view_start_editing: None,
            stack_view_start: CPU::STACK_MAX,
            stack_view_start_editing: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, bytes: &[u8]) -> io::Result<()> {
        self.program = bytes.to_vec();
        self.nano_core
            .load_program(&self.program, 0x00)
            .unwrap_or_else(|e| {
                eprintln!("Error loading program: {}", e);
                std::process::exit(1);
            });

        while !self.exit {
            terminal.draw(|frame| ui::draw(self, frame))?;
            events::handle_events(self)?;

            if self.running {
                self.run_full();
            }
        }

        Ok(())
    }

    pub fn handle_edit_input<F>(
        key_code: KeyCode,
        editing: &mut Option<String>,
        mut apply: F,
        reset_value: u8,
    ) where
        F: FnMut(u8),
    {
        if let Some(input) = editing {
            match key_code {
                KeyCode::Esc => *editing = None,
                KeyCode::Char('k') => {
                    *editing = None;
                    apply(reset_value);
                }
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    let addr = Self::parse_addr(input);
                    apply(addr);
                    *editing = None;
                }
                _ => {}
            }
        }
    }

    pub fn none_editing(&self) -> bool {
        self.editing_breakpoint.is_none()
            && self.mem_view_start_editing.is_none()
            && self.stack_view_start_editing.is_none()
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn next(&mut self) {
        if !self.nano_core.cpu.is_halted
            && let Err(e) = self.nano_core.cycle()
        {
            eprintln!("Emulator error: {}", e);
            self.nano_core.cpu.is_halted = true;
        }
    }

    pub fn run_full(&mut self) {
        if self.nano_core.cpu.is_halted || self.breakpoints.contains(&self.nano_core.cpu.pc) {
            self.running = false;
            return;
        }

        if Instant::now().duration_since(self.last_tick) > self.tick_rate {
            self.last_tick = Instant::now();
            if let Err(e) = self.nano_core.cycle() {
                eprintln!("Emulator error: {}", e);
                self.nano_core.cpu.is_halted = true;
                self.running = false;
            }
        }
    }

    pub fn reset(&mut self) {
        self.nano_core = NanoCore::new();
        self.nano_core
            .load_program(&self.program, 0x00)
            .unwrap_or_else(|e| {
                eprintln!("Error reloading program: {}", e);
            });
        self.running = false;
    }

    fn parse_addr(s: &str) -> u8 {
        let addr = s.strip_prefix("0x").unwrap_or("0");

        if let Ok(addr) = hex::decode(addr)
            && let Some(&addr) = addr.first()
        {
            return addr;
        }

        0
    }
}
