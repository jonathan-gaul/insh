use std::io::stdin;

use crate::vm::{
    value::{fvalue, ivalue, Value},
    vm::{Vm, VmError},
};

impl Vm {
    fn get_value(&mut self, from: String) -> Result<Value, VmError> {
        match from.as_str() {
            "console" => {
                let mut buffer = String::new();
                match stdin().read_line(&mut buffer) {
                    Ok(_) => Ok(Value::String(buffer)),
                    Err(_) => return Err(VmError::InvalidOperation),
                }
            }
            _ => return Ok(Value::String(from)),
        }
    }

    fn read_number(&mut self, from: Value) -> Result<Value, VmError> {
        match from {
            Value::Bool(_) => Ok(Value::Int(from.to_ivalue()?)),
            Value::String(s) => {
                let v = self.get_value(s)?;
                let t = v.to_native_string();

                let first_digit_pos = match t.find(|c: char| c.is_digit(10)) {
                    Some(x) => x,
                    None => return Err(VmError::InvalidValue),
                };

                let number_str = &t[first_digit_pos..]
                    .chars()
                    .take_while(|c| c.is_digit(10) || *c == '.')
                    .collect::<String>();

                if number_str.contains('.') {
                    if let Ok(x) = number_str.parse::<fvalue>() {
                        Ok(Value::Float(x))
                    } else {
                        return Err(VmError::InvalidValue);
                    }
                } else if let Ok(x) = number_str.parse::<ivalue>() {
                    Ok(Value::Int(x))
                } else {
                    return Err(VmError::InvalidValue);
                }
            }
            _ => return Err(VmError::InvalidOperation),
        }
    }

    pub fn syscall_read(&mut self) -> Result<(), VmError> {
        let from = self.pop_stack();
        let what = self.pop_stack().to_native_string();

        println!("syscall: {} from {}", what, from);

        let value = match what.as_str() {
            "number" => self.read_number(from)?,
            _ => return Err(VmError::InvalidOperation),
        };

        self.push_stack(value);

        Ok(())
    }
}
