use std::{
    io::{self, stdout, Stdout}, panic::{set_hook, take_hook}, time::Duration};

use crossterm::event::{self, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::Frame;
use ratatui::widgets::{block::*,*};
use crossterm::{execute, terminal::*};
use circular_buffer::CircularBuffer;

use crate::{interface::{ProgramStep, RegisterState, UiInterface}, ui_components::{BaseHandler, InputField, PopupMenu, WrappedHandlers}};
use crate::ui_components::InputDestination;

const TERMINAL_WIDTH:usize = 100;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn start_ui() -> io::Result<Tui> {
    setup_panic_hook();
    let mut output_line = stdout();
    execute!(output_line,EnterAlternateScreen)?;
    execute!(output_line,PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES))?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(output_line))
}

pub fn stop_ui() -> io::Result<()> {
    let mut output_line = stdout();
    execute!(output_line,PopKeyboardEnhancementFlags)?;
    execute!(output_line,LeaveAlternateScreen)?;
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
pub struct MainUiState<'a, T> where 
    T:UiInterface {
    /// Recorded recently executed instructions.
    prog_states:Box<CircularBuffer<1024,ProgramStep>>,
    /// Text that has been displayed via the `OUT` opcode.
    terminal_text:Vec<String>,
    /// Communication channel with the VM.
    vm_channel:T,
    /// Layered input widgets, over the top of the main UI.
    input_layers:Vec<WrappedHandlers<'a>>,
    /// Signals when the program should quit.
    exit:bool
}

const DEFAULT_STATE:ProgramStep = ProgramStep::const_default();
const POLL_TIME:Duration = Duration::from_millis(100);
const INPUT_PRINTABLES:&str = " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
const DECIMAL_PRINTABLES:&str = "0123456789";
const HEXADECIMAL_PRINTABLES:&str = "0123456789abcdefABCDEF";

#[derive(Debug)]
enum UiMutation {
    /// Do not change the UI stack.
    None,
    /// Remove the nth item from the UI stack.
    Delete(usize),
    /// Insert the popup menu top of the stack.
    Popup
}

impl <'a,T:UiInterface+'a> MainUiState<'a,T> {
    pub fn new(vm_channel:T) -> Self{
        Self { 
            prog_states: CircularBuffer::<1024,ProgramStep>::boxed(), 
            terminal_text: Vec::new(),
            vm_channel: vm_channel,
            input_layers: Vec::with_capacity(5),
            exit: false,
        }
    }

    pub fn main_loop(&mut self, terminal:&mut Tui) -> io::Result<()> {
        self.input_layers.push(WrappedHandlers::BaseHandler(BaseHandler::default()));
        self.input_layers.push(WrappedHandlers::PopupMenu(PopupMenu::default()));

        while !self.exit {
            self.prog_states.extend(self.vm_channel.read_steps());
            
            if let Some(line) = self.vm_channel.read_output() {
                self.prep_string_input(line);
            }
            
            if self.vm_channel.need_input() && self.input_layers.len() == 1{
                let in_field = WrappedHandlers::InputField(
                    InputField::new(
                        "Input", 
                        INPUT_PRINTABLES, 
                        256,
                        InputDestination::Input 
                        )
                    );
                self.input_layers.push(in_field);
            }

            let input_available = event::poll(POLL_TIME).unwrap_or(false);
            if input_available {
                let event = event::read().expect("Could not decode waiting event.");
                let mut to_discard = UiMutation::None;
                for (index,input_handler) in self.input_layers.iter_mut().enumerate().rev() { //iterate in *rev*erse! Last added is first to run!
                    let rm = input_handler.handle_input(event.clone());
                    match rm {
                        crate::ui_components::InputDone::Keep => break,
                        crate::ui_components::InputDone::Discard => {
                            to_discard = UiMutation::Delete(index);
                            break},
                        crate::ui_components::InputDone::Pass => continue,
                        crate::ui_components::InputDone::Quit => {
                            self.exit = true;
                            break},
                        crate::ui_components::InputDone::Input(input_destination, value) => {
                            match input_destination {
                                InputDestination::Input => self.vm_channel.write_input(&value).expect("Could not write input to VM"),
                                InputDestination::ProgramCounter => todo!("Implement setting the program counter here."),
                            }
                            to_discard = UiMutation::Delete(index);
                            break},
                        crate::ui_components::InputDone::Popup => {
                            to_discard = UiMutation::Popup;
                            break},
                    }
                };
                match to_discard {
                    UiMutation::None => (),
                    UiMutation::Delete(index) => {self.input_layers.remove(index);()},
                    UiMutation::Popup => {self.input_layers.push(WrappedHandlers::PopupMenu(PopupMenu::default()))},
                }
            }

            terminal.draw(|frame| self.render_frame(frame))?;
        }
        Ok(())
    }

    fn render_frame(&self, frame:&mut Frame){
        let root_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(4),Constraint::Fill(1)])
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

        for layer in self.input_layers.iter() {
            match layer {
                WrappedHandlers::BaseHandler(widget) => frame.render_widget(widget, frame.size()),
                WrappedHandlers::InputField(widget) => frame.render_widget(widget, frame.size()),
                WrappedHandlers::PopupMenu(widget) => frame.render_widget(widget, frame.size()),
            }
        }

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
