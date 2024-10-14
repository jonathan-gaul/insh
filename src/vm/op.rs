#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Op {
    Invalid = 0,

    Return = 1,

    IntConstant = 8,
    FloatConstant = 9,
    StringConstant = 10,
    BoolConstant = 11,
    NoneConstant = 12,

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
    Equal = 55,

    Command = 64,

    Branch = 96,
    BranchIfFalse = 97,
    BranchBack = 98,

    SysCall = 128,
    FunctionDefinition = 129,

    BeginScope = 224,
    EndScope = 225,

    // Add new opcodes before here.
    Unknown = 255,
}

pub const OP_SIZE: usize = size_of::<Op>();
