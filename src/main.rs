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
    let mut vm = VirtualMachine::init_from_file(&arguments[1][..]);
    match vm {
        Ok(mach) => {
            println!("File parsed OK.");
            mach.summary();
        },
        Err(_) => println!("Could not parse file!"),
    }
}


