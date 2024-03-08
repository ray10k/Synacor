use std::env;

use crate::machine::VirtualMachine;


mod machine;

fn main() {
    println!("Starting up...");
    let arguments:Vec<String> = env::args().collect();
    println!("{:?}",arguments);
    if arguments.len() != 2 {
        println!("Not enough arguments! Specify a path to the binary file!");
        std::process::exit(1);
    }
    let vm = VirtualMachine::init_from_file(&arguments[1][..]);
    match vm {
        Ok(mach) => {
            let mut mach = mach;
            println!("File parsed OK: {mach}");
            println!("Starting program execution.");
            for op in mach.run_program(false) {
                
            }
        },
        Err(_) => println!("Could not parse file!"),
    }

    
}


