use std::{
    io::{self, stdout, Stdout}, 
    panic::{take_hook,set_hook}, 
    time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::Frame;
use ratatui::widgets::{block::*,*};
use crossterm::{execute, terminal::*};
use circular_buffer::CircularBuffer;

use crate::interface::{UiInterface,ProgramStep,RegisterState,RuntimeState};

const TERMINAL_WIDTH:usize = 100;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn start_ui() -> io::Result<Tui> {
    setup_panic_hook();
    execute!(stdout(),EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn stop_ui() -> io::Result<()> {
    execute!(stdout(),LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn setup_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = stop_ui();
        original_hook(panic_info);
    }));
}

/// Data needed to display the current state of the VM.
#[derive(Debug,Default)]
pub struct MainUiState {
    /// Recorded recently executed instructions.
    prog_states:Box<CircularBuffer<1024,ProgramStep>>,
    /// Text that has been displayed via the `OUT` opcode.
    terminal_text:Vec<String>,
    /// Current state of the display.
    ui_mode:UiMode,
    /// Holding buffer for user input, prior to sending.
    input_buffer:String,
    /// To be removed.
    exit:bool
}

/// Current state of the UI.
#[derive(Debug,Default,PartialEq)]
enum UiMode {
    /// Displaying VM data as it arrives.
    #[default]
    Normal,
    /// Displaying a text-input field.
    WaitingForInput,
    WaitingForAddress,
    WaitingForCount,
    /// Input has been confirmed and is ready for use.
    InputReady,
    AddressReady,
    CountReady,
    /// VM operations suspended, waiting for command selection.
    Command,
    /// VM operations suspended.
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
            input_buffer: String::new(),
            exit: false 
        }
    }


    pub fn main_loop(&mut self, terminal:&mut Tui, input:&mut impl UiInterface) -> io::Result<()> {
        while !self.exit {
            let latest_steps = input.read_steps();
            self.prog_states.extend(latest_steps);
            if let Some(line) = input.read_output() {
                self.prep_string_input(line);
            }
            
            if input.is_finished() && self.ui_mode == UiMode::Normal {
                self.ui_mode = UiMode::Paused;
            } else if input.need_input() && self.ui_mode == UiMode::Normal{
                self.ui_mode = UiMode::WaitingForInput;
                self.input_buffer = String::with_capacity(32);
            }

            match self.ui_mode {
                UiMode::InputReady => {
                    let terminal_text = format!("> {}\n",&self.input_buffer[..]);
                    self.terminal_text.push(terminal_text);
                    input.write_input(&self.input_buffer)?;
                    self.ui_mode = UiMode::Normal;
                },
                UiMode::AddressReady => {
                    if let Ok(address) = u16::from_str_radix(&self.input_buffer[..], 16){
                        input.write_state(RuntimeState::RunUntilAddress(address)).expect("Could not send address to VM");
                    }
                    self.ui_mode = UiMode::Normal;
                },
                UiMode::CountReady => {
                    if let Ok(count) = self.input_buffer.parse::<usize>(){
                        input.write_state(RuntimeState::RunForSteps(count)).expect("Could not send step count to VM");
                    }
                    self.ui_mode = UiMode::Normal;
                },
                _ => ()
            }

            match self.handle_input() {
                Ok(Some(x)) => {
                    input.write_state(x)?;
                }
                Ok(None) => (),
                Err(e) => return Err(e),
            }
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
            .take((mid_layout[1].height - 2) as usize) // -2 to allow room for the borders around the list.
            .rev()
            .map(|state| {
                let inst_line = format!("{:04x}:{}",state.registers.program_counter,&state.instruction[..]);
                Line::from(inst_line)
            })
            .collect();

        let terminal_lines:Vec<Line> = self.terminal_text.iter()
            .rev()
            .take((mid_layout[0].height - 2) as usize) // See above.
            .rev()
            .map(|text| Line::from(&text[..]))
            .collect();

        frame.render_widget(&current_state.registers, root_layout[0]);
        frame.render_widget(Paragraph::new(terminal_lines).block(Block::default().title("Terminal").borders(Borders::ALL).border_set(border::THICK)),mid_layout[0]);
        frame.render_widget(Paragraph::new(instruction_lines).block(Block::default().title("Instructions").borders(Borders::ALL).border_set(border::THICK)), mid_layout[1]);
        frame.render_widget(self, root_layout[2]);
    }

    fn handle_input(&mut self) -> io::Result<Option<RuntimeState>> {
        if let Ok(true) = event::poll(POLL_TIME) {
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
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Enter => {
                                    self.input_buffer.push('\u{0a}'); //Manually add the line-feed character at the end.
                                    self.ui_mode = UiMode::InputReady;
                                },
                                KeyCode::Char(letter) => {
                                    self.input_buffer.push(letter);
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
                                    self.input_buffer = String::with_capacity(5)},
                                KeyCode::Char('n') => {self.ui_mode = UiMode::WaitingForCount;
                                    self.input_buffer = String::with_capacity(6)},
                                KeyCode::Char('r') => {return Ok(Some(RuntimeState::Run))},
                                KeyCode::Esc => {self.ui_mode = UiMode::Normal;},
                                _ => {}//By default, ignore all unknown keypresses.
                            }
                        }
                    }
                    UiMode::WaitingForAddress => {
                        if key.kind == KeyEventKind::Press {
                            if let KeyCode::Char(ch) = key.code {
                                if ch.is_digit(16) { //hexadecimal address entry!
                                    self.input_buffer.push(ch);
                                }
                            } else if let KeyCode::Enter = key.code {
                                if self.input_buffer.len() > 0 {
                                    self.ui_mode = UiMode::AddressReady;
                                } else {
                                    self.ui_mode = UiMode::Normal;
                                }
                            }
                        }
                    }
                    UiMode::WaitingForCount => {
                        if key.kind == KeyEventKind::Press {
                            if let KeyCode::Char(ch) = key.code {
                                if ch.is_digit(10) {
                                    self.input_buffer.push(ch);
                                }
                            } else if let KeyCode::Enter = key.code {
                                if self.input_buffer.len() > 0 {
                                    self.ui_mode = UiMode::CountReady;
                                } else {
                                    self.ui_mode = UiMode::Normal;
                                }
                            }
                        }
                    }
                    UiMode::InputReady | 
                    UiMode::AddressReady |
                    UiMode::CountReady |
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


    ///
    /// Write a new string to the main output window.
    /// If the string contains one or more line-breaks (0x0A), new lines will be generated.
    fn prep_string_input(&mut self, src:String) {
        if src.len() == 0 {
            return
        }
        if self.terminal_text.len() == 0 {
            self.terminal_text.push(String::with_capacity(50));
        }
        let mut top_line = self.terminal_text.last_mut().expect("Should be impossible, just pushed a blank string.");
        
        for cr in src.chars() {
            match cr {
                '\u{000A}' => {
                    self.terminal_text.push(String::with_capacity(50));
                    top_line = self.terminal_text.last_mut().expect("should be impossible, just pushed a new string.");
                },
                any => {
                    top_line.push(any);
                    if top_line.len() >= TERMINAL_WIDTH {
                        self.terminal_text.push(String::with_capacity(50));
                        top_line = self.terminal_text.last_mut().expect("This should be unreachable.");
                    }
                }
            }
        }
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
            let block_title:Title;
            let mut block_content:Line = Line::raw("");
            match self.ui_mode {
                UiMode::Normal => {
                    //Show instructions.
                    block_title = Title::from(Line::from(vec![
                        "press ".into(),
                        "esc".bold().blue(),
                        " to enter Command Mode".into()
                    ]));
                },
                UiMode::WaitingForInput |
                UiMode::WaitingForCount |
                UiMode::WaitingForAddress=> {
                    //Show input field.
                    let buff = &self.input_buffer[..];
                    block_title = Title::from(Line::from(vec![
                        "> ".into(),
                        buff.green(),
                        "█".white()
                    ]));
                },
                UiMode::InputReady |
                UiMode::CountReady |
                UiMode::AddressReady => {
                    //Show 'please stand by' message, until input is sent.
                    block_title = Title::from("Sending input, stand by.");
                }
                UiMode::Command => {
                    //Show command options.
                    block_title = Title::from("Command mode");
                    block_content = Line::from(vec![
                        "(".white(),
                        "esc".blue().on_white(),
                        ") exit command mode|".white(),
                        "R".blue().on_white(),
                        "un in normal mode,".white(),
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
                }
                UiMode::Paused => {
                    block_title = Title::from("Execution paused");
                },

            }
            Paragraph::new(block_content)
                .block(Block::default()
                    .title(block_title)
                    .borders(Borders::ALL)
                    .border_set(border::THICK))
            .render(area, buf);
    }
}
