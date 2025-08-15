use std::fmt::{Display, Result as fmtResult};

#[derive(Debug, PartialEq)]
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
    Error(u16),
}

impl From<u16> for Operation {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Halt,
            1 => Self::Set,
            2 => Self::Push,
            3 => Self::Pop,
            4 => Self::Eq,
            5 => Self::Gt,
            6 => Self::Jmp,
            7 => Self::Jt,
            8 => Self::Jf,
            9 => Self::Add,
            10 => Self::Mult,
            11 => Self::Mod,
            12 => Self::And,
            13 => Self::Or,
            14 => Self::Not,
            15 => Self::Rmem,
            16 => Self::Wmem,
            17 => Self::Call,
            18 => Self::Ret,
            19 => Self::Out,
            20 => Self::In,
            21 => Self::Noop,
            _ => Self::Error(value),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::Halt => "HALT",
                Self::Set => "SET ",
                Self::Push => "PUSH",
                Self::Pop => "POP ",
                Self::Eq => "EQ  ",
                Self::Gt => "GT  ",
                Self::Jmp => "JMP ",
                Self::Jt => "JT  ",
                Self::Jf => "JF  ",
                Self::Add => "ADD ",
                Self::Mult => "MULT",
                Self::Mod => "MOD ",
                Self::And => "AND ",
                Self::Or => "OR  ",
                Self::Not => "NOT ",
                Self::Rmem => "RMEM",
                Self::Wmem => "WMEM",
                Self::Call => "CALL",
                Self::Ret => "RET ",
                Self::Out => "OUT ",
                Self::In => "IN  ",
                Self::Noop => "NOOP",
                Self::Error(_) => "!?!?",
            }
        )
    }
}

impl Operation {
    pub fn operands(&self) -> u16 {
        match self {
            Self::Halt | Self::Ret | Self::Noop => 0,
            Self::Push | Self::Pop | Self::Jmp | Self::Call | Self::Out | Self::In => 1,
            Self::Set | Self::Jt | Self::Jf | Self::Not | Self::Rmem | Self::Wmem => 2,
            Self::Eq | Self::Gt | Self::Add | Self::Mult | Self::Mod | Self::And | Self::Or => 3,
            Self::Error(_) => 0xffff,
        }
    }
}

pub enum ParsedValue {
    Literal(u16),
    Register(u16),
    Error(u16),
}

impl From<u16> for ParsedValue {
    fn from(value: u16) -> Self {
        match value {
            0..=32767 => Self::Literal(value),
            32768..=32775 => Self::Register(value - 32768),
            _ => Self::Error(value),
        }
    }
}

impl Display for ParsedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        match self {
            Self::Literal(v) => write!(f, "{v:04x}"),
            Self::Register(v) => write!(f, "  R{v}"),
            Self::Error(v) => write!(f, "E({v})"),
        }
    }
}
