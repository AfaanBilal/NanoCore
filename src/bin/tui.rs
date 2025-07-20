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

use std::{fs, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use nanocore::{assembler::Assembler, nanocore::NanoCore};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Spacing},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph},
};

#[derive(Debug, Default)]
pub struct App {
    nano_core: NanoCore,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let title = Line::from(" NanoCore ".green().on_black().bold());
        let instructions = Line::from(vec![
            " Next ".into(),
            "<Space>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        let main = Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)])
            .margin(1)
            .split(frame.area());

        let description = Text::from(vec![
            Line::from("(c) Afaan Bilal".blue()).centered(),
            Line::from("https://afaan.dev".blue()).centered(),
            Line::from("https://github.com/AfaanBilal/nanocore".blue()).centered(),
        ]);

        frame.render_widget(Paragraph::new(description).block(block), main[0]);

        let inner = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main[1]);

        let cpu_block = Block::bordered()
            .padding(Padding::top(1))
            .title(Line::from(" CPU ").centered())
            .title(Span::styled(
                (if self.nano_core.cpu.is_halted {
                    " HLT "
                } else {
                    ""
                })
                .to_string(),
                Style::default().bg(Color::Red).fg(Color::White),
            ));

        let cpu_block_inner = cpu_block.inner(inner[0]);
        frame.render_widget(cpu_block, inner[0]);

        let cpu = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(30),
            Constraint::Percentage(7),
            Constraint::Fill(1),
        ])
        .split(cpu_block_inner);

        let cpu_top = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(cpu[0]);

        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    " {:#04X} ({:03}) ",
                    self.nano_core.cycle, self.nano_core.cycle
                ))
                .centered(),
            )
            .block(Block::bordered().title(" Cycle ")),
            cpu_top[0],
        );
        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    " {:#04X} ({:03}) ",
                    self.nano_core.cpu.pc, self.nano_core.cpu.pc
                ))
                .centered(),
            )
            .block(Block::bordered().title(" PC ")),
            cpu_top[1],
        );
        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    " {:#04X} ({:03}) ",
                    self.nano_core.cpu.sp, self.nano_core.cpu.sp
                ))
                .centered(),
            )
            .block(Block::bordered().title(" SP ")),
            cpu_top[2],
        );
        frame.render_widget(
            Paragraph::new(Line::from(format!(" {:08b} ", self.nano_core.cpu.flags)).centered())
                .block(Block::bordered().title(" Flags ")),
            cpu_top[3],
        );

        let register_block = Block::bordered().title(" Registers ");
        let register_block_inner = register_block.inner(cpu[1]);

        let registers = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(Spacing::Space(1))
            .split(register_block_inner);

        let registers_top =
            Layout::horizontal([Constraint::Percentage(100 / 8); 8]).split(registers[0]);
        let registers_bottom =
            Layout::horizontal([Constraint::Percentage(100 / 8); 8]).split(registers[1]);

        frame.render_widget(register_block, cpu[1]);

        for i in 0..8 {
            frame.render_widget(
                Paragraph::new(Text::from(vec![
                    Line::from(format!("{:04}", self.nano_core.cpu.registers[i])),
                    Line::from(format!("{:#04X}", self.nano_core.cpu.registers[i])),
                ]))
                .block(Block::bordered().title(format!(" R{i} "))),
                registers_top[i],
            );
        }

        for i in 8..16 {
            frame.render_widget(
                Paragraph::new(Text::from(vec![
                    Line::from(format!("{:04}", self.nano_core.cpu.registers[i])),
                    Line::from(format!("{:#04X}", self.nano_core.cpu.registers[i])),
                ]))
                .block(Block::bordered().title(format!(" R{i} "))),
                registers_bottom[i - 8],
            );
        }

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                self.nano_core.current_instruction.clone().green(),
                if self.nano_core.current_skipped {
                    " (SKIP)".red()
                } else {
                    "".red()
                },
            ]))
            .block(Block::bordered().title(" Current Instruction ")),
            cpu[2],
        );
        frame.render_widget(
            Paragraph::new(self.nano_core.output.clone())
                .block(Block::bordered().title(" Output ")),
            cpu[3],
        );

        let memory_block = Block::bordered()
            .title(Line::from(" Memory ").centered())
            .padding(Padding::top(1));
        let memory_block_inner = memory_block.inner(inner[1]);
        frame.render_widget(memory_block, inner[1]);

        let memory = Layout::horizontal([Constraint::Percentage(20), Constraint::Fill(1)])
            .split(memory_block_inner);

        let mut addr_vec = vec![];
        let mut mem_vec = vec![];
        for i in 0..self.nano_core.cpu.memory.len() {
            mem_vec.push(Line::from(format!(
                " {:08b} ({:#04X})",
                self.nano_core.cpu.memory[i], self.nano_core.cpu.memory[i]
            )));

            if i as u8 == self.nano_core.cpu.pc {
                addr_vec.push(Line::from(format!("-> {i:#04X}")));
            } else {
                addr_vec.push(Line::from(format!("   {i:#04X}")));
            }

            if self.nano_core.cpu.memory[i] == 0 {
                break;
            }
        }

        frame.render_widget(
            Paragraph::new(Text::from(addr_vec)).block(Block::bordered().title(" Address ")),
            memory[0],
        );

        frame.render_widget(
            Paragraph::new(Text::from(mem_vec)).block(Block::bordered().title(" Data ")),
            memory[1],
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char(' ') => self.next(),
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next(&mut self) {
        self.nano_core.cycle();
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    // let bin = std::env::args().nth(1).expect("Missing filename.");
    let bin = "programs/counter.nca";

    let bytes = if bin.ends_with(".nca") {
        let asm = fs::read_to_string(bin).unwrap();

        let mut c = Assembler::default();
        c.assemble(&asm);

        c.program
    } else {
        fs::read(bin).unwrap()
    };

    let mut app = App {
        nano_core: NanoCore::new(),
        exit: false,
    };

    app.nano_core.load_program(&bytes, 0x00);

    let app = app.run(&mut terminal);
    ratatui::restore();
    app
}
