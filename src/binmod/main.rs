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
    data: Vec<u16>,
}

impl FromStr for Segment {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //Since a "segment" needs to follow the pattern address:data, check if that can work.
        if s.len() < 9 || s.chars().nth(5) != Some(':') {
            return Err(Error::new(ErrorKind::Other, "Malformed argument"));
        }
        let seg_address = &s[0..5];
        if !all(seg_address.chars(), |c| hex_num(c)) {
            return Err(Error::new(ErrorKind::Other,"Malformed address"));
        }
        let seg_data = &s[6..];
        if !all(seg_data.chars(), |c| hex_num(c)) || seg_data.len() % 4 != 0 {
            return Err(Error::new(ErrorKind::Other, "Malformed data segment."));
        }
        Ok(Self {
            start_address: seg_address.parse().unwrap(),
            data: seg_data
                .chars()
                .chunks(4)
                .into_iter()
                .map(|word| {
                    let word_str = String::from_iter(word);
                    word_str.parse().unwrap()
                })
                .collect(),
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
        value_parser = Segment::parse_arg
    )]
    segments: Vec<Segment>,
}

fn main() {
    println!("Hello world!");
}
