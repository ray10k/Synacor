use std::io::{BufReader,Read};
use std::fs::File;
use std::convert::From;
use std::fmt::{Display,Result as fmtResult};
use itertools::Itertools;

pub struct VirtualMachine {
    memory:Vec<u16>,
    registers:[u16;8],
    stack:Vec<u16>,
    program_counter:usize
}

pub enum ParsedValue {
    Literal(u16),
    Register(u16),
    Error(u16)
}

impl From<u16> for ParsedValue{
    fn from(value: u16) -> Self {
        match value {
            0..=32767 => ParsedValue::Literal(value),
            32768..=32775 => ParsedValue::Register(value - 32768),
            _ => ParsedValue::Error(value)
        }
    }
}

pub enum Operation {
    Halt,
    Set,
    Push,
    Pop,
    Eq,
    Gt,
    Jmp,
    Jt,
    Jf,
    Add,
    Mult,
    Mod,
    And,
    Or,
    Not,
    Rmem,
    Wmem,
    Call,
    Ret,
    Out,
    In,
    Noop,
    Error(u16)
}

impl From<u16> for Operation {
    fn from(value: u16) -> Self {
        match value {
            0 => Operation::Halt,
            1 => Operation::Set,
            2 => Operation::Push,
            3 => Operation::Pop,
            4 => Operation::Eq,
            5 => Operation::Gt,
            6 => Operation::Jmp,
            7 => Operation::Jt,
            8 => Operation::Jf,
            9 => Operation::Add,
            10=> Operation::Mult,
            11=> Operation::Mod,
            12=> Operation::And,
            13=> Operation::Or,
            14=> Operation::Not,
            15=> Operation::Rmem,
            16=> Operation::Wmem,
            17=> Operation::Call,
            18=> Operation::Ret,
            19=> Operation::Out,
            20=> Operation::In,
            21=> Operation::Noop,
            _ => Operation::Error(value)
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        write!(f,"{}", match *self{
            Operation::Halt => "HALT",
            Operation::Set => "SET ",
            Operation::Push => "PUSH",
            Operation::Pop => "POP ",
            Operation::Eq => "EQ  ",
            Operation::Gt => "GT  ",
            Operation::Jmp => "JMP ",
            Operation::Jt => "JT  ",
            Operation::Jf => "JF  ",
            Operation::Add => "ADD ",
            Operation::Mult => "MULT",
            Operation::Mod => "MOD ",
            Operation::And => "AND ",
            Operation::Or => "OR  ",
            Operation::Not => "NOT ",
            Operation::Rmem => "RMEM",
            Operation::Wmem => "WMEM",
            Operation::Call => "CALL",
            Operation::Ret => "RET ",
            Operation::Out => "OUT ",
            Operation::In => "IN  ",
            Operation::Noop => "NOOP",
            Operation::Error(_) => "!?!?",
        })
    }
}

impl Operation {
    pub fn operands(&self) -> u16 {
        match *self {
            Self::Halt | Self::Ret | Self::Noop => 0,
            Self::Push | Self::Pop | Self::Jmp | Self::Call | Self::Out | Self::In => 1,
            Self::Set | Self::Jt | Self::Jf | Self::Not | Self::Rmem |Self::Wmem => 2,
            Self::Eq | Self::Gt | Self::Add | Self::Mult | Self::Mod | Self::And | Self::Or => 3,
            Operation::Error(_) => 0xffff,
        }
    }
}

impl VirtualMachine {
    pub fn init_from_file(file_path:&str) -> Result<Self,std::io::Error> {
        let source_file = File::open(file_path)?;
        let buffer = BufReader::new(source_file);
        let data_buffer:Vec<u16> = buffer.bytes()
        .into_iter()
        .map(|x|x.unwrap_or(0))
        .tuples::<(u8,u8)>()
        .map(|(low,hi)|{
            let mut retval:u16 = hi as u16;
            retval <<= 8;
            retval |= low as u16;
            retval
        })
        .collect();
        Ok(VirtualMachine{
            memory : data_buffer,
            registers : [0;8],
            stack : Vec::<u16>::new(),
            program_counter : 0
        })
    }

    pub fn summary(&self) -> () {
        println!("Program with {} dbytes in memory, {} items on stack.",self.memory.len(),self.stack.len());
    }
}
