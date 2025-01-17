use std::{collections::HashSet, ffi::OsStr, fmt::Display, fs::File, io::Write};

use crate::instruction::*;
use itertools::Itertools;

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

    fn contains(&self,addr:usize) -> bool {
        self.start as usize >= addr && self.end as usize <= addr
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
    /// The jump will always happen, and returns from a subroutine.
    Return,
    /// The "jump" is a halt-instruction. Program execution stops here.
    Halt,
    /// The "jump" is a malformed instruction. Program execution errors out here.
    Error,
    /// The jump may not happen, depending on register state.
    Conditional
}

impl TryInto<JumpType> for Operation {
    type Error = ();

    fn try_into(self) -> Result<JumpType, <Operation as TryInto<JumpType>>::Error> {
        match self {
            Self::Jmp => Ok(JumpType::Fixed),
            Self::Jf | Self::Jt => Ok(JumpType::Conditional),
            Self::Call => Ok(JumpType::Call),
            Self::Ret => Ok(JumpType::Return),
            Self::Halt => Ok(JumpType::Halt),
            Self::Error(_) => Ok(JumpType::Error),
            _ => Err(())
        }
    }
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

struct JumpLabel {
    from:u16,
    target:u16
}

impl Jump {
    fn get_label(&self) -> Option<JumpLabel> {
        if let Some(target) = self.target {
            Some(JumpLabel{from:self.from,target:target})
        } else {
            None
        }
    }
}

enum AnalysisError {
    GenericError,
    FileAccessError,
    FileWriteError,
}

pub fn parse_program_and_save(program:&[u16],original_name:&str,save_path:&OsStr) -> Result<(),AnalysisError> {
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

                    jump_info.push(Jump { from: program_counter as u16, target: None, jump_type: instruction.try_into().unwrap() });
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
                        jump_info.push(Jump { from: program_counter as u16, target: Some(address), jump_type: JumpType::Fixed });
                    } else {
                        jump_info.push(Jump { from: program_counter as u16, target: None, jump_type: JumpType::Fixed });
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
                        jump_info.push(Jump { from: program_counter as u16, target: Some(address), jump_type: JumpType::Conditional });
                    } else {
                        jump_info.push(Jump { from: program_counter as u16, target: None, jump_type: JumpType::Conditional });
                    }
                },
                Operation::Call => {
                    //Try to add the jump target to the buffer, and continue.
                    let target = ParsedValue::from(program[program_counter+1]);
                    if let ParsedValue::Literal(address) = target {
                        jump_targets.push(address);
                        jump_info.push(Jump { from: program_counter as u16, target: Some(address), jump_type: JumpType::Call });
                    } else {
                        jump_info.push(Jump { from: program_counter as u16, target: None, jump_type: JumpType::Call });
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
    //For now, keep only the fixed-target jumps and discard anything that doesn't have a target address.
    let mut targeted_jumps:Vec<Jump> = jump_info.into_iter()
        .filter(|jump| jump.target.is_some())
        .collect();
    //sort based on destination address, so the data can be used to make labels.
    targeted_jumps.sort_by(|a,b|
        a.target
        .as_ref()
        .unwrap()
        .cmp(b.target
            .as_ref()
            .unwrap()));
    let known_labels:Vec<JumpLabel> = targeted_jumps.iter().filter_map(|jmp| jmp.get_label()).collect();
    //Deduplicate and combine the execution blocks, to identify non-executable data.
    //Sort in reverse.
    exec_blocks.sort_by(|a,b| b.start.cmp(&a.start));
    
    let exec_blocks:Vec<ExecBlock> = exec_blocks.into_iter().coalesce(|l,r| {
        if l.end < r.start {
            Err((l,r))
        } else if l.end >= r.end {
            Ok(l)
        } else {
            Ok(ExecBlock::new(l.start, r.end))
        }
    }).collect();

    let mut destination_file = File::create(save_path).or(Err(AnalysisError::FileAccessError))?;

    writeln!(&mut destination_file,"Data listing for file {original_name}").or(Err(AnalysisError::FileWriteError))?;
    writeln!(&mut destination_file,"Binary size: {} bytes ({} words)",program.len()*2,program.len()).or(Err(AnalysisError::FileWriteError))?;
    writeln!(&mut destination_file,"\n\n").or(Err(AnalysisError::FileWriteError))?;

    let mut exec_blocks = exec_blocks.iter();
    let mut current_block = exec_blocks.next().expect("No block of execution at the start of the program");
    let mut current_address:usize = 0;

    while current_address < program.len() {
        //First: determine if this is executable instructions, or data according to the current block.
        if current_block.contains(current_address) {
            //instruction-block. Read one instruction, check for labels, write out.
            let label = known_labels.iter().filter(|label|label.target as usize == current_address).collect::<Vec<_>>();
            for l in label.into_iter() {
                writeln!(&mut destination_file,"     :l{:0>4x}",l.from).or(Err(AnalysisError::FileWriteError))?;
            }
            let instr = Operation::from(program[current_address]);

            write!(&mut destination_file,"{:0>4x} {instr}",current_address&0xffff).or(Err(AnalysisError::FileWriteError))?;

            for op in 0..instr.operands() {
                let op_address = current_address + (op as usize);
                let parsed_op = ParsedValue::from(program[op_address]);
                write!(&mut destination_file," {parsed_op}").or(Err(AnalysisError::FileWriteError))?;
            }
            writeln!(&mut destination_file,"").or(Err(AnalysisError::FileWriteError))?;

            current_address += (instr.operands() as usize) + 1;
        } else {
            //data-block. Fetch the next one, then write word-after-word of this block
            // until the start of the next instruction-block.
            let another_block = exec_blocks.next();
            let stop_point;
            if let Some(blk) = another_block {
                current_block = blk;
                stop_point = blk.end as usize;
            } else {
                //Write-out until end of file.
                stop_point = program.len();
            }
            
            //I want to print out the data-block in the following format:
            //<start-address of the line>: <8 words of data in hexadecimal> | <same 8 words as 16 ascii characters>
            //Beginning of a data-block might not be on an  8-word boundary, in which case the leading characters/words are left blank.
            //per example:
            //023B: 6162 4344 6566 4748 6970 5152 7374 5556 | abCDefGHijKLmnOP

            for block_start in (current_address..stop_point).step_by(8) {
                if stop_point - block_start < 8 {
                    //Handle last (shorter) block.
                    let block_data = &program[block_start..stop_point];
                    let empties = 8 - (stop_point - block_start); //number of words that this block misses, and should be left empty.
                    write!(&mut destination_file,"{block_start:0>4x}: ").or(Err(AnalysisError::FileWriteError))?;
                    for word in block_data.iter() {
                        write!(&mut destination_file,"{:0>4x} ",word).or(Err(AnalysisError::FileWriteError))?;
                    }
                    for _ in 0..empties {
                        write!(&mut destination_file,"     ").or(Err(AnalysisError::FileWriteError))?;
                    }
                    write!(&mut destination_file,"| ").or(Err(AnalysisError::FileWriteError))?;

                    for word in block_data.iter() {
                        let l = char::from_u32((0x7f & *word) as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
                        let r = char::from_u32((0x7f & (*word >> 8)) as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
                        write!(&mut destination_file,"{l}{r}").or(Err(AnalysisError::FileWriteError))?;
                    }
                    //No need to pad the end out. Still need a newline though, so empty writeln. 
                    writeln!(&mut destination_file,"").or(Err(AnalysisError::FileWriteError))?;
                } else {
                    //Handle full block.
                    let block_data = &program[block_start..(block_start+8)];
                    let block_letters = String::from_iter(block_data.iter() //Take the words from the current block...
                        .map(|num| [0x7f & *num, 0x7f & (*num >> 8)]) //... split the 16-bit word into a pair of 8-bit characters in an array...
                        .flatten() //...flatten the two-wide arrays into a single sequence of bytes (presented as u16's still)...
                        .map(|num| char::from_u32(num as u32).unwrap_or(char::REPLACEMENT_CHARACTER))); //...and cast them to characters (or use the default replacement character � if that is not possible.)

                    writeln!(&mut destination_file,"{block_start:0>4x}: {:0>4x} {:0>4x} {:0>4x} {:0>4x} {:0>4x} {:0>4x} {:0>4x} {:0>4x} | {}",
                        block_data[0],block_data[1],block_data[2],block_data[3],block_data[4],block_data[5],block_data[6],block_data[7], block_letters
                    ).or(Err(AnalysisError::FileWriteError))?;
                }
            }
        }
    }

    
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