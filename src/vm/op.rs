#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Op {
    Invalid = 0,

    Return = 1,

    IntConstant = 8,
    FloatConstant = 9,
    StringConstant = 10,
    BoolConstant = 11,

    Pop = 16,

    GetEnv = 24,
    SetEnv = 25,

    DefineLocal = 32,
    PinLocal = 33,
    GetLocal = 34,
    SetLocal = 35,

    Negate = 48,
    Add = 49,
    Subtract = 50,
    Multiply = 51,
    Divide = 52,
    Pipe = 53,
    Swap = 54,

    Command = 64,

    BranchIfFalse = 96,

    SysCall = 128,

    BeginScope = 224,
    EndScope = 225,

    // Add new opcodes before here.
    Unknown = 255,
}

pub const OP_SIZE: usize = size_of::<Op>();

impl Op {
    pub fn to_ne_bytes(op: &Op) -> [u8; 1] {
        [op.clone() as u8]
    }

    pub fn from(val: u8) -> Op {
        match val {
            x if x == Op::Invalid as u8 => Op::Invalid,

            x if x == Op::Return as u8 => Op::Return,

            x if x == Op::IntConstant as u8 => Op::IntConstant,
            x if x == Op::FloatConstant as u8 => Op::FloatConstant,
            x if x == Op::StringConstant as u8 => Op::StringConstant,
            x if x == Op::Pop as u8 => Op::Pop,

            x if x == Op::DefineLocal as u8 => Op::DefineLocal,
            x if x == Op::PinLocal as u8 => Op::PinLocal,
            x if x == Op::GetEnv as u8 => Op::GetEnv,
            x if x == Op::SetEnv as u8 => Op::SetEnv,
            x if x == Op::GetLocal as u8 => Op::GetLocal,
            x if x == Op::SetLocal as u8 => Op::SetLocal,

            x if x == Op::Negate as u8 => Op::Negate,
            x if x == Op::Add as u8 => Op::Add,
            x if x == Op::Subtract as u8 => Op::Subtract,
            x if x == Op::Multiply as u8 => Op::Multiply,
            x if x == Op::Divide as u8 => Op::Divide,
            x if x == Op::Pipe as u8 => Op::Pipe,
            x if x == Op::Swap as u8 => Op::Swap,

            x if x == Op::BeginScope as u8 => Op::BeginScope,
            x if x == Op::EndScope as u8 => Op::EndScope,

            x if x == Op::Command as u8 => Op::Command,

            x if x == Op::SysCall as u8 => Op::SysCall,
            _ => Op::Unknown,
        }
    }
}
