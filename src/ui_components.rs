use std::fmt::Debug;
use std::marker::PhantomData;

use crossterm::event::{Event, KeyEventKind, KeyCode};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};
use ratatui::prelude::{Rect, Buffer};

#[derive(Debug)]
pub enum InputDone {
    /// Input handled, but the object can't be disposed yet.
    Keep,
    /// Input handled, and the object is all done.
    Discard,
    /// Input not handled, pass it along to somewhere else.
    Pass,
    /// The UI can start shutting down.
    Quit,
    /// Open the popup menu (and implicitly, keep this object)
    Popup,
    /// There is data ready (and implicitly, discard this object)
    Input(InputDestination,String)
}

pub trait InputHandler{
    fn handle_input(&mut self, event:Event) -> InputDone;
}

#[derive(Clone, Copy, Debug)]
pub enum InputDestination {
    /// VM input
    Input,
    /// VM program counter
    ProgramCounter,
    //TODO: add more things that may need to receive input here.
}

/// Input field UI element, with a callback for when the user presses `enter`.
pub struct InputField<'a> {
    buffer: String,
    printables: &'a str,
    title: &'a str,
    max_len:u16,
    destination:InputDestination,
}

impl <'a> InputField<'a> {
    pub fn new(title:&'a str, printables:&'a str, max_len:u16, destination:InputDestination) -> Self {
        return InputField { buffer: String::new(), printables:printables, title:title, max_len:max_len.min(80), destination:destination}
    }
}

impl Widget for &InputField<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        // Rules for formatting: the title, if it can't fit, just gets cut off where Ratatui decides it should get cut off.
        // The input gets at most enough lines to fit its entire target length, and at least as many as the available height
        // minus 2 (for the border). Just kinda done playing nice :sweat_smile:
        // First, how wide will the box be? Should be enough to hold the title, ideally, as well as the text to be entered.
        let target_width = (self.max_len as usize).max(self.title.len()).min(area.width as usize) as u16 - 2;
        // Second: how tall? Will need at least 3 lines; top border (with title), text field, bottom border.
        let target_height = (2 + (self.max_len / target_width)).min(area.height);

        // next: figure out the origin point (top-left) of the render-area.
        let origin_x = (area.x) + (area.width - target_width / 2);
        let origin_y = (area.y) + (area.height - target_height / 2);
        let field_area = Rect::new(origin_x, origin_y, target_width, target_height);

        // Also next: Split up the buffered input into lines, so they can be displayed.

        Clear::default().render(field_area, buf);
        Block::default().title(self.title).borders(Borders::ALL).border_set(border::DOUBLE).render(field_area, buf);

    }
}

impl <'a> Debug for InputField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputField")
            .field("buffer", &self.buffer)
            .field("printables", &self.printables)
            .field("title", &self.title)
            .field("max_len", &self.max_len)
            .field("when_done", &"CALLBACK FUNCTION").finish()
    }
}

impl <'a> InputHandler for InputField<'a> {
    fn handle_input(&mut self, event:Event) -> InputDone{
        if let Event::Key(key_event) = event { // The type of event is "something from the keyboard,"
            match key_event.code {
                KeyCode::Char(c) => { //Handle a letter, number or other printable thing.
                    if self.printables.contains(c) {
                        self.buffer.push(c);
                    }
                },
                KeyCode::Backspace => { //handle backspace.
                    self.buffer.pop();
                },
                KeyCode::Enter => { //handle enter.
                    return InputDone::Input(self.destination,self.buffer.clone());
                },
                KeyCode::Esc => { //and handle escape.
                    todo!("Inform the VMUI that the menu is needed!");
                }
                _ => {} //ignore all other keys.
            }
        }
        InputDone::Keep
    }
}

#[derive(Debug,Default)]
enum MenuMode {
    #[default]
    /// Display a list of sub-menus
    Main,
    /// Display options for adjusting runtime speed, and limited execution.
    RunModes,
    /// Display options for adjusting VM state, such as register contents.
    VMState,
    /// Display file-related options, such as saving the current VM state.
    FileOptions,
}

/// Pop-up menu with options for manipulating the VM.
#[derive(Debug,Default)]
pub struct PopupMenu<'a>{
    menu_mode:MenuMode,
    phantom:PhantomData<&'a ()>,
}

const POPUP_SIZE:(u16,u16) = (40,20);
const MENU_NORMAL_STYLE:Style = Style::new().bg(Color::Green).fg(Color::White);
const MENU_HILIGHT_STYLE:Style = Style::new().bg(Color::LightRed).fg(Color::Black).underline_color(Color::Gray).add_modifier(Modifier::UNDERLINED);

impl Widget for &PopupMenu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
            let x = (area.width - POPUP_SIZE.0) / 2;
            let y = (area.height - POPUP_SIZE.1) / 2;

            let sub_frame = Rect::new(x, y, POPUP_SIZE.0, POPUP_SIZE.1);

            let (title,lines_vec) = match self.menu_mode {
                MenuMode::Main => (Line::from("Main menu"),vec![
                    build_menu_line("Change &Runtime options.", MENU_NORMAL_STYLE, MENU_HILIGHT_STYLE),
                    build_menu_line("Change VM &State.", MENU_NORMAL_STYLE, MENU_HILIGHT_STYLE),
                    build_menu_line("&File operations.", MENU_NORMAL_STYLE, MENU_HILIGHT_STYLE),
                    "".into(),
                    build_menu_line("&Quit", MENU_NORMAL_STYLE, MENU_HILIGHT_STYLE),
                    build_menu_line("(&E&S&C) to close the menu.", MENU_NORMAL_STYLE, MENU_HILIGHT_STYLE)
                ]),
                MenuMode::RunModes => todo!(),
                MenuMode::VMState => todo!(),
                MenuMode::FileOptions => todo!(),
            };


            ratatui::widgets::Clear::default().render(sub_frame, buf);

            
            Paragraph::new(lines_vec)
                .block(Block::default()
                    .style(MENU_NORMAL_STYLE)
                    .title(title)
                    .borders(Borders::RIGHT | Borders::BOTTOM)
                    .border_set(border::PLAIN)
                    .title_alignment(Alignment::Center))
                .alignment(Alignment::Center)
                .render(sub_frame,buf);
    }
}

const AMPERSAND_SIZE:usize = '&'.len_utf8();

///Apply the `normal_style` to `text`, except for the character immediately following an ampersand; those characters
/// have the `highlight_style` applied instead.
fn build_menu_line(text:&str, normal_style:Style, highlight_style:Style) -> Line {
    //Walk over the string, one character at a time. Remember, Rust uses utf-8 encoded strings, can't just walk byte-by-byte.
    let mark_locations:Vec<usize> = text.chars()
        //Track where in the string the character is, and start yielding (position,character) pairs.
        .enumerate()
        //Keep the positions of all ampersands, drop all other positions.
        .filter_map(|(position,letter)| {
            if letter == '&' {Some(position)} else {None}
        })
        //Turn the iterator into a vector, by collecting it. The type of `mark_locations` tells Rust to use a Vector in this case.
        .collect();
    if mark_locations.len() == 0 { //If there are no ampersands, just style the entire line with the normal style.
        return Line::from(Span::styled(text,normal_style))
    }

    let mut part_start:usize = 0;
    let mut line_parts:Vec<Span> = Vec::with_capacity(1+(2*mark_locations.len()));

    for amp_position in mark_locations.into_iter() {
        if part_start != amp_position {
            line_parts.push(Span::styled(&text[part_start..amp_position],normal_style));
        }
        let highlight_position = amp_position+AMPERSAND_SIZE;
        let highlight_end = highlight_position + 1; //will break on any character larger than one byte. TODO: figure out where the next character begins.
        line_parts.push(Span::styled(&text[amp_position+AMPERSAND_SIZE..highlight_end],highlight_style));
        part_start = highlight_end;
    }

    if part_start != text.len() {
        line_parts.push(Span::styled(&text[part_start..],normal_style));
    }

    Line::from(line_parts)
}

impl <'a> InputHandler for PopupMenu<'a> {
    fn handle_input(&mut self, event:Event) -> InputDone{
        if let Event::Key(key_event) = event { // The type of event is "something from the keyboard,"
            match self.menu_mode {
                MenuMode::Main => {
                    match key_event.code {
                        KeyCode::Char('r') => {self.menu_mode = MenuMode::RunModes},
                        KeyCode::Char('s') => {self.menu_mode = MenuMode::VMState},
                        KeyCode::Char('f') => {self.menu_mode = MenuMode::FileOptions},
                        KeyCode::Char('q') => {return InputDone::Quit},
                        KeyCode::Esc => {return InputDone::Discard},
                        _ => ()
                    }
                },
                MenuMode::RunModes => todo!(),
                MenuMode::VMState => todo!(),
                MenuMode::FileOptions => todo!(),
            }
        }
        InputDone::Keep
    }
}

#[derive(Debug,Default)]
pub struct BaseHandler<'a>{
    phantom:PhantomData<&'a ()>
}

impl <'a> InputHandler for BaseHandler<'a> {
    fn handle_input(&mut self, event:Event) -> InputDone {
        //Wait for esc, and tell the main UI to show the menu when that happens.
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Esc => {return InputDone::Popup}
                _ => ()
            }
        }
        return InputDone::Keep;
    }
}

impl Widget for &BaseHandler<'_> {
    fn render(self, _: Rect, __: &mut Buffer)
    where
        Self: Sized {}
}

#[derive(Debug)]
pub enum WrappedHandlers<'a> {
    BaseHandler(BaseHandler<'a>),
    InputField(InputField<'a>),
    PopupMenu(PopupMenu<'a>),
}

impl <'a> WrappedHandlers<'a> {
    pub fn handle_input(&mut self, event:Event) -> InputDone {
        match self {
            WrappedHandlers::BaseHandler(base_handler) => base_handler.handle_input(event),
            WrappedHandlers::InputField(input_field) => input_field.handle_input(event),
            WrappedHandlers::PopupMenu(popup_menu) => popup_menu.handle_input(event),
        }
    }
}