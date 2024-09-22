use super::{chunk::ByteCodeChunk, op::Op};

impl ByteCodeChunk {
    fn disassemble_simple(&self, name: &str, offset: usize) -> (usize, String) {
        (offset + 1, name.to_owned())
    }

    fn disassemble_i64_1(&self, name: &str, offset: usize) -> (usize, String) {
        let word = self.read_i64(offset + 1);
        (offset + 9, format!("{} {}", name, word))
    }

    fn disassemble_f64_1(&self, name: &str, offset: usize) -> (usize, String) {
        let f = self.read_f64(offset + 1);
        (offset + 9, format!("{} {}", name, f))
    }

    fn disassemble_string_const(&self, name: &str, offset: usize) -> (usize, String) {
        let word = self.read_i64(offset + 1) as usize;
        (
            offset + 9,
            format!("{} {} // \"{}\"", name, word, self.get_string(word)),
        )
    }

    pub fn disassemble(&self, offset: usize) -> (usize, String) {
        let op = self.read_op(offset);
        let (next, text) = match op {
            Op::Return => self.disassemble_simple("rts", offset),

            Op::IntConstant => self.disassemble_i64_1("cni", offset),
            Op::FloatConstant => self.disassemble_f64_1("cnf", offset),
            Op::StringConstant => self.disassemble_string_const("cns", offset),
            Op::Pop => self.disassemble_simple("pop", offset),

            Op::GetEnv => self.disassemble_string_const("gev", offset),
            Op::SetEnv => self.disassemble_string_const("sev", offset),
            Op::DefineLocal => self.disassemble_string_const("dlv", offset),
            Op::PinLocal => self.disassemble_string_const("plv", offset),
            Op::GetLocal => self.disassemble_string_const("glv", offset),
            Op::SetLocal => self.disassemble_string_const("slv", offset),

            Op::Add => self.disassemble_simple("add", offset),
            Op::Subtract => self.disassemble_simple("sub", offset),
            Op::Multiply => self.disassemble_simple("mul", offset),
            Op::Divide => self.disassemble_simple("div", offset),
            Op::Pipe => self.disassemble_simple("pip", offset),
            Op::Swap => self.disassemble_simple("swp", offset),

            Op::Negate => self.disassemble_simple("neg", offset),

            Op::Command => self.disassemble_simple("cmd", offset),

            Op::SysCall => self.disassemble_simple("sys", offset),

            _ => (offset + 1, "???".to_owned()),
        };

        (next, format!("{:08} [{:02x}] {}", offset, op as u8, text))
    }
}
