use crate::instruction::*;

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

/// A block of data inside a larger sequence.
struct DataBlock {
    /// Offset from the start of the whole sequence for this block.
    start:usize,
    /// Information about read-write-execute ordering.
    block_type:BlockType,
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