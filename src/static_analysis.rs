use std::{collections::HashSet, ffi::OsStr, fmt::Display};

use crate::instruction::*;

/// A block of executed code.
#[derive(PartialEq, Eq, PartialOrd, Ord,Debug,Clone,Copy)]
struct ExecBlock{
    /// Address of the first instruction in the block.
    start:u16,
    /// Address immediately after the last executed word in this block.
    end:u16,
}

impl ExecBlock {
    fn new(start:u16,end:u16) -> Self {
        Self { start: start, end: end }
    }
}

impl Display for ExecBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Block from {} to {}",self.start,self.end)
    }
}


/// What type of jump this is, depending on what conditions change where execution resumes.
enum JumpType {
    /// The jump will always happen.
    Fixed,
    /// The jump will always happen, and starts a subroutine.
    Call,
    /// The jump may not happen, depending on register state.
    Conditional,
    /// Either the jump itself or the operand may be overwritten and executed.
    Modified
}

/// A jump in execution; can be conditional.
struct Jump {
    /// Location of the instruction that causes the jump.
    from:u16,
    /// Location at the other side, if one is known.
    target:Option<u16>,
    /// Type of jump.
    jump_type:JumpType,
}

enum AnalysisError {
    GenericError,
    FileAccessError,
}

fn parse_program_and_save(program:&[u16],save_path:&OsStr) -> Result<(),AnalysisError> {
    //Step 1: setup.
    let mut read_addresses:HashSet<u16> = HashSet::new();
    let mut write_addresses:HashSet<u16> = HashSet::new();
    let mut exec_blocks:Vec<ExecBlock> = Vec::new();
    let mut jump_targets:Vec<u16> = Vec::with_capacity(8);
    let mut jump_info:Vec<Jump> = Vec::new();
    jump_targets.push(0);

    //Step 2: simulate.
    //Grab a 'waiting' jump target to begin.
    'executable: while let Some(block_start) = jump_targets.pop() {
        //Check if there is a block that starts at this point already.
        for block in exec_blocks.iter() {
            if block.start == block_start {
                continue 'executable
            }
        }
        let mut program_counter = block_start as usize;
        loop {
            let instruction = Operation::from(program[program_counter]);
            let operands = instruction.operands();
            match instruction {
                //option 1: end-of-block with no further considerations needed.
                Operation::Halt | Operation::Ret | Operation::Error(_)=> {
                    //Save the current block, keep going.
                    let end = program_counter as u16 + operands + 1;
                    exec_blocks.push(ExecBlock::new(block_start, end));
                    continue 'executable;
                },
                //option 2: end-of-block via unconditional, un-resumable jump.
                Operation::Jmp => {
                    //Save the current block, try to add the jump target to the buffer.
                    let end = program_counter as u16 + operands + 1;
                    exec_blocks.push(ExecBlock::new(block_start, end));
                    let target = ParsedValue::from(program[program_counter + 1]);
                    if let ParsedValue::Literal(address) = target {
                        jump_targets.push(address);
                    }
                    continue 'executable;
                },
                //option 3: optional jump.
                Operation::Jf | Operation::Jt => {
                    //Try to add the jump target to the buffer, and continue.
                    // Note that the *second* operand holds the jump target.
                    let target = ParsedValue::from(program[program_counter+2]);
                    if let ParsedValue::Literal(address) = target {
                        jump_targets.push(address);
                    }
                },
                Operation::Call => {
                    //Try to add the jump target to the buffer, and continue.
                    let target = ParsedValue::from(program[program_counter+1]);
                    if let ParsedValue::Literal(address) = target {
                        jump_targets.push(address);
                    }
                },
                //option 4: memory read.
                Operation::Rmem => {
                    let target = ParsedValue::from(program[program_counter+1]);
                    if let ParsedValue::Literal(address) = target {
                        read_addresses.insert(address);
                    }
                },
                //option 5: memory write.
                Operation::Wmem => {
                    let target = ParsedValue::from(program[program_counter+1]);
                    if let ParsedValue::Literal(address) = target {
                        write_addresses.insert(address);
                    }
                },
                //option 6: anything else, IDC.
                _ => {}
            }
            //Continue on the next operation. Increment program counter, then skip over however
            // many operands the current operation has.
            program_counter += 1 + operands as usize;
        }
    }

    //Step 3: prepare to write out.

    Ok(())
}
/*
fn find_containing_block(blocks:&mut Vec<DataBlock>,address:u16) -> &mut DataBlock {
    blocks.sort();
    let mut retval = blocks.len();
    for (index,block) in blocks.iter().enumerate() {
        if block.end > address {
            retval = index;
            break;
        }
    }
    blocks.get_mut(retval).unwrap()
}*/