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

use std::{
    fs, io,
    ops::AddAssign,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use nanocore::{Op, assembler::Assembler, cpu::CPU, nanocore::NanoCore};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, Padding, Paragraph},
};

#[derive(Debug)]
pub struct App {
    exit: bool,

    nano_core: NanoCore,

    filename: String,
    program: Vec<u8>,

    running: bool,
    tick_rate: Duration,
    last_tick: Instant,

    breakpoints: Vec<u8>,
    editing_breakpoint: Option<String>,

    mem_view_start: u8,
    mem_view_start_editing: Option<String>,

    stack_view_start: u8,
    stack_view_start_editing: Option<String>,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, bytes: &[u8]) -> io::Result<()> {
        self.program = bytes.to_vec();
        self.nano_core.load_program(&self.program, 0x00);

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;

            if self.running {
                self.run_full();
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let title = Line::from(" NanoCore ".green().on_black().bold());
        let instructions = Line::from(vec![
            " Next Instruction ".into(),
            "<Space>".light_blue().bold(),
            if self.running {
                " | Stop ".into()
            } else {
                " | Run ".into()
            },
            "<Enter>".light_blue().bold(),
            " | ".into(),
            "●".red(),
            " Breakpoint ".into(),
            (if self.breakpoints.is_empty() {
                "<B>".light_blue()
            } else {
                "<B>".light_green()
            })
            .bold(),
            " | Memory View ".into(),
            (if self.mem_view_start == 0 {
                "<M>".light_blue()
            } else {
                "<M>".light_green()
            })
            .bold(),
            " | Stack View ".into(),
            (if self.stack_view_start == CPU::STACK_MAX {
                "<S>".light_blue()
            } else {
                "<S>".light_green()
            })
            .bold(),
            " | Tick Rate (".into(),
            "<⬆>".light_blue().bold(),
            " +50ms) (".into(),
            "<⬇>".light_blue().bold(),
            " -50ms) | Reset ".into(),
            "<R>".light_blue().bold(),
            " | Quit ".into(),
            "<Q> ".light_blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        let main = Layout::vertical([Constraint::Percentage(10), Constraint::Fill(1)])
            .margin(1)
            .split(frame.area());

        let description = Text::from(vec![
            Line::from(
                "(c) Afaan Bilal <https://afaan.dev> | https://github.com/AfaanBilal/NanoCore"
                    .gray(),
            )
            .centered(),
            Line::from(
                format!(
                    "Running: {} | Tick rate: {}ms",
                    if self.running { "Yes" } else { "No" },
                    self.tick_rate.as_millis()
                )
                .cyan(),
            )
            .centered(),
        ]);

        frame.render_widget(Paragraph::new(description).block(block), main[0]);

        let inner =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Fill(1)]).split(main[1]);

        let cpu_block = Block::default()
            .borders(Borders::TOP)
            .title(Line::from(" CPU ").centered().bold());

        let cpu_block_inner = cpu_block.inner(inner[0]);
        frame.render_widget(cpu_block, inner[0]);

        let cpu = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Percentage(7),
            Constraint::Percentage(30),
            Constraint::Fill(1),
        ])
        .split(cpu_block_inner);

        let cpu_top = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Fill(1),
        ])
        .split(cpu[0]);

        let state_line = if self.nano_core.cpu.is_halted {
            Line::from(" HLT ".white().on_red())
        } else if self.breakpoints.contains(&self.nano_core.cpu.pc) {
            Line::from(" BRK ".white().on_red())
        } else {
            Line::from(" RUN ".green().on_black())
        };

        frame.render_widget(
            Paragraph::new(state_line)
                .centered()
                .block(Block::bordered().title("State")),
            cpu_top[0],
        );
        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    " {:#04X} ({:03}) ",
                    self.nano_core.cycle, self.nano_core.cycle
                ))
                .centered(),
            )
            .block(Block::bordered().title(" Cycle ")),
            cpu_top[1],
        );
        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    " {:#04X} ({:03}) ",
                    self.nano_core.cpu.pc, self.nano_core.cpu.pc
                ))
                .centered(),
            )
            .block(Block::bordered().title(Line::from(vec![" ►".magenta(), " PC ".white()]))),
            cpu_top[2],
        );
        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    " {:#04X} ({:03}) ",
                    self.nano_core.cpu.sp, self.nano_core.cpu.sp
                ))
                .centered(),
            )
            .block(Block::bordered().title(Line::from(vec![" ►".cyan(), " SP ".white()]))),
            cpu_top[3],
        );

        let flag_z = self.nano_core.cpu.get_flag(CPU::FLAG_Z);
        let flag_c = self.nano_core.cpu.get_flag(CPU::FLAG_C);
        let flag_n = self.nano_core.cpu.get_flag(CPU::FLAG_N);
        let flag_y = self.nano_core.cpu.get_flag(CPU::FLAG_Y);

        let mut flag_line_z = Span::raw(format!("Z({:01}) ", flag_z as u8));
        let mut flag_line_c = Span::raw(format!("C({:01}) ", flag_c as u8));
        let mut flag_line_n = Span::raw(format!("N({:01}) ", flag_n as u8));
        let mut flag_line_y = Span::raw(format!("Y({:01})", flag_y as u8));

        if !flag_z {
            flag_line_z = flag_line_z.dark_gray();
        }

        if !flag_c {
            flag_line_c = flag_line_c.dark_gray();
        }

        if !flag_n {
            flag_line_n = flag_line_n.dark_gray();
        }

        if !flag_y {
            flag_line_y = flag_line_y.dark_gray();
        }

        let mut flag_line =
            Line::from(vec![flag_line_z, flag_line_c, flag_line_n, flag_line_y]).centered();

        if self.nano_core.cpu.flags == 0 {
            flag_line = flag_line.dark_gray();
        }

        frame.render_widget(
            Paragraph::new(flag_line).block(Block::bordered().title(" Flags ")),
            cpu_top[4],
        );

        frame.render_widget(
            Paragraph::new(
                Line::from(format!(
                    "{} │ {} bytes ({:#04X})",
                    std::path::Path::new(&self.filename)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    self.program.len(),
                    self.program.len()
                ))
                .centered(),
            )
            .block(Block::bordered().title(" Current Program ")),
            cpu_top[5],
        );

        // -- Registers

        let register_block = Block::bordered()
            .padding(Padding::left(1))
            .title(Line::from(" Registers ").centered());
        let register_block_inner = register_block.inner(cpu[1]);

        let registers = Layout::horizontal([Constraint::Fill(1); 16])
            .spacing(1)
            .split(register_block_inner);

        frame.render_widget(register_block, cpu[1]);

        for i in 0..self.nano_core.cpu.registers.len() {
            let mut dec_line = Line::from(format!("{:04}", self.nano_core.cpu.registers[i]));
            let mut hex_line = Line::from(format!("{:#04X}", self.nano_core.cpu.registers[i]));

            if self.nano_core.cpu.registers[i] == 0 {
                dec_line = dec_line.dark_gray();
                hex_line = hex_line.dark_gray();
            }

            let mut reg_block = Block::default();

            if i < 15 {
                reg_block = reg_block.borders(Borders::RIGHT)
            }

            frame.render_widget(
                Paragraph::new(Text::from(vec![dec_line, hex_line]))
                    .block(reg_block.title(Line::from(format!("R{i}")))),
                registers[i],
            );
        }

        // -- Current instruction

        let (op, args, rest) = Self::get_instruction_parts(&self.nano_core.current_instruction);

        let mut op_span = Span::raw(format!("{op:5}")).cyan();
        let mut op_bin_span = Span::raw(&self.nano_core.current_instruction_bin).light_cyan();

        if !op.is_empty() {
            if matches!(
                Op::from(op.as_str()),
                Op::JMP | Op::JZ | Op::JNZ | Op::CALL | Op::RET
            ) {
                op_span = op_span.magenta();
                op_bin_span = op_bin_span.magenta();
            }

            if Op::from(op.as_str()) == Op::HLT {
                op_span = op_span.red();
                op_bin_span = op_bin_span.red();
            }
        }

        let ci_block = Block::bordered().title(" Current Instruction ");
        let ci_block_inner = ci_block.inner(cpu[2]);

        let ci_columns =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(ci_block_inner);

        frame.render_widget(ci_block, cpu[2]);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                format!("{:03}", self.nano_core.instruction_log.len()).dim(),
                Span::raw(" "),
                op_span,
                Span::raw(format!(" {:<8}", args.trim())).green(),
                Span::raw(format!(" │{rest}")).dim(),
            ])),
            ci_columns[0],
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                " ".white(),
                op_bin_span,
                if self.nano_core.current_skipped {
                    " │ (SKIP)".red()
                } else {
                    "".red()
                },
            ]))
            .block(Block::default().borders(Borders::LEFT)),
            ci_columns[1],
        );

        // -- Instruction log

        let log_block = Block::bordered().title(" Instruction Log ");
        let log_block_inner = log_block.inner(cpu[3]);

        let log_columns =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(log_block_inner);

        let log_left = self.get_instruction_list(0, 14);
        let log_right = self
            .get_instruction_list(14, 14)
            .block(Block::default().borders(Borders::LEFT));

        frame.render_widget(log_block, cpu[3]);
        frame.render_widget(log_left, log_columns[0]);
        frame.render_widget(log_right, log_columns[1]);

        // -- Output

        let bottom_block = Block::default();
        let bottom_block_inner = bottom_block.inner(cpu[4]);

        let bottom_columns = Layout::horizontal([Constraint::Percentage(70), Constraint::Fill(1)])
            .split(bottom_block_inner);

        let output = Paragraph::new(self.nano_core.output.clone())
            .block(Block::bordered().title(" Output "));
        frame.render_widget(output, bottom_columns[0]);

        // -- Stack

        let stack_block = Block::default()
            .borders(Borders::TOP)
            .title(Line::from(" Stack ").centered());
        let stack_block_inner = stack_block.inner(bottom_columns[1]);
        frame.render_widget(stack_block, bottom_columns[1]);

        let stack = Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)])
            .split(stack_block_inner);

        let mut stack_addr_vec = vec![Line::from("  Hex  Dec".light_blue())];
        let mut stack_mem_vec = vec![Line::from(" Bin       Hex   Dec".light_blue())];

        if self.stack_view_start < CPU::STACK_MAX {
            stack_addr_vec.push(Line::from("  ···  ···".dim()));
            stack_mem_vec.push(Line::from(" ···      ···  ···".dim()));
        }

        let mut sv_start = self.stack_view_start as usize;
        if sv_start < 32 {
            sv_start = 32;
        }

        for i in ((sv_start - 32)..=sv_start).rev() {
            let mut mem_line = Line::from(format!(
                " {:08b}  {:#04X}  {:03}  ",
                self.nano_core.cpu.memory[i],
                self.nano_core.cpu.memory[i],
                self.nano_core.cpu.memory[i],
            ));

            if self.nano_core.cpu.memory[i] == 0 {
                mem_line = mem_line.dim();
            }

            if i as u8 == self.nano_core.cpu.sp {
                stack_addr_vec.push(Line::from(vec![
                    "► ".cyan(),
                    format!("{i:#04X} {i:03} ").white(),
                ]));
                mem_line = mem_line.white().on_cyan();
            } else {
                stack_addr_vec.push(Line::from(format!("  {i:#04X} {i:03} ")).dark_gray());
            }

            stack_mem_vec.push(mem_line);
        }

        frame.render_widget(
            Paragraph::new(Text::from(stack_addr_vec)).block(Block::bordered().title(" Address ")),
            stack[0],
        );

        frame.render_widget(
            Paragraph::new(Text::from(stack_mem_vec)).block(Block::bordered().title(" Data ")),
            stack[1],
        );

        frame.render_widget(bottom_block, cpu[4]);

        // -- Memory

        let memory_block = Block::default()
            .borders(Borders::TOP)
            .title(Line::from(" Memory ").centered().bold());
        let memory_block_inner = memory_block.inner(inner[1]);
        frame.render_widget(memory_block, inner[1]);

        let memory = Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)])
            .split(memory_block_inner);

        let mut addr_vec = vec![Line::from("  Hex  Dec".light_blue())];
        let mut mem_vec = vec![Line::from(" Bin      Hex  Dec Op".light_blue())];

        if self.mem_view_start > 0 {
            addr_vec.push(Line::from("  ···  ···".dim()));
            mem_vec.push(Line::from(" ···      ···  ··· ··".dim()));
        }

        let mut skip_bytes = 0;
        for i in (self.mem_view_start as usize)..(self.nano_core.cpu.memory.len()) {
            let op: Op = if skip_bytes == 0 {
                let op: Op = self.nano_core.cpu.memory[i].into();

                skip_bytes = op.instruction_len() - 1;

                op
            } else {
                skip_bytes -= 1;

                Op::NOP
            };

            let op_str = if op == Op::NOP {
                "·".to_string()
            } else {
                op.to_string()
            };

            let mut op_span = Span::raw(format!("{op_str:5}"));
            op_span = match op {
                _ if i as u8 == self.nano_core.cpu.pc => op_span.white(),
                Op::NOP => op_span.dim(),
                Op::HLT => op_span.red().dim(),
                Op::JMP | Op::JZ | Op::JNZ | Op::CALL | Op::RET => op_span.magenta(),
                _ => op_span.cyan(),
            };

            let mut mem_line = Line::from(vec![
                Span::raw(format!(
                    " {:08b} {:#04X} {:03} ",
                    self.nano_core.cpu.memory[i],
                    self.nano_core.cpu.memory[i],
                    self.nano_core.cpu.memory[i],
                )),
                op_span,
            ]);

            if self.nano_core.cpu.memory[i] == 0 {
                mem_line = mem_line.dark_gray();
            }

            if i as u8 == self.nano_core.cpu.pc {
                addr_vec.push(Line::from(vec![
                    "► ".magenta(),
                    format!("{i:#04X} {i:03} ").white(),
                ]));
                mem_line = mem_line.white().on_magenta();
            } else if self.breakpoints.contains(&(i as u8)) {
                addr_vec.push(Line::from(format!("● {i:#04X} {i:03} ").red()));
                mem_line = mem_line.white().on_red();
            } else {
                addr_vec.push(Line::from(format!("  {i:#04X} {i:03}")).dark_gray());
            }

            mem_vec.push(mem_line);
        }

        frame.render_widget(
            Paragraph::new(Text::from(addr_vec)).block(Block::bordered().title(" Address ")),
            memory[0],
        );

        frame.render_widget(
            Paragraph::new(Text::from(mem_vec)).block(Block::bordered().title(" Data ")),
            memory[1],
        );

        if let Some(breakpoint) = &self.editing_breakpoint {
            let mut bp_modal_lines = vec![
                Line::from(vec![
                    "Address: ".into(),
                    format!(" {:20} ", breakpoint.as_str())
                        .black()
                        .on_white()
                        .bold(),
                    " ↵".bold(),
                ]),
                "".into(),
                Line::from(vec!["<Esc> ".bold(), "Close".into()]),
            ];

            let mut bp_y = 8;

            if !self.breakpoints.is_empty() {
                bp_modal_lines.push(Line::from(vec![
                    "<K>   ".bold(),
                    "Clear all breakpoints".into(),
                ]));

                bp_y = 9;
            }

            frame.render_widget(
                Paragraph::new(Text::from(bp_modal_lines)).block(
                    Block::bordered()
                        .title(Line::from(" Add / Remove Breakpoint "))
                        .white()
                        .on_red(),
                ),
                Self::centered_rect(20, bp_y, frame.area()),
            );
        }

        if let Some(mem_view_start) = &self.mem_view_start_editing {
            let mut mv_modal_lines = vec![
                Line::from(vec![
                    "Address: ".into(),
                    format!(" {:20} ", mem_view_start.as_str())
                        .black()
                        .on_white()
                        .bold(),
                    " ↵".bold(),
                ]),
                "".into(),
                Line::from(vec!["<Esc> ".bold(), "Close".into()]),
            ];

            let mut mv_y = 8;

            if self.mem_view_start > 0 {
                mv_modal_lines.push(Line::from(vec!["<K>   ".bold(), "Reset".into()]));

                mv_y = 9;
            }

            frame.render_widget(
                Paragraph::new(Text::from(mv_modal_lines)).block(
                    Block::bordered()
                        .title(Line::from(" Set Memory View Start "))
                        .white()
                        .on_red(),
                ),
                Self::centered_rect(20, mv_y, frame.area()),
            );
        }

        if let Some(stack_view_start) = &self.stack_view_start_editing {
            let mut sv_modal_lines = vec![
                Line::from(vec![
                    "Address: ".into(),
                    format!(" {:20} ", stack_view_start.as_str())
                        .black()
                        .on_white()
                        .bold(),
                    " ↵".bold(),
                ]),
                "".into(),
                Line::from(vec!["<Esc> ".bold(), "Close".into()]),
            ];

            let mut sv_y = 8;

            if self.stack_view_start < CPU::STACK_MAX {
                sv_modal_lines.push(Line::from(vec!["<K>   ".bold(), "Reset".into()]));

                sv_y = 9;
            }

            frame.render_widget(
                Paragraph::new(Text::from(sv_modal_lines)).block(
                    Block::bordered()
                        .title(Line::from(" Set Stack View Start "))
                        .white()
                        .on_red(),
                ),
                Self::centered_rect(20, sv_y, frame.area()),
            );
        }
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

        Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
    }

    fn get_instruction_list(&self, skip: usize, take: usize) -> List {
        List::new(
            self.nano_core
                .instruction_log
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, l)| Self::get_instruction_line(i, l))
                .rev()
                .skip(skip)
                .take(take)
                .collect::<Vec<Line>>(),
        )
    }

    fn get_instruction_line(i: usize, l: String) -> Line<'static> {
        let (op, args, rest) = Self::get_instruction_parts(&l);

        let mut op_span = Span::raw(format!("{op:5}")).cyan();
        let mut args_span = Span::raw(format!(" {:<8}", args.trim()));
        let mut rest_span = Span::raw(format!(" │{}", rest.clone())).dim();

        if matches!(
            Op::from(op.as_str()),
            Op::JMP | Op::JZ | Op::JNZ | Op::CALL | Op::RET
        ) {
            op_span = op_span.magenta();
        }

        if Op::from(op.as_str()) == Op::HLT {
            op_span = op_span.red();
        }

        if rest.contains("(SKIP)") {
            op_span = op_span.dim();
            args_span = args_span.dim();
            rest_span = rest_span.yellow();
        }

        Line::from(vec![
            Span::raw(format!("{i:03}")).dim(),
            Span::raw(" "),
            op_span,
            args_span,
            rest_span,
        ])
    }

    fn get_instruction_parts(l: &str) -> (String, String, String) {
        let op = l.split_whitespace().next().unwrap_or("").to_owned();
        let args = l
            .strip_prefix(&op)
            .unwrap()
            .split('|')
            .next()
            .unwrap_or("")
            .to_owned();
        let rest = l
            .strip_prefix(&format!("{op}{args}"))
            .unwrap()
            .replace('|', "");

        (op, args, rest)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Char(' ') => self.next(),
                        KeyCode::Enter if self.none_editing() => self.running = !self.running,
                        KeyCode::Up => self.tick_rate.add_assign(Duration::from_millis(50)),
                        KeyCode::Down => {
                            self.tick_rate =
                                self.tick_rate.saturating_sub(Duration::from_millis(50))
                        }
                        KeyCode::Char('r') => self.reset(),
                        KeyCode::Char('b') => self.editing_breakpoint = Some("0x".into()),
                        KeyCode::Char('m') => {
                            self.mem_view_start_editing =
                                Some(format!("{:#04X}", self.mem_view_start))
                        }
                        KeyCode::Char('s') => {
                            self.stack_view_start_editing =
                                Some(format!("{:#04X}", self.stack_view_start))
                        }
                        key_code => {
                            Self::handle_edit_input(
                                key_code,
                                &mut self.editing_breakpoint,
                                |addr| {
                                    if self.breakpoints.contains(&addr) {
                                        self.breakpoints.retain(|x| *x != addr);
                                    } else {
                                        self.breakpoints.push(addr);
                                    }
                                },
                                0,
                            );
                            Self::handle_edit_input(
                                key_code,
                                &mut self.mem_view_start_editing,
                                |addr| self.mem_view_start = addr,
                                0,
                            );
                            Self::handle_edit_input(
                                key_code,
                                &mut self.stack_view_start_editing,
                                |addr| self.stack_view_start = addr,
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

    fn handle_edit_input<F>(
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

    fn none_editing(&self) -> bool {
        self.editing_breakpoint.is_none()
            && self.mem_view_start_editing.is_none()
            && self.stack_view_start_editing.is_none()
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next(&mut self) {
        self.nano_core.cycle();
    }

    fn run_full(&mut self) {
        if self.last_tick.elapsed() >= self.tick_rate {
            self.nano_core.cycle();
            self.last_tick = Instant::now();

            if self.nano_core.cpu.is_halted || self.breakpoints.contains(&self.nano_core.cpu.pc) {
                self.running = false;
            }
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.nano_core = NanoCore::new();
        self.nano_core.load_program(&self.program, 0x00);
    }

    fn parse_addr(s: &str) -> u8 {
        let addr = s.strip_prefix("0x").unwrap_or("0");

        if let Ok(addr) = hex::decode(addr) {
            if let Some(&addr) = addr.first() {
                return addr;
            }
        }

        0
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let bin = std::env::args().nth(1).expect("Missing filename.");

    let bytes = if bin.ends_with(".nca") {
        let asm = fs::read_to_string(&bin).unwrap();

        let mut c = Assembler::default();
        c.assemble(&asm);

        c.program
    } else {
        fs::read(&bin).unwrap()
    };

    let mut app = App {
        exit: false,

        nano_core: NanoCore::new(),
        filename: bin,
        program: vec![],

        running: false,
        tick_rate: Duration::from_millis(100),
        last_tick: Instant::now(),

        breakpoints: vec![],
        editing_breakpoint: None,

        mem_view_start: 0,
        mem_view_start_editing: None,

        stack_view_start: CPU::STACK_MAX,
        stack_view_start_editing: None,
    };

    let app = app.run(&mut terminal, &bytes);

    ratatui::restore();

    app
}
