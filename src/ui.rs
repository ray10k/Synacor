use std::{
    io::{self, stdout, Stdout},
    time::Duration};

use crossterm::event::{KeyCode, KeyEventKind,self,Event};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::Frame;
use ratatui::widgets::{block::*,*};
use crossterm::{execute, terminal::*};
use circular_buffer::CircularBuffer;

use crate::interface::{UiInterface,ProgramStep,RegisterState,RuntimeState};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn start_ui() -> io::Result<Tui> {
    execute!(stdout(),EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn stop_ui() -> io::Result<()> {
    execute!(stdout(),LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Debug,Default)]
pub struct MainUiState {
    prog_states:Box<CircularBuffer<1024,ProgramStep>>,
    terminal_text:Vec<String>,
    ui_mode:UiMode,
    input_buffer:Option<String>,
    exit:bool
}

#[derive(Debug,Default,PartialEq)]
enum UiMode {
    #[default]
    Normal,
    WaitingForInput,
    WaitingForAddress,
    WaitingForCount,
    InputReady,
    Command,
    Paused,
}

const DEFAULT_STATE:ProgramStep = ProgramStep::const_default();
const POLL_TIME:Duration = Duration::from_millis(100);

impl MainUiState {
    pub fn new() -> Self{
        Self { 
            prog_states: CircularBuffer::<1024,ProgramStep>::boxed(), 
            terminal_text: Vec::new(),
            ui_mode: UiMode::Normal,
            input_buffer: None,
            exit: false 
        }
    }

    pub fn main_loop(&mut self, terminal:&mut Tui, input:&mut impl UiInterface) -> io::Result<()> {
        while !self.exit {

            if input.need_input() && self.ui_mode == UiMode::Normal{
                self.ui_mode = UiMode::WaitingForInput;
                self.input_buffer = Some(String::with_capacity(32));
            } else if self.ui_mode == UiMode::Normal {
                let latest_steps = input.get_steps();
                self.prog_states.extend(latest_steps);
                if let Some(term) = input.get_output() {
                    if let Some(existing) = self.terminal_text.last_mut() {
                        if existing.ends_with('\n') {
                            self.terminal_text.push(term);
                        } else {
                            existing.push_str(&term);
                            if existing.contains('\n') {
                                let splitpoint = existing.rfind('\n').expect("impossible");
                                let remainder = existing.split_off(splitpoint+1);
                                self.terminal_text.push(remainder);
                            }
                        }
                    } else {
                        self.terminal_text.push(term);
                    }
                }
            }
            
            if self.ui_mode == UiMode::InputReady {
                if let Some(to_send) = &self.input_buffer {
                    let terminal_text = format!("> {to_send}\n");
                    self.terminal_text.push(terminal_text);
                    input.send_input(to_send)?;

                } else {
                    panic!("Could not send; buffer is missing.");
                }
                self.input_buffer = None;
                self.ui_mode = UiMode::Normal;
            }

            self.handle_input()?;
            terminal.draw(|frame| self.render_frame(frame))?;
        }
        Ok(())
    }

    fn render_frame(&self, frame:&mut Frame){
        let root_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(4),Constraint::Fill(1),Constraint::Length(3)])
            .split(frame.size());
        let mid_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(47),Constraint::Length(28)])
            .split(root_layout[1]);
        let def = DEFAULT_STATE;
        let current_state = self.prog_states.back().unwrap_or(&def);

        let instruction_lines:Vec<Line> = self.prog_states.iter()
            .rev()
            .take(mid_layout[1].height as usize)
            .rev()
            .map(|state| Line::from(&state.instruction[..]))
            .collect();

        let terminal_lines:Vec<Line> = self.terminal_text.iter()
            .rev()
            .take(mid_layout[0].height as usize)
            .rev()
            .map(|text| Line::from(&text[..]))
            .collect();

        frame.render_widget(&current_state.registers, root_layout[0]);
        frame.render_widget(Paragraph::new(terminal_lines).block(Block::default().title("Terminal").borders(Borders::ALL).border_set(border::THICK)),mid_layout[0]);
        frame.render_widget(Paragraph::new(instruction_lines).block(Block::default().title("Instructions").borders(Borders::ALL).border_set(border::THICK)), mid_layout[1]);
        frame.render_widget(self, root_layout[2]);
    }

    fn handle_input(&mut self) -> io::Result<Option<RuntimeState>> {
        if event::poll(POLL_TIME)? {
            if let Event::Key(key) = event::read()? {
                match self.ui_mode {
                    UiMode::Normal => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Esc => {self.ui_mode = UiMode::Command},
                                _ => {}
                            }
                        }
                    },
                    UiMode::WaitingForInput => {
                        //Handle specifics for writing input.
                        let buffer = self.input_buffer.as_mut().expect("Buffer was not initialized.");
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Enter => {
                                    self.ui_mode = UiMode::InputReady;
                                },
                                KeyCode::Char(letter) => {
                                    buffer.push(letter);
                                },
                                _ => ()
                            }
                        }
                    },
                    UiMode::Command => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => {self.exit = true;},
                                KeyCode::Char('s') => {return Ok(Some(RuntimeState::SingleStep))},
                                KeyCode::Char('a') => {self.ui_mode = UiMode::WaitingForAddress;
                                    self.input_buffer = Option::Some(String::with_capacity(5))},
                                KeyCode::Char('n') => {self.ui_mode = UiMode::WaitingForCount;
                                    self.input_buffer = Option::Some(String::with_capacity(6))},
                                KeyCode::Esc => {self.ui_mode = UiMode::Normal;}
                                _ => {}//By default, ignore all unknown keypresses.
                            }
                        }
                    }
                    UiMode::WaitingForAddress => {
                        todo!("Handle address input.");
                    }
                    UiMode::WaitingForCount => {
                        if key.kind == KeyEventKind::Press {
                            if let KeyCode::Char(ch) = key.code {
                                if ch.is_digit(10) {
                                    self.input_buffer
                                        .as_mut()
                                        .expect("Buffer not initialized.")
                                        .push(ch);
                                }
                            } else if let KeyCode::Enter = key.code {
                                todo!("finalize count input.");
                            }
                        }
                    }
                    UiMode::InputReady | 
                    UiMode::Paused => {
                        if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                            self.ui_mode = UiMode::Command;
                        }
                    },
                }
                
            }
        }
        
        Ok(None)
    }
}

impl Widget for &RegisterState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
        let text = vec![
            format!("R0:{:04x} R1:{:04x} R2:{:04x} R3:{:04x}  PC:{}",self.registers[0],self.registers[1],self.registers[2],self.registers[3],self.program_counter).into(),
            format!("R4:{:04x} R5:{:04x} R6:{:04x} R7:{:04x}  ST:{}",self.registers[4],self.registers[5],self.registers[6],self.registers[7],self.stack_depth).into()
        ];
        let par = Paragraph::new(text).block(Block::default().title("registers").borders(Borders::ALL).border_set(border::THICK));
        par.render(area,buf)
    }
}

impl Widget for &MainUiState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
            //Set up the layout.
            match self.ui_mode {
                UiMode::Normal => {
                    //Show instructions.
                    let footer = Title::from(Line::from(vec![
                        "press ".into(),
                        "space".bold().blue(),
                        " to start/stop the VM".into()
                    ]));
                    let _block = Block::default()
                        .title(footer.alignment(Alignment::Center).position(Position::Bottom))
                        .borders(Borders::ALL)
                        .border_set(border::THICK)
                        .render(area,buf);
                },
                UiMode::WaitingForInput => {
                    //Show input field.
                    let buff = self.input_buffer.as_ref().expect("Message buffer missing.").clone();
                    let footer = Title::from(Line::from(vec![
                        "> ".into(),
                        buff.green(),
                        "█".white()
                    ]));
                    let _block = Block::default()
                        .title(footer.alignment(Alignment::Left).position(Position::Top))
                        .borders(Borders::ALL)
                        .border_set(border::THICK)
                        .render(area, buf);
                },
                UiMode::InputReady => {
                    //Show 'please stand by' message, until input is sent.
                    let footer = Title::from("Sending input, stand by.");
                    let _block = Block::default()
                        .title(footer.alignment(Alignment::Center).position(Position::Top))
                        .borders(Borders::ALL)
                        .border_set(border::THICK)
                        .render(area, buf);
                }
                UiMode::Command => {
                    //Show command options.
                    let title = Title::from("Command mode");
                    let body = Line::from(vec![
                        "(".white(),
                        "esc".blue().on_white(),
                        ") exit command mode|".white(),
                        "S".blue().on_white(),
                        "ingle step|".white(),
                        "Run until ".white(),
                        "a".blue().on_white(),
                        "ddress|".white(),
                        "Run for ".white(),
                        "N".blue().on_white(),
                        " steps|".white(),
                        "Q".blue().on_white(),
                        "uit".white()
                    ]);
                    Paragraph::new(body)
                        .block(Block::default()
                            .title(title)
                            .borders(Borders::ALL)
                            .border_set(border::THICK))
                        .render(area, buf);
                }
                UiMode::Paused => {
                    let title = Title::from("Execution paused");
                    let _block = Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_set(border::THICK)
                        .render(area, buf);
                },
                UiMode::WaitingForAddress => {
                    todo!("Implement waiting for address.");
                },
                UiMode::WaitingForCount => {
                    todo!("Implement waiting for count.");
                }

            }
    }
}
