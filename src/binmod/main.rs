use std::fs;
use std::str::FromStr;
use std::io::{Error,ErrorKind};

use clap::Parser;
use itertools::{all, Itertools};

fn hex_num(c: char) -> bool {
    "0123456789ABCDEF".contains(c.to_ascii_uppercase())
}

#[derive(Debug, Clone)]
struct Segment {
    start_address: u16,
    data: Vec<u8>,
}

impl FromStr for Segment {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //Since a "segment" needs to follow the pattern address:data, check if that can work.
        if s.len() < 9 || s.chars().nth(4) != Some(':') {
            return Err(Error::new(ErrorKind::Other, "Malformed argument"));
        }
        let seg_address = &s[0..=3];
        if !all(seg_address.chars(), |c| hex_num(c)) {
            return Err(Error::new(ErrorKind::Other,"Malformed address"));
        }
        let seg_data = &s[5..];
        if !all(seg_data.chars(), |c| hex_num(c)) || seg_data.len() % 4 != 0 {
            return Err(Error::new(ErrorKind::Other, "Malformed data segment."));
        }
        let parsed_data:Vec<u8> = (0..seg_data.len())
            .tuples::<(usize,usize)>()
            .map(|(left_index,right_index)| {
                u8::from_str_radix(&seg_data[left_index..=right_index],16).expect("Invalid number")
            }).tuples::<(u8,u8)>()
            .flat_map(|(first_byte,second_byte)|{
                [second_byte,first_byte].into_iter()
            })
            .collect();
        let parsed_address:u16 =  u16::from_str_radix(seg_address,16).unwrap();
        Ok(Self {
            start_address: parsed_address,
            data: parsed_data
        })
    }
}

impl Segment {
    fn parse_arg(argument:&str) -> Result<Self,Error> {
        Self::from_str(argument)
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg()]
    input_path: String,
    #[arg(
        help = "One pair of starting-address and data-to-write.",
        long_help = "Specify one chunk of data to overwrite in the source binary, in the format [starting-address]:[data]",
        value_parser = Segment::parse_arg,
        short = 's'
    )]
    segments: Vec<Segment>,
    #[arg(
        help = "Output file",
        long_help = "Destination path that the patched binary should be saved to. File will be overwritten if it already exists.",
        required = true,
        short = 'o'
    )]
    destination_path: String
}

fn main() {
    println!("Hello world!");
    let args = Args::parse();
    println!("{args:x?}");
    //Fetch the original file; crash if that's not an option.
    let mut file_data = fs::read(&args.input_path).expect("Could not access file!");
    //Apply each segment to the fetched data.
    for segment in args.segments.iter() {
        let start_index = (segment.start_address as usize) * 2;
        for index in 0..=segment.data.len() {
            file_data[start_index + index] = segment.data[index];
        }
    }
    //Save the result to the destination address.
    fs::write(&args.destination_path, file_data).expect("Could not save to the desination file!");
}
