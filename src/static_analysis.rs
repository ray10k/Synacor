use std::ffi::OsStr;

use crate::instruction::*;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
/// What type of block this is, depending on when it gets read, written or executed.
enum BlockType {
    /// This block is executable code, which doesn't get overwritten.
    Text,
    /// This block is data only; reads and writes happen, but no executions.
    Data,
    /// This block starts out as Text, and later gets overwritten but not executed again.
    TextDemoted,
    /// This block gets written to, and later gets executed. Here be dragons.
    SelfModifying,
    /// This block is unreached (probably)
    Unknown,
}

#[derive(PartialEq, Eq, Ord)]
/// A block of data inside a larger sequence.
struct DataBlock {
    /// Offset from the end of the whole sequence for this block.
    end:u16,
    /// Information about read-write-execute ordering.
    block_type:BlockType,
}

impl DataBlock {
    fn new(end:u16,b_type:BlockType) -> Self {
        Self { end:end, block_type: b_type }
    }
}

impl PartialOrd for DataBlock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.end.partial_cmp(&other.end)
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
    //Initially, assume that the first word will get executed, and that the rest of the 
    //program is unknown.
    let mut blocks:Vec<DataBlock> = vec![DataBlock::new(0,BlockType::Text),DataBlock::new(0x7fff, BlockType::Unknown)];
    let mut current_block = find_containing_block(&mut blocks, 0);
    //Initialize empty, since no jumps can be known ahead of time.
    let mut jumps:Vec<Jump> = Vec::new();
    //Since the entire program gets analyzed from the entry point, start at address 0.
    let mut program_counter = 0u16;
    let mut resume_points:Vec<u16> = Vec::new();
    //Did you know that Rust has a dedicated infinite loop keyword? Pretty neat~
    loop {
        let op = Operation::from(program[program_counter as usize]);

        match op {
            Operation::Jmp => {
                //For unconditional jumps and subroutine calls, continue 
                // from the other side of the jump or stop if the jump 
                // address is in a register (and therefore unpredictable.)
                let target = ParsedValue::from(program[program_counter as usize+1]);
                let jump_destination:Option<u16>;
                if let ParsedValue::Literal(dest) = target {
                    resume_points.push(dest);
                    jump_destination = Some(dest);
                }
                else {
                    jump_destination = None
                }
                let jump_info = Jump{from:program_counter, target:jump_destination,jump_type:JumpType::Fixed};
                jumps.push(jump_info);
                //Since there is no way to continue the current block, assume that the block ends here instead.
                //Update the running block (the end is found,) and try to grab a new block.
                current_block.end = program_counter + 1 + op.operands();
                if let Some(resume) = resume_points.pop() {
                    program_counter = resume;

                } else {
                    //No more possible resume-points (which *can* happen, if the jump being analyzed had a 
                    //target in a register.)
                    break;
                }
            }, 
            Operation::Jt | Operation::Jf | Operation::Call => {
                //For conditional jumps, register and
                // continue forward. Note that CALL is a conditional jump since the subroutine "should"
                // eventually return here.
            },
            Operation::Ret | Operation::Halt => {
                //For returns from subroutines or end-of-program, mark the
                // end of the block and pick another block to continue analysis.
            }
            _ => {
                //For other instructions, just continue as normal.
            }
        }        
        let values = op.operands();
        program_counter += 1 + values;
    }

    Ok(())
}

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
}