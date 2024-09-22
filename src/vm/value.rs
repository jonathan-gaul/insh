use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Map(HashMap<Value, Value>),
    Command(String, Vec<Value>),
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
        }
    }
}
