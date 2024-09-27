mod startup;
mod machine;
mod ui;
mod interface;
mod thread_interface;

use clap::Parser;
use std::io::stdin;

use crate::machine::VirtualMachine;

#[derive(Parser,Debug)]
#[command(version, about)]
struct Args{
    file_name:Option<String>,

    #[arg(short)]
    sequence:Option<String>
}

fn main() {
    let args = Args::parse();
    print!("{args:?}");
    let vm = if let Some(path) = args.file_name {
        VirtualMachine::init_from_file(&path).expect("Error loading binary file.")
    } else if let Some(seq) = args.sequence {
        if seq.len() % 4 != 0 {
            panic!("Sequence should be a multiple of 16 bits!");
        }; 
        let parsed = sequence_decypher(&seq);
        VirtualMachine::init_from_sequence(&parsed[..])
    } else {
        let binary_path = get_file_path();
        VirtualMachine::init_from_file(binary_path.trim()).expect("Error loading binary file.")
    };
    
    startup::main_interface(vm).expect("Something went wrong running the program!");
}  

fn get_file_path() -> String {
    println!("No file path was specified at the command line!\nPlease enter a path to a binary file to run.");
    print!("> ");
    let mut buffer:String = String::new();
    stdin().read_line(&mut buffer).expect("Something went wrong, restart the program!");
    println!("\nRunning file {buffer}.");
    buffer
}

fn sequence_decypher(input:&str) -> Vec<u16> {
    let words = input.len()/4;
    (0..words).into_iter().map(|start|{
        let left = start * 4;
        u16::from_str_radix(&input[left..left+4], 16).expect("Malformed sequence input!")
    }).collect()
}
