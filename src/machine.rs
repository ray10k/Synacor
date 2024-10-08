use std::io::{BufReader,Read,BufWriter,Write,Result as io_result};
use std::fs::File;
use std::convert::From;
use std::fmt::{Display,Result as fmtResult};
use itertools::Itertools;
use crate::interface::{VmInterface,RuntimeState,RegisterState,ProgramStep};
use crate::instruction::{Operation,ParsedValue};

pub struct VirtualMachine {
    memory:Vec<u16>,
    registers:[u16;8],
    stack:Vec<usize>,
    program_counter:usize,
    input_buffer:Vec<u16>,
}

pub struct VirtualMachineStep<'a> {
    machine:&'a mut VirtualMachine,
}

#[derive(Debug)]
pub enum RuntimeError {
    ErrFinished,
    ErrUnknownOperation(u16),
    ErrUnknownOperand(u16),
    ErrRegisterExpected,
    ErrInputEmpty,
    ErrStackEmpty,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        let message = match self {
            RuntimeError::ErrFinished => String::from("Program already finished."),
            RuntimeError::ErrUnknownOperation(x) => format!("Unknown operation with opcode {x:x}."),
            RuntimeError::ErrUnknownOperand(x) => format!("Unknown operand with value {x:x}."),
            RuntimeError::ErrRegisterExpected => String::from("Expected a register, got a literal value."),
            RuntimeError::ErrStackEmpty => String::from("POP instruction executed with empty stack."),
            RuntimeError::ErrInputEmpty => String::from("IN instruction executed while input buffer was empty.")
        };
        write!(f,"{message}" )
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
            stack : Vec::<usize>::new(),
            program_counter : 0,
            input_buffer : Vec::with_capacity(32)
        })
    }

    pub fn init_from_sequence(input_sequence:&[u16]) -> Self {
        VirtualMachine{
            memory : Vec::from_iter(input_sequence.iter().map(|x|*x)),
            registers : [0;8],
            stack : Vec::<usize>::new(),
            program_counter : 0,
            input_buffer : Vec::with_capacity(32)
        }
    }

    fn dereference(&self,val:&ParsedValue) -> u16 {
        match val {
            ParsedValue::Literal(x) => *x,
            ParsedValue::Register(r) => self.registers[*r as usize].clone(),
            ParsedValue::Error(e) => e & 0x7FFF,
        }
    }

    pub fn operation(&mut self) -> Result<(Operation,Vec<ParsedValue>,Option<char>),RuntimeError> {
        let mut to_print = None;
        //fetch
        let current_instruction = Operation::from(self.memory[self.program_counter]);
        let old_count = self.program_counter;
        //decode
        let argcount = current_instruction.operands() as usize;
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
                    ParsedValue::Error(e) => {
                        return Err(RuntimeError::ErrUnknownOperand(e));
                    }
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
                if self.memory.len() < a as usize {
                    self.memory.resize((a+1) as usize, 0);
                }
                self.memory[a as usize] = b;
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
                let print_char:char = char::from_u32(self.dereference(&operands[0])as u32).unwrap_or('�');
                to_print = Some(print_char);
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
                if let Some(ch) = self.input_buffer.pop() {
                    self.registers[register_number] = ch;
                } else {
                    self.program_counter = old_count; //Stall the program if the buffer is empty.
                    return Err(RuntimeError::ErrInputEmpty);
                }
            },
            Operation::Noop => (),
            Operation::Error(_) => return Err(RuntimeError::ErrUnknownOperation(self.memory[old_count])),
        };
        Ok((current_instruction,operands,to_print))
    }

    pub fn register_snapshot(&self) -> RegisterState {
        RegisterState { 
            registers: self.registers.clone(), 
            stack_depth: self.stack.len(), 
            program_counter: (self.program_counter & 0xffff) as u16
        }
    }

    pub fn run_program(&mut self, output:&mut impl VmInterface) {
        use RuntimeState::*;
        let mut run_state = Pause;
        let mut delay:usize = 0;
        loop {
            run_state = output.read_state(run_state == Pause)
                .unwrap_or(run_state);

            let reg_state = self.register_snapshot();

            match self.operation() {
                Ok((inst,operands,to_print)) => {
                    // Set up the "representation" of the executed instruction; a string giving
                    // a human-readable version.
                    let mut repr = format!("{inst}");
                    for pv in operands {
                        repr.push_str(&format!(" {pv}")[..]);
                    }
                    let _ = output.write_step(
                        ProgramStep::step(
                            reg_state, 
                            repr));
                    if let Some(to_print) = to_print {
                        let _ = output.write_output(to_print);
                    }
                },
                Err(RuntimeError::ErrInputEmpty) => {
                    let new_input = output.read_input(); //Note that this is a blocking operation.
                    self.input_buffer.extend(
                        new_input.chars() //take the characters of the string,
                        .filter(|ch| ch.is_ascii())// Keep the ones that are ASCII characters,
                        .map(|ch| (ch as u64 &0x7f) as u16)// Turn the characters into 16-bit values (since that's what the VM works with,)
                        .rev()//and finally reverse the string, so that the first character is at the top of the 'stack'.
                    );
                },
                Err(RuntimeError::ErrFinished) => {
                    let _ = output.write_step(ProgramStep::step(
                        reg_state,
                        "HALT".into()
                    ));
                    run_state = RuntimeState::Terminate;
                }
                Err(e) => {
                    output.runtime_err(format!("{e}"));
                },
            }

            match run_state {
                // Default state; just keep going.
                Run => (), 
                // Immediately go to the next iteration.
                Pause => continue, 
                // Perform one step, then pause.
                SingleStep | RunForSteps(1) => {run_state = Pause; continue}, 
                // Subtract one step from the remaining count.
                RunForSteps(steps) => run_state = RunForSteps(steps-1), 
                // Check if the address is part of the instruction about to be executed; pause after if it is.
                RunUntilAddress(addr) => {
                    let inst_start = (self.program_counter &0xffff) as u16;
                    let inst_end = inst_start + (Operation::from(self.memory[self.program_counter]).operands());
                    if inst_start == addr ||  (inst_start < addr && addr <= inst_end) {
                        run_state = Pause;
                    }
                },
                SetCommandDelay(new_delay,pause_after) => {
                    delay = new_delay;
                    if pause_after {
                        run_state = RuntimeState::Pause;
                    } else {
                        run_state = RuntimeState::Run;
                    }
                },
                // quit immediately.
                Terminate => break,
            }

            if delay > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay.try_into().expect("Invalid delay duration.")));
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
                let wordcount = 1 + value.operands() as usize;
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
    type Item = (Operation,Vec<ParsedValue>,Option<char>);

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
