use crate::vm::op::Op;

use super::compiler::Compiler;

impl Compiler {
    pub(super) fn emit_return(&mut self) {
        self.chunk.write_op(Op::Return)
    }

    pub(super) fn emit_int_constant(&mut self, v: i64) {
        self.chunk.write_op(Op::IntConstant);
        self.chunk.write_i64(v);
    }

    pub(super) fn emit_string_constant(&mut self, v: String) -> usize {
        let constant_id = self.chunk.add_string(v);
        self.chunk.write_op(Op::StringConstant);
        self.chunk.write_usize(constant_id);
        constant_id
    }

    pub(super) fn emit_float_constant(&mut self, v: f64) {
        self.chunk.write_op(Op::FloatConstant);
        self.chunk.write_f64(v);
    }

    pub(super) fn emit_command(&mut self, cmd: String) {
        self.emit_string_constant(cmd);
        self.chunk.write_op(Op::Command);
    }

    pub(super) fn emit_sys_call(&mut self, call: String) {
        self.chunk.write_op(Op::SysCall);
        self.emit_string_constant(call);
    }

    pub(super) fn emit_begin_scope(&mut self) {
        self.chunk.write_op(Op::BeginScope);
    }

    pub(super) fn emit_end_scope(&mut self) {
        self.chunk.write_op(Op::EndScope);
    }
}
