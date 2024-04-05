use std::env;
use std::io;

use crate::machine::VirtualMachine;


mod machine;
mod ui;

/*
fn main()->io::Result<()>{
    let mut term = ui::start_ui()?;
    let app = ui::MainUiState::default().main_loop(&mut term);
    ui::stop_ui()?;
    Ok(())
}*/
 

fn main() {
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
                
            }}
        },
        Err(_) => println!("Could not parse file!"),
    }
}

