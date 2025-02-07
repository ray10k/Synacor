mod startup;
mod machine;
mod ui;
mod interface;
mod thread_interface;
mod instruction;
mod static_analysis;
mod instruction_tracker;

use clap::Parser;
use std::{fs::File, io::prelude::*};
use itertools::Itertools;

use crate::machine::VirtualMachine;

#[derive(Parser,Debug)]
#[command(version, about)]
struct Args{
    #[arg()]
    binary_source:String,
    #[arg(help="Treat input as raw data.",long_help="Interpret the input as a sequence of 15-bit words, presented in little endian order hexadecimal notation.",short='r')]
    raw_input:bool,
    #[arg(help="Analyze input, and save result.",long_help="Instead of executing the provided input, analyze the data and save an assembly file at the provided location.",short='a',)]
    analyze:Option<String>,
}



fn main() {
    let args = Args::parse();
    println!("{args:?}");

    match args.analyze {
        Some(destination) => {
            let bytes:Vec<u16>;
            let original_name:&str;
            if args.raw_input {
                bytes =  sequence_decypher(&args.binary_source);
                original_name = "<raw input>";
            } else {
                //open the file, parse from bytes to words, put in vec.
                bytes = File::open(&args.binary_source).expect("Error opening file")
                .bytes()
                .into_iter()
                .map(|byte| byte.unwrap_or(0))
                .tuples::<(u8,u8)>()
                .map(|(low,high)| (low as u16) | (high as u16) << 8)
                .collect();
                if destination.contains(&['/','\\']) {
                    original_name = &args.binary_source[args.binary_source.find(&['/','\\']).unwrap()..];
                } else {
                    original_name = &args.binary_source[..];
                }
            }
            if let Err(e) = static_analysis::parse_program_and_save(&bytes, original_name, &destination[..]) {
                println!("Error in analysis: {e:?}");
            } else {
                println!("Analysis completed sucessfully.");
            }
        },
        None => {
            let vm:VirtualMachine;
            if args.raw_input {
                let byte_sequence:Vec<u16> = sequence_decypher(&args.binary_source);
                vm = VirtualMachine::init_from_sequence(&byte_sequence);
            } else {
                vm = VirtualMachine::init_from_file(&args.binary_source).expect("Could not parse given file");
            }
            startup::main_interface(vm).expect("Serious error during program runtime");
        },
    }
    /*
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
    
    startup::main_interface(vm).expect("Something went wrong running the program!");*/
}  

fn sequence_decypher(input:&str) -> Vec<u16> {
    let words = input.len()/4;
    (0..words).into_iter().map(|start|{
        let left = start * 4;
        let low_byte:u16 = u16::from_str_radix(&input[left..left+2], 16).expect("Malformed sequence input!");
        let high_byte:u16 = u16::from_str_radix(&input[left+2..left+4], 16).expect("Malformed sequence input!");
        (high_byte << 8) | low_byte
    }).collect()
}
