use super::{runtime, value::Value, vm::Vm};

pub enum EvaluateContext {
    None,
    Assignment,
}

impl Vm {
    pub(super) fn evaluate(&self, v: Value, context: EvaluateContext) -> Value {
        match v {
            Value::Command(cmd, args) => match context {
                EvaluateContext::Assignment => {
                    let (_, out, _) = runtime::execute(cmd, args, true);
                    Value::String(out)
                }
                _ => {
                    let (status, _, _) = runtime::execute(cmd, args, false);
                    Value::Int(status)
                }
            },
            x => x,
        }
    }
}
