use crate::vm::op::Op;
use crate::vm::value::{fvalue, ivalue, Value, IVALUE_SIZE};

use super::compiler::Compiler;

impl Compiler {
    #[inline(always)]
    pub(super) fn emit_bytes(&mut self, v: &[u8]) {
        self.chunk.content.extend(v)
    }

    #[inline(always)]
    pub(super) fn emit_op(&mut self, v: Op) {
        self.emit_bytes(&[v as u8])
    }

    #[inline(always)]
    pub(super) fn emit_var(&mut self, op: Op, name: &String) {
        let constant_id = self.chunk.add_string(name);
        self.chunk.write_op(op);
        self.chunk.write_usize(constant_id);
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
        let constant_id = self.chunk.add_string(&v);
        self.chunk.write_op(Op::StringConstant);
        self.chunk.write_usize(constant_id);
        constant_id
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
        let constant_id = self.chunk.add_string(&cmd);
        self.chunk.write_op(Op::Command);
        self.chunk.write_usize(constant_id);
    }

    #[inline(always)]
    pub(super) fn emit_sys_call(&mut self, call: String) {
        let constant_id = self.chunk.add_string(&call);
        self.chunk.write_op(Op::SysCall);
        self.chunk.write_usize(constant_id);
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
        self.chunk.write_op(op);

        let offset = self.chunk.len();
        self.chunk.write_usize(usize::MAX);

        offset
    }

    #[inline(always)]
    pub(super) fn emit_loop(&mut self, start_offset: usize) {
        self.chunk.write_op(Op::BranchBack);

        let offset = self.chunk.len() - start_offset + IVALUE_SIZE;

        self.chunk.write_usize(offset);
    }

    #[inline(always)]
    pub(super) fn emit_function(&mut self, func: Value) {
        let id = self.chunk.add_function(func);
        self.chunk.write_op(Op::Function);
        self.chunk.write_usize(id);
    }
}
