use std::env;
use std::io;
use std::rc::Rc;

use interface::ProgramStep;

use crate::interface::RegisterState;
use crate::machine::VirtualMachine;
use crate::interface::UiInterface;

mod machine;
mod ui;
mod interface;


fn main()->io::Result<()>{
    let mut term = ui::start_ui()?;
    let mut interface = TestUiInterface::default();
    let _app = ui::MainUiState::default().main_loop(&mut term,&mut interface);
    ui::stop_ui()?;
    Ok(())
}

#[derive(Default)]
struct TestUiInterface {
    step:u8,
    counter:usize
}

impl UiInterface for TestUiInterface {
    fn get_output(&mut self) -> Option<String> {
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

    fn get_steps(&mut self) -> Vec<interface::ProgramStep> {
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

    fn send_input(&mut self, _:&str) -> std::io::Result<()> {
        std::io::Result::Ok(())
    }

    fn send_state(&mut self, _:interface::RuntimeState) -> std::io::Result<()> {
        std::io::Result::Ok(())
    }

    fn need_input(&self) -> bool {
        false
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

