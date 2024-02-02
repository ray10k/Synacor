use std::io::{BufReader,Read};
use std::fs::File;
use itertools::Itertools;

pub struct VirtualMachine {
    memory:Vec<u16>,
    r0:u16,
    r1:u16,
    r2:u16,
    r3:u16,
    r4:u16,
    r5:u16,
    r6:u16,
    r7:u16,
    stack:Vec<u16>
}

pub enum InitError {
    BinaryFileNotFoundError
}

impl VirtualMachine {
    pub fn init_from_file(file_path:&str) -> Result<Self,InitError> {
        let source_file = File::open(file_path);
        if let Err(_) = source_file {
            return Err(InitError::BinaryFileNotFoundError);
        }
        let source_file = source_file.unwrap();
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
            r0 : 0,
            r1 : 0,
            r2 : 0,
            r3 : 0,
            r4 : 0,
            r5 : 0,
            r6 : 0,
            r7 : 0,
            stack : Vec::<u16>::new()
        })
    }

    pub fn summary(&self) -> () {
        println!("Program with {} dbytes in memory, {} items on stack.",self.memory.len(),self.stack.len());
    }
}
