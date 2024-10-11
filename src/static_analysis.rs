use std::ffi::OsStr;

use crate::instruction::*;

#[derive(PartialEq)]
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

#[derive(PartialEq)]
/// A block of data inside a larger sequence.
struct DataBlock {
    /// Offset from the start of the whole sequence for this block.
    start:usize,
    /// Information about read-write-execute ordering.
    block_type:BlockType,
}

impl DataBlock {
    fn new(start:usize,b_type:BlockType) -> Self {
        Self { start: start, block_type: b_type }
    }
}

impl PartialOrd for DataBlock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start.partial_cmp(&other.start)
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
    from:usize,
    /// Location at the other side, if one is known.
    target:Option<usize>,
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
    let mut blocks:Vec<DataBlock> = vec![DataBlock::new(0,BlockType::Text),DataBlock::new(1, BlockType::Unknown)];
    //Initialize empty, since no jumps can be known ahead of time.
    let mut jumps:Vec<Jump> = Vec::new();
    //Since the entire program gets analyzed from the entry point, start at address 0.
    let mut program_counter = 0usize;
    let mut resume_points:Vec<usize> = Vec::new();
    //Did you know that Rust has a dedicated infinite loop keyword? Pretty neat~
    loop {
        let op = Operation::from(program[program_counter]);

        match op {
            Operation::Jmp => {
                //For unconditional jumps and subroutine calls, continue 
                // from the other side of the jump or stop if the jump 
                // address is in a register (and therefore unpredictable.)
                let target = ParsedValue::from(program[program_counter+1]);
            }, 
            Operation::Jt | Operation::Jf | Operation::Call => {
                //For conditional jumps, register and
                // continue forward. 
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
        program_counter += 1 + (values as usize);
    }

    Ok(())
}