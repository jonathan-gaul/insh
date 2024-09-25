use std::collections::HashMap;

pub use i32 as ivalue;
pub const IVALUE_SIZE: usize = size_of::<ivalue>();

pub use f64 as fvalue;

use super::{chunk::bytecode_chunk::ByteCodeChunk, vm::VmError};
pub const FVALUE_SIZE: usize = size_of::<fvalue>();

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Int(ivalue),
    Float(fvalue),
    String(String),
    Bool(bool),
    Map(HashMap<Value, Value>),
    Command(String, Vec<Value>),
    Function(String, u8, ByteCodeChunk),
}

#[macro_export]
macro_rules! string_value {
    ($($arg:tt)*) => {
        Value::String((format!($($arg)*).to_owned()))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "(empty)"),
            Value::Int(x) => write!(f, "{}", x),
            Value::Float(x) => write!(f, "{:?}", x),
            Value::String(x) => write!(f, "{}", x),
            Value::Map(x) => {
                for (k, v) in x {
                    _ = write!(f, "{} = {}", k, v);
                }
                Ok(())
            }
            Value::Command(cmd, args) => write!(f, "{}/{}", cmd, args.len()),
            Value::Bool(x) => write!(f, "{}", x),
            Value::Function(name, arity, _) => write!(f, "{}/{}", name, arity),
            // Value::Array(x) => {
            //     for k in x {
            //         _ = write!(f, "{}", k);
            //     }
            //     Ok(())
            // }
        }
    }
}

impl Value {
    pub fn to_native_string(&self) -> String {
        match self {
            Value::None => format!("(empty)"),
            Value::Int(x) => format!("{}", x),
            Value::Float(x) => format!("{}", x),
            Value::String(x) => x.to_owned(),
            Value::Map(_) => "".to_owned(),
            Value::Command(x, _) => x.to_owned(),
            Value::Bool(x) => format!("{}", x),
            Value::Function(name, arity, _) => format!("{}/{}", name, arity),
        }
    }

    pub fn to_native_bool(&self) -> bool {
        match self {
            Value::None
            | Value::Int(0)
            | Value::Float(0.0)
            | Value::Bool(false)
            | Value::Command(_, _) => false,
            Value::String(x) => x.len() != 0,
            Value::Map(x) => x.len() != 0,
            _ => true,
        }
    }

    pub fn to_fvalue(&self) -> Result<fvalue, VmError> {
        match self {
            Value::None => Ok(0.0),
            Value::String(v) => match (*v).parse::<fvalue>() {
                Ok(x) => Ok(x),
                Err(_) => return Err(VmError::InvalidValue),
            },
            Value::Int(x) => Ok(*x as fvalue),
            Value::Float(x) => Ok(*x),
            Value::Map(_) => return Err(VmError::InvalidOperation),
            Value::Command(..) => return Err(VmError::InvalidOperation),
            Value::Bool(x) => Ok(match x {
                true => 1,
                false => 0,
            } as fvalue),
            Value::Function(..) => return Err(VmError::InvalidOperation),
        }
    }

    pub fn to_ivalue(&self) -> Result<ivalue, VmError> {
        match self {
            Value::None => Ok(0),
            Value::String(v) => match (*v).parse::<ivalue>() {
                Ok(x) => Ok(x),
                Err(_) => return Err(VmError::InvalidValue),
            },
            Value::Int(x) => Ok(*x),
            Value::Float(x) => Ok(*x as ivalue),
            Value::Map(_) => return Err(VmError::InvalidOperation),
            Value::Command(..) => return Err(VmError::InvalidOperation),
            Value::Bool(x) => Ok(match x {
                true => 1,
                false => 0,
            }),
            Value::Function(..) => return Err(VmError::InvalidOperation),
        }
    }
}
