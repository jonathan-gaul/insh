use crate::vm::op::Op;
use crate::vm::value::{fvalue, ivalue, Value, IVALUE_SIZE};

use super::compiler::Compiler;

impl Compiler {
    #[inline(always)]
    pub(super) fn emit_bytes(&mut self, v: &[u8]) {
        if let Value::Function(_, _, chunk) = &mut self.function {
            chunk.content.extend(v)
        }
    }

    #[inline(always)]
    pub(super) fn emit_op(&mut self, v: Op) {
        self.emit_bytes(&[v as u8])
    }

    #[inline(always)]
    pub(super) fn emit_var(&mut self, op: Op, name: &String) {
        if let Value::Function(_, _, chunk) = &mut self.function {
            let constant_id = chunk.add_string(name);
            chunk.write_op(op);
            chunk.write_usize(constant_id);
        }
    }

    #[inline(always)]
    pub(super) fn emit_ivalue(&mut self, v: ivalue) {
        self.emit_bytes(&ivalue::to_ne_bytes(v))
    }

    #[inline(always)]
    pub(super) fn emit_fvalue(&mut self, v: fvalue) {
        self.emit_bytes(&fvalue::to_ne_bytes(v))
    }

    #[inline(always)]
    pub(super) fn emit_return(&mut self) {
        self.emit_op(Op::Return);
    }

    #[inline(always)]
    pub(super) fn emit_bool_constant(&mut self, v: bool) {
        self.emit_op(Op::BoolConstant);
        self.emit_bytes(&[if v { 1 } else { 0 }])
    }

    #[inline(always)]
    pub(super) fn emit_int_constant(&mut self, v: ivalue) {
        self.emit_op(Op::IntConstant);
        self.emit_ivalue(v)
    }

    #[inline(always)]
    pub(super) fn emit_string_constant(&mut self, v: String) -> usize {
        if let Value::Function(_, _, chunk) = &mut self.function {
            let constant_id = chunk.add_string(&v);
            chunk.write_op(Op::StringConstant);
            chunk.write_usize(constant_id);
            constant_id
        } else {
            0
        }
    }

    #[inline(always)]
    pub(super) fn emit_float_constant(&mut self, v: fvalue) {
        self.emit_op(Op::FloatConstant);
        self.emit_fvalue(v);
    }

    #[inline(always)]
    pub(super) fn emit_none(&mut self) {
        self.emit_op(Op::NoneConstant);
    }

    #[inline(always)]
    pub(super) fn emit_command(&mut self, cmd: String) {
        if let Value::Function(_, _, chunk) = &mut self.function {
            let constant_id = chunk.add_string(&cmd);
            chunk.write_op(Op::Command);
            chunk.write_usize(constant_id);
        }
    }

    #[inline(always)]
    pub(super) fn emit_sys_call(&mut self, call: String) {
        if let Value::Function(_, _, chunk) = &mut self.function {
            let constant_id = chunk.add_string(&call);
            chunk.write_op(Op::SysCall);
            chunk.write_usize(constant_id);
        }
    }

    #[inline(always)]
    pub(super) fn emit_begin_scope(&mut self) {
        self.emit_op(Op::BeginScope);
    }

    #[inline(always)]
    pub(super) fn emit_end_scope(&mut self) {
        self.emit_op(Op::EndScope);
    }

    #[inline(always)]
    pub(super) fn emit_branch(&mut self, op: Op) -> usize {
        if let Value::Function(_, _, chunk) = &mut self.function {
            chunk.write_op(op);

            let offset = chunk.len();
            chunk.write_usize(usize::MAX);

            offset
        } else {
            0
        }
    }

    #[inline(always)]
    pub(super) fn emit_loop(&mut self, start_offset: usize) {
        if let Value::Function(_, _, chunk) = &mut self.function {
            chunk.write_op(Op::BranchBack);

            let offset = chunk.len() - start_offset + IVALUE_SIZE;

            chunk.write_usize(offset);
        }
    }

    #[inline(always)]
    pub(super) fn emit_function(&mut self, func: Value) {
        if let Value::Function(_, _, chunk) = &mut self.function {
            let id = chunk.add_function(func);
            chunk.write_op(Op::Function);
            chunk.write_usize(id);
        }
    }
}
