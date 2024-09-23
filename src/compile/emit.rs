use crate::vm::op::Op;
use crate::vm::value::{fvalue, ivalue, IVALUE_SIZE};

use super::compiler::Compiler;

impl Compiler {
    pub(super) fn emit_return(&mut self) {
        self.chunk.write_op(&Op::Return)
    }

    pub(super) fn emit_int_constant(&mut self, v: ivalue) {
        self.chunk.write_op(&Op::IntConstant);
        self.chunk.write_ivalue(v);
    }

    pub(super) fn emit_string_constant(&mut self, v: String) -> ivalue {
        let constant_id = self.chunk.add_string(v);
        self.chunk.write_op(&Op::StringConstant);
        self.chunk.write_ivalue(constant_id);
        constant_id
    }

    pub(super) fn emit_float_constant(&mut self, v: fvalue) {
        self.chunk.write_op(&Op::FloatConstant);
        self.chunk.write_fvalue(v);
    }

    pub(super) fn emit_command(&mut self, cmd: String) {
        self.emit_string_constant(cmd);
        self.chunk.write_op(&Op::Command);
    }

    pub(super) fn emit_sys_call(&mut self, call: String) {
        self.chunk.write_op(&Op::SysCall);
        self.emit_string_constant(call);
    }

    pub(super) fn emit_begin_scope(&mut self) {
        self.chunk.write_op(&Op::BeginScope);
    }

    pub(super) fn emit_end_scope(&mut self) {
        self.chunk.write_op(&Op::EndScope);
    }

    pub(super) fn emit_branch(&mut self, op: &Op) -> usize {        
        self.chunk.write_op(op);

        let offset = self.chunk.len();

        self.chunk.write_usize(usize::MAX);
        
        offset
    }

    pub(super) fn emit_loop(&mut self, start_offset: usize) {
        self.chunk.write_op(&Op::BranchBack);

        let offset = self.chunk.content.len() - start_offset + IVALUE_SIZE;
        
        self.chunk.write_usize(offset);
    }
}
