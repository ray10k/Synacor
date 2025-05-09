use crossterm::event::{Event, KeyEventKind, KeyCode};
use ratatui::symbols::border;
use ratatui::widgets::{self, Block, Widget, Clear, Borders};
use ratatui::prelude::{Rect, Buffer};

use crate::ui::VirtualMachineUI;

pub enum InputDone {
    /// Input handled, but the object can't be disposed yet.
    Keep,
    /// Input handled, and the object is all done.
    Discard
}

pub trait InputHandler {
    fn handle_input(&mut self, event:Event, parent:&mut VirtualMachineUI) -> InputDone;
    fn render(&self, area: Rect, buf: &mut Buffer);
}

struct InputField<'a> {
    buffer: String,
    printables: &'a str,
    title: &'a str,
    max_len:u16,
    when_done: Box<dyn Fn(&str)>
}

impl <'a> InputField<'a> {
    pub fn new(title:&'a str, printables:&'a str, max_len:u16, when_done:Box<dyn Fn(&str)>) -> Self {
        return InputField { buffer: String::new(), printables:printables, title:title, max_len:max_len.min(80), when_done: when_done}
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

impl <'a> InputHandler for InputField<'a> {
    fn handle_input(&mut self, event:Event, parent:&mut VirtualMachineUI) -> InputDone{
        if let Event::Key(key_event) = event { // The type of event is "something from the keyboard,"
            if let KeyEventKind::Release = key_event.kind { // more specifically "a key was released,"
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
                        (self.when_done)(&self.buffer[..]);
                        return InputDone::Discard;
                    },
                    KeyCode::Esc => { //and handle escape.
                        todo!("Inform the VMUI that the menu is needed!");
                    }
                    _ => {} //ignore all other keys.
                }
            }
        }
        InputDone::Keep
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Widget::render(self,area,buf)
    }
}