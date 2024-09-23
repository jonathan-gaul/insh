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
            (Op::Return, ("RTS", ByteCodeChunk::disassemble_simple as DisassembleFn)),
            (Op::IntConstant, ("CNI", ByteCodeChunk::disassemble_1::<ivalue>)),
            (Op::FloatConstant, ("CNF", ByteCodeChunk::disassemble_1::<fvalue>)),
            (Op::StringConstant, ("CNS", ByteCodeChunk::disassemble_string_const)),
            (Op::Pop, ("POP", ByteCodeChunk::disassemble_simple)),
            (Op::GetEnv, ("GEV", ByteCodeChunk::disassemble_string_const)),
            (Op::SetEnv, ("SEV", ByteCodeChunk::disassemble_string_const)),
            (Op::DefineLocal, ("DLV", ByteCodeChunk::disassemble_string_const)),
            (Op::PinLocal, ("PLV", ByteCodeChunk::disassemble_string_const)),
            (Op::GetLocal, ("GLV", ByteCodeChunk::disassemble_string_const)),
            (Op::SetLocal, ("SLV", ByteCodeChunk::disassemble_string_const)),
            (Op::Add, ("ADD", ByteCodeChunk::disassemble_simple)),
            (Op::Subtract, ("SUB", ByteCodeChunk::disassemble_simple)),
            (Op::Multiply, ("MUL", ByteCodeChunk::disassemble_simple)),
            (Op::Divide, ("DIV", ByteCodeChunk::disassemble_simple)),
            (Op::Pipe, ("PIP", ByteCodeChunk::disassemble_simple)),
            (Op::Swap, ("SWP", ByteCodeChunk::disassemble_simple)),
            (Op::Negate, ("NEG", ByteCodeChunk::disassemble_simple)),
            (Op::Command, ("CMD", ByteCodeChunk::disassemble_simple)),
            (Op::BranchIfFalse, ("BIF", ByteCodeChunk::disassemble_1::<usize>)),
            (Op::Branch, ("BRA", ByteCodeChunk::disassemble_1::<usize>)),
            (Op::BranchBack, ("BRB", ByteCodeChunk::disassemble_1::<usize>)),
            (Op::SysCall, ("SYS", ByteCodeChunk::disassemble_simple)),
            (Op::BeginScope, ("BSC", ByteCodeChunk::disassemble_simple)),
            (Op::EndScope, ("ESC", ByteCodeChunk::disassemble_simple)),

        ].into_iter().map(|(op, (name, func))| (op, (name, func as DisassembleFn))).collect();

        let mut output = String::new();

        while let Some(op) = reader.next::<Op>() {
            output += &format!("{:08} [{:02x}] ", reader.get_offset(), op as u8);
            if let Some((name, func)) = op_funcs.get(&op) {
                output += &func(self, &mut reader, name)?;
                output += "\n";
            } else {
                output += "???\n";
            }
        };

        Ok(output)
    }
}
