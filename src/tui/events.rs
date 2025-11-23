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

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{io, ops::AddAssign, time::Duration};

use crate::cpu::CPU;

use super::app::App;

pub fn handle_events(app: &mut App) -> io::Result<()> {
    if event::poll(Duration::from_millis(10))? {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => app.exit(),
                    KeyCode::Char(' ') => app.next(),
                    KeyCode::Enter if app.none_editing() => app.running = !app.running,
                    KeyCode::Up => app.tick_rate.add_assign(Duration::from_millis(50)),
                    KeyCode::Down => {
                        app.tick_rate = app.tick_rate.saturating_sub(Duration::from_millis(50))
                    }
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('b') => app.editing_breakpoint = Some("0x".into()),
                    KeyCode::Char('m') => {
                        app.mem_view_start_editing = Some(format!("{:#04X}", app.mem_view_start))
                    }
                    KeyCode::Char('s') => {
                        app.stack_view_start_editing =
                            Some(format!("{:#04X}", app.stack_view_start))
                    }
                    key_code => {
                        App::handle_edit_input(
                            key_code,
                            &mut app.editing_breakpoint,
                            |addr| {
                                if app.breakpoints.contains(&addr) {
                                    app.breakpoints.retain(|x| *x != addr);
                                } else {
                                    app.breakpoints.push(addr);
                                }
                            },
                            0,
                        );
                        App::handle_edit_input(
                            key_code,
                            &mut app.mem_view_start_editing,
                            |addr| app.mem_view_start = addr,
                            0,
                        );
                        App::handle_edit_input(
                            key_code,
                            &mut app.stack_view_start_editing,
                            |addr| app.stack_view_start = addr,
                            CPU::STACK_MAX,
                        );
                    }
                }
            }
            _ => {}
        };
    }

    Ok(())
}
