use crossterm::event::{Event, KeyEventKind, KeyCode};

use crate::ui::VirtualMachineUI;

pub enum InputDone {
    /// Input handled, but the object can't be disposed yet.
    Keep,
    /// Input handled, and the object is all done.
    Discard
}

pub trait InputHandler {
    fn handle_input(&mut self, event:Event, parent:&mut VirtualMachineUI) -> InputDone;
}

struct InputField {
    buffer: String,
    printables: &'static str,
    when_done: Box<dyn Fn(&str)>
}

impl InputField {
    pub fn new(printables:&'static str, when_done:Box<dyn Fn(&str)>) -> Self {
        return InputField { buffer: String::new(), printables:printables, when_done: when_done}
    }
}

impl InputHandler for InputField {
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
}