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

#[derive(Debug)]
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

    pub fn init_from_sequence(input_sequence:&[u16]) -> Self {
        VirtualMachine{
            memory : Vec::from_iter(input_sequence.iter().map(|x|*x)),
            registers : [0;8],
            stack : Vec::<u16>::new(),
            program_counter : 0
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
                        self.stack.push(l);
                    }
                    ParsedValue::Register(r) => {
                        self.stack.push(self.registers[r as usize]);
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
                            self.registers[r as usize] = val;
                        } else {
                            return Err(RuntimeError::ErrStackEmpty);
                        }
                    },
                    ParsedValue::Literal(v) => return Err(RuntimeError::ErrUnknownOperand(v)),
                    ParsedValue::Error(v) => return Err(RuntimeError::ErrUnknownOperand(v))
                }
            },
            Operation::Eq => todo!(),
            Operation::Gt => todo!(),
            Operation::Jmp => todo!(),
            Operation::Jt => todo!(),
            Operation::Jf => todo!(),
            Operation::Add => {
                if let ParsedValue::Register(a) = operands[0] {
                    let b = self.dereference(&operands[1]);
                    let c = self.dereference(&operands[2]);
                    self.registers[a as usize] = (b + c) & 0x7FFF;
                } else {
                    return Err(RuntimeError::ErrRegisterExpected);
                }
            },
            Operation::Mult => todo!(),
            Operation::Mod => todo!(),
            Operation::And => todo!(),
            Operation::Or => todo!(),
            Operation::Not => todo!(),
            Operation::Rmem => todo!(),
            Operation::Wmem => todo!(),
            Operation::Call => todo!(),
            Operation::Ret => todo!(),
            Operation::Out => {
                let to_print:char = char::from_u32(self.dereference(&operands[0])as u32).unwrap_or('�');
                print!("{to_print}");
            },
            Operation::In => todo!(),
            Operation::Noop => (),
            Operation::Error(_) => return Err(RuntimeError::ErrUnknownOperation(self.memory[old_count])),
        };
        Ok(current_instruction)
    }

}

impl Display for VirtualMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        write!(f,"[PC:{}; R:{:?}; mem {} stack {:?}]",self.program_counter,self.registers,self.memory.len(),self.stack)
    }
}
