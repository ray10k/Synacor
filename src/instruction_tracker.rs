use std::fs::File;
use std::io::{BufWriter, Write};

use crate::instruction::{Operation, ParsedValue};
use crate::interface::RegisterState;

pub struct InstructionTracker{
    destination:BufWriter<File>
}

impl InstructionTracker {
    pub fn new(save_to:&str) -> Result<Self,std::io::Error> {
        let destination = File::create(save_to)?;
        Ok(Self{destination: BufWriter::new(destination)})
    }

    pub fn instruction(&mut self,pc:u16,operation:Operation,operands:&[u16],registers:RegisterState) -> Result<(),std::io::Error> {
        let op_type:char;
        let op_addr:u16; //The "memory address being targeted."

        match operation {
            Operation::Jmp => {op_type = 'J'; op_addr = operands[0]},
            Operation::Jf => {
                //Check if the jump will happen. Otherwise, break out.
                let src:ParsedValue = operands[0].into();
                match src {
                    ParsedValue::Literal(0) | ParsedValue::Register(_) => {
                        op_type = 'S'; op_addr = operands[1];
                    }
                    _ => {
                        return Ok(())
                    }
                }
            },
            Operation::Jt => {
                let src:ParsedValue = operands[0].into();
                match src {
                    ParsedValue::Literal(0) => {
                        return Ok(())
                    }
                    ParsedValue::Literal(_) | ParsedValue::Register(_) => {
                        op_type = 'S'; op_addr = operands[1];
                    }
                    _ => {
                        return Ok(())
                    }
                }
            },
            Operation::Rmem => {op_type = 'L'; op_addr = operands[1]}, //load
            Operation::Wmem => {op_type = 'S'; op_addr = operands[0]}, //store
            Operation::Call => {op_type = 'C'; op_addr = operands[0]},
            Operation::Ret =>  {op_type = 'R'; op_addr = 0x7fff} //Return address is on the stack, can't retrieve here.
            _ => {return Ok(())/* pass */}
        }
        let pv:ParsedValue = op_addr.into();
        let op_addr = match pv {
            ParsedValue::Literal(x) | ParsedValue::Error(x) => {x},
            ParsedValue::Register(r) => {registers.registers[r as usize]},
        };

        write!(&mut self.destination,"{op_type} {pc:0>4x} {op_addr:0>4x}")?;
        Ok(())
    }
}

impl Drop for InstructionTracker {
    fn drop(&mut self) {
        let _ = self.destination.flush();
    }
}
