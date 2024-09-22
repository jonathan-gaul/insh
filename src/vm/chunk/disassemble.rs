use std::collections::HashMap;

use crate::{vm::op::Op, vm::value::{ivalue, fvalue}};

use super::bytecode_chunk::ByteCodeChunk;
use super::reader::ByteCodeChunkReader;

#[derive(Debug)]
pub enum DisassembleError {
    ChunkTooSmall
}

type DisassembleFn = fn(&ByteCodeChunk, &mut ByteCodeChunkReader, &str) -> Result<String, DisassembleError>;


impl ByteCodeChunk {
    fn disassemble_simple(&self, _: &mut ByteCodeChunkReader, name: &str) -> Result<String, DisassembleError> {
        Ok(name.to_owned())
    }

    fn disassemble_1<T: std::fmt::Display>(&self, reader: &mut ByteCodeChunkReader, name: &str) -> Result<String, DisassembleError> {
        if let Some(word) = reader.next::<T>() {
            Ok(format!("{} {}", name, word))
        } else {
            Err(DisassembleError::ChunkTooSmall)
        }
    }

    fn disassemble_string_const(&self, reader: &mut ByteCodeChunkReader, name: &str) -> Result<String, DisassembleError> {
        if let Some(word) = reader.next::<ivalue>() {
            Ok(format!("{} {} // \"{}\"", name, word, self.get_string(word as usize)))
        } else {
            Err(DisassembleError::ChunkTooSmall)
        }
    }

    pub fn disassemble(&self) -> Result<String, DisassembleError> {
        let mut reader = ByteCodeChunkReader::new(self);

        let op_funcs: HashMap<Op, (&str, DisassembleFn)> = [
            (Op::Return, ("rts", ByteCodeChunk::disassemble_simple as DisassembleFn)),
            (Op::IntConstant, ("cni", ByteCodeChunk::disassemble_1::<ivalue>)),
            (Op::FloatConstant, ("cnf", ByteCodeChunk::disassemble_1::<fvalue>)),
            (Op::StringConstant, ("cns", ByteCodeChunk::disassemble_string_const)),
            (Op::Pop, ("pop", ByteCodeChunk::disassemble_simple)),
            (Op::GetEnv, ("gev", ByteCodeChunk::disassemble_string_const)),
            (Op::SetEnv, ("sev", ByteCodeChunk::disassemble_string_const)),
            (Op::DefineLocal, ("dlv", ByteCodeChunk::disassemble_string_const)),
            (Op::PinLocal, ("plv", ByteCodeChunk::disassemble_string_const)),
            (Op::GetLocal, ("glv", ByteCodeChunk::disassemble_string_const)),
            (Op::SetLocal, ("slv", ByteCodeChunk::disassemble_string_const)),
            (Op::Add, ("add", ByteCodeChunk::disassemble_simple)),
            (Op::Subtract, ("sub", ByteCodeChunk::disassemble_simple)),
            (Op::Multiply, ("mul", ByteCodeChunk::disassemble_simple)),
            (Op::Divide, ("div", ByteCodeChunk::disassemble_simple)),
            (Op::Pipe, ("pip", ByteCodeChunk::disassemble_simple)),
            (Op::Swap, ("swp", ByteCodeChunk::disassemble_simple)),
            (Op::Negate, ("neg", ByteCodeChunk::disassemble_simple)),
            (Op::Command, ("cmd", ByteCodeChunk::disassemble_simple)),
            (Op::BranchIfFalse, ("brf", ByteCodeChunk::disassemble_1::<usize>)),
            (Op::SysCall, ("sys", ByteCodeChunk::disassemble_simple)),

        ].into_iter().map(|(op, (name, func))| (op, (name, func as DisassembleFn))).collect();

        let mut output = String::new();

        while let Some(op) = reader.next::<Op>() {
            output.push_str(&format!("{:08} [{:02x}]", reader.get_offset(), op as u8));
            if let Some((name, func)) = op_funcs.get(&op) {                
                output.push_str(&func(self, &mut reader, name)?);
                output.push_str("\n");
            } else {
                output.push_str("???");
            }
        };

        Ok(output)
    }
}
