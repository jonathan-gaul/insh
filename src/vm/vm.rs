use std::ptr;

use crate::{string_value, vm::evaluate::EvaluateContext};

use super::{
    chunk::bytecode_chunk::ByteCodeChunk,
    local::{Scope, ScopeSearch},
    op::Op,
    value::{fvalue, ivalue, Value},
};

#[derive(Debug)]
pub enum VmError {
    InvalidOperation,
    InvalidValue,
    LocalAlreadyDefined,
    UndefinedLocal,
    PinnedLocal,
    UnknownSysCall,
}

pub struct Vm {
    chunk: ByteCodeChunk,
    ip: *const u8,
    stack: Vec<Value>,
    pub(super) scopes: Vec<Scope>,
}

impl Vm {
    pub(super) fn reset_stack(&mut self) {
        self.stack.clear();
    }

    pub(super) fn push_stack(&mut self, v: Value) {
        self.stack.push(v);
    }

    pub(super) fn pop_stack(&mut self) -> Value {
        if self.stack.is_empty() {
            panic!(
                "stack pop underflow at {:08}",
                (self.ip.wrapping_sub(self.chunk.content.as_ptr() as usize) as usize)
            )
        }
        self.stack.pop().unwrap()
    }

    fn peek_stack(&self, _: usize) -> Value {
        if let Some(last) = self.stack.last() {
            last.clone()
        } else {
            panic!(
                "stack peek underflow at {:08}",
                (self.ip.wrapping_sub(self.chunk.content.as_ptr() as usize) as usize)
            )
        }
    }

    fn read_as<T>(&mut self) -> T {
        let val: T;
        unsafe {
            val = ptr::read_unaligned(self.ip as *const T);
            self.ip = self.ip.add(size_of::<T>());
        }
        val
    }

    pub fn run(
        &mut self,
        chunk: ByteCodeChunk,
        context: EvaluateContext,
    ) -> Result<Value, VmError> {
        self.reset_stack();

        self.chunk = chunk;
        self.ip = self.chunk.content.as_ptr();

        loop {
            let op: Op = { self.read_as::<Op>() };

            match op {
                Op::Return => {
                    let value = self.pop_stack();
                    let value = self.evaluate(value, context);
                    return Ok(value);
                }

                Op::IntConstant => {
                    let v = self.read_as::<ivalue>();
                    self.push_stack(Value::Int(v));
                }

                Op::FloatConstant => {
                    let v = self.read_as::<fvalue>();
                    self.push_stack(Value::Float(v));
                }

                Op::StringConstant => {
                    let string_id = self.read_as::<usize>();
                    let s = self.chunk.get_string(string_id).to_owned();
                    self.push_stack(Value::String(s))
                }

                Op::BoolConstant => {
                    let v = self.read_as::<bool>();
                    self.push_stack(Value::Bool(v))
                }

                Op::NoneConstant => {
                    self.push_stack(Value::None);
                }

                Op::Negate => match self.pop_stack() {
                    Value::Int(x) => self.push_stack(Value::Int(-x)),
                    Value::Float(x) => self.push_stack(Value::Float(-x)),
                    _ => return Err(VmError::InvalidOperation),
                },

                Op::Add => match self.pop_stack() {
                    Value::None => match self.pop_stack() {
                        Value::None => self.push_stack(Value::Int(0)),
                        Value::Int(y) => self.push_stack(Value::Int(y)),
                        Value::Float(y) => self.push_stack(Value::Float(y)),
                        Value::String(v) => match (*v).parse::<ivalue>() {
                            Ok(y) => self.push_stack(Value::Int(y)),
                            Err(_) => return Err(VmError::InvalidValue),
                        },
                        _ => return Err(VmError::InvalidOperation),
                    },
                    Value::Int(x) => {
                        let y = self.pop_stack().to_ivalue()?;
                        self.push_stack(Value::Int(y + x));
                    }
                    Value::Float(x) => {
                        let y = self.pop_stack().to_fvalue()?;
                        self.push_stack(Value::Float(y + x));
                    }
                    Value::String(x) => {
                        let y = self.pop_stack().to_native_string();
                        self.push_stack(Value::String(y + &x));
                    }
                    _ => return Err(VmError::InvalidOperation),
                },

                Op::Multiply => match self.pop_stack() {
                    Value::None => match self.pop_stack() {
                        Value::None => self.push_stack(Value::None),
                        Value::Int(_) => self.push_stack(Value::Int(0)),
                        Value::Float(_) => self.push_stack(Value::Float(0.0)),
                        Value::String(_) => self.push_stack(Value::None),
                        _ => return Err(VmError::InvalidOperation),
                    },
                    Value::Int(x) => {
                        let y = self.pop_stack().to_ivalue()?;
                        self.push_stack(Value::Int(y * x));
                    }
                    Value::Float(x) => {
                        let y = self.pop_stack().to_fvalue()?;
                        self.push_stack(Value::Float(y * x));
                    }
                    Value::String(x) => match self.pop_stack() {
                        Value::None => self.push_stack(Value::Int(0)),
                        Value::Int(y) => {
                            self.push_stack(Value::String(x.repeat(y as usize)));
                        }
                        Value::Float(y) => {
                            self.push_stack(Value::String(x.repeat(y as usize)));
                        }
                        _ => return Err(VmError::InvalidOperation),
                    },
                    _ => return Err(VmError::InvalidOperation),
                },

                Op::Subtract => match self.pop_stack() {
                    Value::None => match self.pop_stack() {
                        Value::None => self.push_stack(Value::None),
                        Value::Int(y) => self.push_stack(Value::Int(y - 0)),
                        Value::Float(y) => self.push_stack(Value::Float(y - 0.0)),
                        Value::String(v) => match (*v).parse::<ivalue>() {
                            Ok(y) => self.push_stack(Value::Int(y - 0)),
                            Err(_) => return Err(VmError::InvalidValue),
                        },
                        _ => return Err(VmError::InvalidOperation),
                    },
                    Value::Int(x) => {
                        let y = self.pop_stack().to_ivalue()?;
                        self.push_stack(Value::Int(0 - x));
                    }
                    Value::Float(x) => {
                        let y = self.pop_stack().to_fvalue()?;
                        self.push_stack(Value::Float(y - x));
                    }
                    Value::String(x) => match self.pop_stack() {
                        Value::None => self.push_stack(Value::String("".to_owned())),
                        Value::Int(y) => {
                            let val = if (y as usize) >= (*x).len() {
                                "".to_owned()
                            } else {
                                x[..x.len() - (y as usize)].to_owned()
                            };
                            self.push_stack(Value::String(val));
                        }
                        Value::Float(y) => {
                            self.push_stack(Value::String(if (y as usize) >= (*x).len() {
                                "".to_owned()
                            } else {
                                (*x)[..(*x).len() - (y as usize)].to_owned()
                            }));
                        }
                        Value::String(y) => {
                            self.push_stack(Value::String((*y).replace(&(*x), "")));
                        }
                        _ => return Err(VmError::InvalidOperation),
                    },
                    _ => return Err(VmError::InvalidOperation),
                },

                Op::Divide => match self.pop_stack() {
                    Value::None => return Err(VmError::InvalidOperation),
                    Value::Int(x) => {
                        let y = self.pop_stack().to_ivalue()?;
                        self.push_stack(Value::Int(y / x));
                    }
                    Value::Float(x) => {
                        let y = self.pop_stack().to_fvalue()?;
                        self.push_stack(Value::Float(y / x));
                    }
                    _ => return Err(VmError::InvalidOperation),
                },

                Op::Equal => {
                    let result = match self.pop_stack() {
                        Value::None => match self.pop_stack() {
                            Value::None => true,
                            _ => false,
                        },
                        Value::Int(x) => {
                            let y = self.pop_stack().to_ivalue()?;
                            y == x
                        }
                        Value::Float(x) => {
                            let y = self.pop_stack().to_fvalue()?;
                            y == x
                        }
                        Value::String(x) => {
                            let y = self.pop_stack().to_native_string();
                            y == x
                        }
                        Value::Bool(x) => {
                            let y = self.pop_stack().to_native_bool();
                            y == x
                        }
                        Value::Map(_) => false,
                        Value::Command(..) => return Err(VmError::InvalidOperation),
                        Value::Function(..) => return Err(VmError::InvalidOperation),
                    };

                    self.push_stack(Value::Bool(result));
                }

                Op::Command => {
                    let cmd = self.read_string_const();

                    if let Value::Int(arg_count) = self.pop_stack() {
                        let mut args = Vec::new();
                        for i in 0..arg_count {
                            args.push(self.pop_stack());
                        }
                        args.reverse();

                        self.push_stack(Value::Command(cmd, args));
                    } else {
                        return Err(VmError::InvalidOperation);
                    }
                }

                Op::GetEnv => {
                    let name = self.read_string_const();
                    match std::env::var(name) {
                        Ok(value) => self.push_stack(Value::String(value)),
                        Err(_) => self.push_stack(Value::String("".to_owned())),
                    }
                }

                Op::SetEnv => {
                    let name = self.read_string_const();
                    let value = self.pop_stack();
                    match std::env::var(&name) {
                        Ok(original_value) => {
                            std::env::set_var(name, value.to_native_string());
                            self.push_stack(Value::String(original_value));
                        }
                        Err(_) => {
                            std::env::set_var(name, value.to_native_string());
                            self.push_stack(Value::String("".to_owned()));
                        }
                    }
                }

                Op::BeginScope => self.begin_scope(),
                Op::EndScope => self.end_scope(),

                Op::SetLocal | Op::PinLocal | Op::DefineLocal => {
                    let name = self.read_string_const();
                    let value = self.pop_stack();
                    let actual = self.evaluate(value, EvaluateContext::Assignment);
                    match op {
                        Op::SetLocal => self.set_local(&name, actual.clone())?,
                        Op::DefineLocal => self.define_local(name, actual.clone(), false)?,
                        Op::PinLocal => self.define_local(name, actual.clone(), true)?,
                        _ => {}
                    };
                    self.push_stack(actual);
                }

                Op::GetLocal => {
                    let name = self.read_string_const();
                    let result = self.get_local(&name, ScopeSearch::AllScopes);
                    match result {
                        Some(value) => self.push_stack(value.value.clone()),
                        None => return Err(VmError::UndefinedLocal),
                    }
                }

                Op::Pop => {
                    let v = self.pop_stack();
                    self.evaluate(v, EvaluateContext::None);
                }

                Op::BranchIfFalse => {
                    let dist = self.read_as::<usize>();
                    let val = self.evaluate(self.peek_stack(0), EvaluateContext::None);
                    if !val.to_native_bool() {
                        self.ip = self.ip.wrapping_add(dist);
                    }
                }

                Op::Branch => {
                    let dist = self.read_as::<usize>();
                    self.ip = self.ip.wrapping_add(dist);
                }

                Op::BranchBack => {
                    let dist = self.read_as::<usize>();
                    self.ip = self.ip.wrapping_sub(dist);
                }

                Op::SysCall => {
                    let call = self.read_string_const();
                    match call.as_str() {
                        "read" => self.syscall_read()?,
                        _ => return Err(VmError::UnknownSysCall),
                    }
                }

                x => {
                    let offset = self.ip.wrapping_sub(self.chunk.content.as_ptr() as usize);
                    println!("unknown op: {:?} at {:08}", x, offset as usize);
                }
            }
        }
    }

    fn read_string_const(&mut self) -> String {
        let string_id = self.read_as::<usize>();
        self.chunk.get_string(string_id).to_owned()
    }

    pub fn new() -> Vm {
        let chunk = ByteCodeChunk::new();
        Vm {
            ip: chunk.content.as_ptr(),
            chunk,
            stack: Vec::new(),
            scopes: vec![Scope::new()],
        }
    }
}
