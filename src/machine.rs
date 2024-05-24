use std::io::{BufReader,Read,BufWriter,Write,stdin,Result as io_result};
use std::fs::File;
use std::convert::From;
use std::fmt::{Display,Result as fmtResult};
use itertools::Itertools;
use crate::interface::{VmInterface,RuntimeState,RegisterState,ProgramStep};

pub struct VirtualMachine {
    memory:Vec<u16>,
    registers:[u16;8],
    stack:Vec<usize>,
    program_counter:usize,
    input_buffer:String,
}

pub struct VirtualMachineStep<'a> {
    machine:&'a mut VirtualMachine,
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

#[derive(Debug,PartialEq)]
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
    pub fn operands(&self) -> usize {
        match *self {
            Self::Halt | Self::Ret | Self::Noop => 0,
            Self::Push | Self::Pop | Self::Jmp | Self::Call | Self::Out | Self::In => 1,
            Self::Set | Self::Jt | Self::Jf | Self::Not | Self::Rmem |Self::Wmem => 2,
            Self::Eq | Self::Gt | Self::Add | Self::Mult | Self::Mod | Self::And | Self::Or => 3,
            Operation::Error(_) => 0xffff,
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    ErrFinished,
    ErrUnknownOperation(u16),
    ErrUnknownOperand(u16),
    ErrRegisterExpected,
    ErrStackEmpty
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        let message = match self {
            RuntimeError::ErrFinished => String::from("Program already finished."),
            RuntimeError::ErrUnknownOperation(x) => format!("Unknown operation with opcode {x:x}."),
            RuntimeError::ErrUnknownOperand(x) => format!("Unknown operand with value {x:x}."),
            RuntimeError::ErrRegisterExpected => String::from("Expected a register, got a literal value."),
            RuntimeError::ErrStackEmpty => String::from("POP instruction executed with empty stack."),
        };
        write!(f,"{message}" )
    }
}

#[derive(Default,Debug)]
pub struct ProgramState {
    pub registers:[u16;8],
    pub program_counter:usize,
    pub stack_depth:usize
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
            stack : Vec::<usize>::new(),
            program_counter : 0,
            input_buffer : String::new()
        })
    }

    pub fn init_from_sequence(input_sequence:&[u16]) -> Self {
        VirtualMachine{
            memory : Vec::from_iter(input_sequence.iter().map(|x|*x)),
            registers : [0;8],
            stack : Vec::<usize>::new(),
            program_counter : 0,
            input_buffer : String::new()
        }
    }

    fn dereference(&self,val:&ParsedValue) -> u16 {
        match val {
            ParsedValue::Literal(x) => *x,
            ParsedValue::Register(r) => self.registers[*r as usize].clone(),
            ParsedValue::Error(e) => e & 0x7FFF,
        }
    }

    pub fn operation(&mut self) -> Result<Operation,RuntimeError> {
        //fetch
        let current_instruction = Operation::from(self.memory[self.program_counter]);
        let old_count = self.program_counter;
        //decode
        let argcount = current_instruction.operands();
        let mut operands:Vec<ParsedValue> = Vec::with_capacity(argcount);
        for x in old_count+1..old_count+1+argcount {
            let pv = ParsedValue::from(self.memory[x]);
            if let ParsedValue::Error(x) = pv {
                return Err(RuntimeError::ErrUnknownOperand(x));
            } else {
                operands.push(pv);
            }
        }
        //Update program counter here, so that jumping instructions can still overwrite it.
        self.program_counter += argcount+1;
        //execute, store
        match current_instruction {
            Operation::Halt => {
                self.program_counter = old_count;
                return Err(RuntimeError::ErrFinished)
            },
            Operation::Set => {
                match operands[0] {
                    ParsedValue::Literal(_) => return Err(RuntimeError::ErrRegisterExpected),
                    ParsedValue::Register(r) => {
                        let val_b = self.dereference(&operands[1]);
                        self.registers[r as usize] = val_b;
                    },
                    ParsedValue::Error(_) => panic!("Should never reach!"),
                }
            },
            Operation::Push => {
                match operands[0] {
                    ParsedValue::Literal(l) => {
                        self.stack.push(l.into());
                    }
                    ParsedValue::Register(r) => {
                        self.stack.push(self.registers[r as usize].into());
                    }
                    ParsedValue::Error(e) => {
                        return Err(RuntimeError::ErrUnknownOperand(e));
                    }
                }
            },
            Operation::Pop => {
                match operands[0] {
                    ParsedValue::Register(r) => {
                        let popped = self.stack.pop();
                        if let Some(val) = popped {
                            self.registers[r as usize] = (val & 0x7fff) as u16;
                        } else {
                            return Err(RuntimeError::ErrStackEmpty);
                        }
                    },
                    ParsedValue::Literal(v) => return Err(RuntimeError::ErrUnknownOperand(v)),
                    ParsedValue::Error(v) => return Err(RuntimeError::ErrUnknownOperand(v))
                }
            },
            Operation::Eq => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    if b == c {
                        self.registers[a as usize] = 1;
                    } else {
                        self.registers[a as usize] = 0;
                    }
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Gt => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    if b > c {
                        self.registers[a as usize] = 1;
                    } else {
                        self.registers[a as usize] = 0;
                    }
                }
            },
            Operation::Jmp => {
                self.program_counter = self.dereference(&operands[0]) as usize;
            },
            Operation::Jt => {
                if self.dereference(&operands[0]) != 0 {
                    self.program_counter = self.dereference(&operands[1]) as usize;
                }
            },
            Operation::Jf => {
                if self.dereference(&operands[0]) == 0 {
                    self.program_counter = self.dereference(&operands[1]) as usize;
                }
            },
            Operation::Add => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    self.registers[a as usize] = (b + c) & 0x7FFF;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Mult => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]) as u32;
                    let c = self.dereference(&operands[2]) as u32;
                    self.registers[a as usize] = (b * c) as u16 & 0x7FFF;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Mod => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    self.registers[a as usize] = b % c;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::And => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    self.registers[a as usize] = b & c;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Or => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    self.registers[a as usize] = b | c;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Not => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    self.registers[a as usize] = b ^ 0x7FFF;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Rmem => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let val:u16;
                    if self.memory.len() as u16 >= b {
                        val = self.memory[b as usize];
                    } else {
                        val = 0;
                    }
                    self.registers[a as usize] = val;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Wmem => {
                let a = self.dereference(&operands[0]);
                let b = self.dereference(&operands[1]);
                if self.memory.len() < b as usize {
                    self.memory.resize((b+1) as usize, 0);
                }
                self.memory[b as usize] = a;
            },
            Operation::Call => {
                if let ParsedValue::Error(a) = operands[0] {
                    return Err(RuntimeError::ErrUnknownOperand(a));
                } else {
                    self.stack.push(self.program_counter);
                    self.program_counter = self.dereference(&operands[0]).into();
                }
            },
            Operation::Ret => {
                if self.stack.len() > 0 {
                    self.program_counter = self.stack.pop().expect("Stack empty!");
                } else {
                    return Err(RuntimeError::ErrStackEmpty);
                }
            },
            Operation::Out => {
                let to_print:char = char::from_u32(self.dereference(&operands[0])as u32).unwrap_or('�');
                print!("{to_print}");
            },
            Operation::In => {
                //Assumption: Since <a> is only a single target, treat this as "read until newline, then put the last character
                //before the newline into <a>."
                if let ParsedValue::Literal(_) = operands[0] {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
                let register_number = {
                    if let ParsedValue::Register(x) = operands[0] {
                        x as usize
                    } else {
                        0xff as usize
                    }
                };
                if self.input_buffer.len() > 0 {
                    let last_char = self.input_buffer.trim() //Take the user's input, and trim off the leading/trailing whitespace or linebreaks.
                    .chars() //Get an iterator over the characters in the resulting substring.
                    .rev() //Reverse the character-iterator. This way, the last characters are now at the start.
                    .next() //Take one character from the string; should be the last before the linebreak.
                    .unwrap_or('\0'); //null-byte to indicate nothing was entered, otherwise the character itself.
                    if last_char.is_ascii() {
                        self.registers[register_number] = last_char as u16;
                    } else {
                        //No idea how to handle the case where a non-ascii character got input, so... wild guessing time.
                        self.registers[register_number] = 0x7fff
                    }
                    //The buffer has been used, clear it.
                    self.input_buffer.clear();
                } else {
                    //Stall the program if there is no input in the buffer.
                    self.program_counter = old_count;
                }
            },
            Operation::Noop => (),
            Operation::Error(_) => return Err(RuntimeError::ErrUnknownOperation(self.memory[old_count])),
        };
        Ok(current_instruction)
    }

    pub fn run_program(&mut self, output:&mut impl VmInterface) {
        let mut latest = Operation::Noop;
        loop {
            if latest == Operation::In {
                //Pull input from interface, and put in buffer.
                self.input_buffer.push_str(&output.read_input()[..]);
            } 

            match self.operation() {
                Ok(inst) => {
                    latest = inst;
                },
                Err(e) => {
                    output.runtime_err(format!("{e}"));
                },
            }
        }
    }

    pub fn dump_memory_to_file(&self, save_location:&str) -> io_result<()>{
        //Set up the output writer.
        let destination_file = File::create(save_location)?;
        let mut out_writer = BufWriter::new(destination_file);
        //Will need to have some control over the iterator, both for operands and for raw data.
        let mut memory_iterator = self.memory.iter().enumerate();


        while let Some((index,current_word)) = memory_iterator.next() {
            let value = Operation::from(*current_word);
            if let Operation::Error(raw) = value {
                //Must be some raw value. Print both the hex value, and (if possible) the ASCII characters.
                let low = (raw & 0xff) as u8;
                let hi = ((raw>>8) &0xff) as u8;
                writeln!(
                    &mut out_writer,
                    "{:04X}: <{raw:04X}      > {}{}",
                    index&0xffff,
                    {
                        if low.is_ascii() && !low.is_ascii_control() {
                            low as char
                        } else {
                            ' '
                        }
                    },
                    {
                        if hi.is_ascii() && !hi.is_ascii_control() {
                            hi as char
                        } else {
                            ' '
                        }
                    }
                )?;
            } else {
                let wordcount = 1 + value.operands();
                let raw_bytes = &self.memory[index..=(wordcount+index)];
                let mut ascii_chars:String = String::with_capacity(8);

                for raw_word in raw_bytes {
                    let low = (raw_word & 0xff) as u8;
                    let high = ((raw_word >> 8) & 0xff) as u8;
                    ascii_chars.push(
                        if low.is_ascii_alphanumeric() {
                            low as char
                        } else {
                            ' '//'�'
                        }
                    );
                    ascii_chars.push(
                        if high.is_ascii_alphanumeric() {
                            high as char
                        } else {
                            ' '//'�'
                        }
                    )
                }

                while ascii_chars.len() < 8 {
                    ascii_chars.push(' ');
                }

                write!(&mut out_writer,"{:04X}: <{ascii_chars}> {value} ",index&0xffff)?;
                for _ in 0..value.operands() {
                    let (_,operand) = memory_iterator.next().expect("Unexpected end of file!");
                    let operand = ParsedValue::from(*operand);
                    match operand {
                        ParsedValue::Literal(v) => write!(&mut out_writer,"{v:04X}  ")?,
                        ParsedValue::Register(r) => write!(&mut out_writer,"REG{r:1}  ")?,
                        ParsedValue::Error(e) => write!(&mut out_writer,"!{e:04X} ")?,
                    }
                }
                writeln!(&mut out_writer,"")?;
            }
        }

        Ok(())
    }
}

impl Display for VirtualMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        write!(f,"[PC:{}; R:{:?}; mem {} stack {:?}]",self.program_counter,self.registers,self.memory.len(),self.stack)
    }
}

impl<'a> Iterator for VirtualMachineStep<'a> {
    type Item = Operation;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.machine.operation();
        match res {
            Ok(x) => Some(x),
            Err(x) => {
                println!("{:?}",x);
                None
            },
        }
    }
}
