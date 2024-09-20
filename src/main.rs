use std::env;
use std::thread;
use std::io;

use interface::ProgramStep;
use ui::MainUiState;

use crate::interface::*;
use crate::machine::VirtualMachine;
use crate::interface::{UiInterface,VmInterface};
use crate::thread_interface::make_interfaces;

mod machine;
mod ui;
mod interface;
mod thread_interface;

fn main()->io::Result<()>{
    let arguments:Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        println!("Not enough arguments! Specify a path to the binary file!");
        std::process::exit(1);
    }
    let file_path = &arguments[1][..];
    let mut term = ui::start_ui()?;
    let (mut ui_interface, vm_interface) = make_interfaces();
    //TODO: load the file path. Again.
    let mut virtual_machine = VirtualMachine::init_from_file(file_path).expect("Could not open the specified binary file.");
    //TODO: start this from its own thread.
    let mut user_interface = MainUiState::new();

    {
        thread::spawn( move || {
            //VM thread
            let mut vm_interface = vm_interface;
            virtual_machine.run_program(&mut vm_interface);
        });
        user_interface.main_loop(&mut term, &mut ui_interface)?;
    }

    ui::stop_ui()?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Default)]
struct TestUiInterface {
    step:u8,
    counter:usize
}

impl UiInterface for TestUiInterface {
    fn read_output(&mut self) -> Option<String> {
        if self.step == 0 {
            Some(String::from("Hello world!"))
        } else if self.counter % 3 == 0 {
            Some(String::from("Take a break!\n"))
        } else if self.counter % 7 == 0 {
            Some(String::from("In the\nmiddle of things."))
        } else {
            None
        }
    }

    fn read_steps(&mut self) -> Vec<interface::ProgramStep> {
        if self.need_input() {
            return Vec::new();
        }

        self.step += 1;
        if self.step > 15 {
            self.step = 0;
        }
        self.counter += 1;
        vec![ProgramStep{
            registers: RegisterState{
                registers:[1<<self.step as u16;8],
                stack_depth: self.counter,
                program_counter: (self.counter & 0xffff) as u16
            },
            instruction: format!("{:04x} -> HELO 1111 2222",self.counter & 0xff),
        };self.step as usize]
    }

    fn write_input(&mut self, _:&str) -> std::io::Result<()> {
        println!("Input");
        self.counter += 1;
        std::io::Result::Ok(())
    }

    fn write_state(&mut self, _:interface::RuntimeState) -> std::io::Result<()> {
        std::io::Result::Ok(())
    }

    fn need_input(&self) -> bool {
        self.counter % 30 == 0
    }

    fn is_finished(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn channel_test() {
        let mut test = TestUiInterface::default();
        assert_eq!(Some(String::from("Hello world!")),test.read_output());
    }
}

/*fn main() {
    println!("Starting up...");
    let arguments:Vec<String> = env::args().collect();
    println!("{:?}",arguments);
    if arguments.len() < 2 {
        println!("Not enough arguments! Specify a path to the binary file!");
        std::process::exit(1);
    }
    let vm = VirtualMachine::init_from_file(&arguments[1][..]);

    match vm {
        Ok(mach) => {
            let mut mach = mach;
            println!("File parsed OK: {mach}");
            println!("Starting program execution.");
            if arguments.len() > 2 {
                //run in listing mode
                mach.dump_memory_to_file(&arguments[2]).expect("Error writing listing out.");
                
            } else {
            for op in mach.run_program(false) {
                
            }};
            println!("Final state of VM: {mach}");
        },
        Err(_) => println!("Could not parse file!"),
    }
}*/

