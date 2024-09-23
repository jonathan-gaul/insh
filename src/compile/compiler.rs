use std::mem;

use crate::{
    scan::{
        scanner::Scanner,
        tokens::{Token, TokenType},
    },
    vm::{chunk::bytecode_chunk::ByteCodeChunk, op::Op},
    vm::value::{ivalue, fvalue},
};

use super::{
    errors::CompileError,
    precedence::{ParseRule, Precedence},
};

pub struct LocalVariable {
    name: Token,
    depth: i64,
}

pub struct Compiler {
    pub(super) scanner: Scanner,
    pub(super) chunk: ByteCodeChunk,

    pub(super) previous: Token,
    pub(super) current: Token,

    pub(super) debug_output_chunk: bool,

    pub(super) locals: Vec<LocalVariable>,
    pub(super) local_count: usize,
    pub(super) scope_depth: i64,
}

impl Compiler {
    pub fn new(scanner: Scanner, chunk: ByteCodeChunk) -> Self {
        Compiler {
            scanner,
            chunk,
            previous: Token::new(TokenType::Unknown, String::new()),
            current: Token::new(TokenType::Unknown, String::new()),

            debug_output_chunk: true,

            locals: Vec::new(),
            local_count: 0,
            scope_depth: 0,
        }
    }

    pub fn into_chunk(self) -> ByteCodeChunk {
        self.chunk
    }

    pub(super) fn advance(&mut self) -> Result<(), CompileError> {
        match self.scanner.read_token() {
            Ok(token) => {
                self.previous = mem::replace(&mut self.current, token);             
                Ok(())
            }
            Err(e) => Err(CompileError::ScanError(e)),
        }
    }

    pub(super) fn consume(&mut self, token_type: TokenType) -> Result<(), CompileError> {
        if self.current.token_type() == token_type {
            self.advance()
        } else {
            Err(CompileError::MissingToken(token_type, self.current.clone()))
        }
    }

    pub(super) fn expression(&mut self) -> Result<(), CompileError> {
        self.parse_precedence(Precedence::Assignment as u8)
    }

    pub(super) fn grouping(&mut self, _: bool) -> Result<(), CompileError> {
        self.expression()?;
        self.consume(TokenType::CloseBracket)?;
        Ok(())
    }

    pub(super) fn unary(&mut self, _: bool) -> Result<(), CompileError> {
        let operator_type = self.previous.token_type;

        self.parse_precedence(Precedence::Unary as u8)?;

        match operator_type {
            TokenType::Minus => self.chunk.write_op(&Op::Negate),
            TokenType::Plus => {}
            _ => return Err(CompileError::UnknownUnaryOperator),
        }

        Ok(())
    }

    pub(super) fn binary(&mut self, _: bool) -> Result<(), CompileError> {
        let operator_type = self.previous.token_type;

        let prec = ParseRule::precedence_for(operator_type) as u8;
        self.parse_precedence(prec + 1)?;

        match operator_type {
            TokenType::Plus => self.chunk.write_op(&Op::Add),
            TokenType::Minus => self.chunk.write_op(&Op::Subtract),
            TokenType::Star => self.chunk.write_op(&Op::Multiply),
            TokenType::Slash => self.chunk.write_op(&Op::Divide),
            TokenType::Pipe => self.chunk.write_op(&Op::Pipe),
            _ => {}
        }

        Ok(())
    }

    pub(super) fn block(&mut self, _: bool) -> Result<(), CompileError> {
        self.emit_begin_scope();
        self.begin_scope();
        while !self.check(TokenType::CloseBrace) && !self.check(TokenType::EndOfFile) {
            self.expression()?;
        }
        self.end_scope();
        self.emit_end_scope();
        Ok(())
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while self.local_count > 0 && self.locals[self.local_count - 1].depth > self.scope_depth {
            self.chunk.write_op(&Op::Pop);
            self.local_count -= 1;
        }
    }

    pub(super) fn command(&mut self, _: bool) -> Result<(), CompileError> {
        let command = self.previous.text.to_owned();
        let mut count = 0;

        loop {
            self.expression()?;

            if matches!(
                self.previous.token_type,
                TokenType::EndCommand | TokenType::EndOfFile | TokenType::EndOfLine
            ) {
                break;
            }

            count += 1;
        }

        self.emit_int_constant(count);
        self.emit_command(command);

        Ok(())
    }

    pub(super) fn parse(&mut self, _: bool) -> Result<(), CompileError> {
        // parse <expr>
        self.expression()?;

        // from
        self.consume(TokenType::From)?;

        // <expr>
        self.expression()?;

        self.emit_sys_call("parse".to_owned());

        Ok(())
    }

    pub(super) fn string_constant(&mut self, _: bool) -> Result<(), CompileError> {
        self.emit_string_constant(self.previous.text.to_owned());
        Ok(())
    }

    pub(super) fn true_literal(&mut self, _: bool) -> Result<(), CompileError> {
        self.chunk.write_op(&Op::BoolConstant);
        self.chunk.write_bool(true);
        Ok(())
    }

    pub(super) fn false_literal(&mut self, _: bool) -> Result<(), CompileError> {
        self.chunk.write_op(&Op::BoolConstant);
        self.chunk.write_bool(false);
        Ok(())
    }

    pub(super) fn int_constant(&mut self, _: bool) -> Result<(), CompileError> {
        match self.previous.text.parse::<ivalue>() {
            Ok(val) => self.emit_int_constant(val),
            Err(_) => return Err(CompileError::ParseError()),
        }
        Ok(())
    }

    pub(super) fn float_constant(&mut self, _: bool) -> Result<(), CompileError> {
        match self.previous.text.parse::<fvalue>() {
            Ok(val) => self.emit_float_constant(val),
            Err(_) => return Err(CompileError::ParseError()),
        }
        Ok(())
    }

    fn check(&self, t: TokenType) -> bool {
        self.current.token_type == t
    }

    pub(super) fn match_type(&mut self, t: TokenType) -> Result<bool, CompileError> {
        Ok(if !self.check(t) {
            false
        } else {
            self.advance()?;
            true
        })
    }

    pub(super) fn env_var(&mut self, _: bool) -> Result<(), CompileError> {
        let constant_id = self.chunk.add_string(self.previous.text.to_owned());
        let op = if self.match_type(TokenType::Equal)? {
            self.expression()?;
            Op::SetEnv
        } else {
            Op::GetEnv
        };

        self.chunk.write_op(&op);
        self.chunk.write_ivalue(constant_id);
        Ok(())
    }

    pub(super) fn let_var(&mut self, _: bool) -> Result<(), CompileError> {
        // let @identifier = <expr>
        self.advance()?;

        let constant_id = self.chunk.add_string(self.previous.text.to_owned());
        self.match_type(TokenType::Equal)?;

        self.expression()?;
        self.chunk.write_op(&Op::DefineLocal);
        self.chunk.write_ivalue(constant_id);

        Ok(())
    }

    pub(super) fn pin_var(&mut self, _: bool) -> Result<(), CompileError> {
        // pin @identifier = <expr>
        self.advance()?;

        let constant_id = self.chunk.add_string(self.previous.text.to_owned());
        self.match_type(TokenType::Equal)?;

        self.expression()?;
        self.chunk.write_op(&Op::PinLocal);
        self.chunk.write_ivalue(constant_id);

        Ok(())
    }

    pub(super) fn local_var(&mut self, can_assign: bool) -> Result<(), CompileError> {
        let constant_id = self.chunk.add_string(self.previous.text.to_owned());
        let is_set = can_assign && self.match_type(TokenType::Equal)?;

        let op = if is_set {
            self.expression()?;
            Op::SetLocal
        } else {
            Op::GetLocal
        };

        self.chunk.write_op(&op);
        self.chunk.write_ivalue(constant_id);

        Ok(())
    }

    pub(super) fn if_(&mut self, _: bool) -> Result<(), CompileError> {

        // if <expr>
        self.expression()?;

        // then
        self.consume(TokenType::Then)?;

        let offset = self.emit_branch(&Op::BranchIfFalse);
        self.chunk.write_op(&Op::Pop);

        // <expr>
        self.expression()?;

        let else_offset = self.emit_branch(&Op::Branch);

        self.patch_branch(offset);
        self.chunk.write_op(&Op::Pop);
        
        if self.match_type(TokenType::Else)? {
            self.expression()?;
        }

        self.patch_branch(else_offset);

        Ok(())
    }

    pub(super) fn and(&mut self, _: bool) -> Result<(), CompileError> {
        let offset = self.emit_branch(&Op::BranchIfFalse);

        self.chunk.write_op(&Op::Pop);
        self.parse_precedence(Precedence::And as u8)?;

        self.patch_branch(offset);

        Ok(())
    }

    pub(super) fn or(&mut self, _: bool) -> Result<(), CompileError> {
        let else_offset = self.emit_branch(&Op::BranchIfFalse);
        let end_offset = self.emit_branch(&Op::Branch);

        self.patch_branch(else_offset);
        self.chunk.write_op(&Op::Pop);

        self.parse_precedence(Precedence::Or as u8)?;

        self.patch_branch(end_offset);

        Ok(())
    }

    pub(super) fn while_(&mut self, _: bool) -> Result<(), CompileError> {

        self.expression()?;

        let end_offset = self.emit_branch(&Op::BranchIfFalse);
        self.chunk.write_op(&Op::Pop);
        
        self.expression()?;

        self.patch_branch(end_offset);

        self.chunk.write_op(&Op::Pop);

        Ok(())
    }

    pub(super) fn patch_branch(&mut self, offset: usize) {
        let size = size_of::<usize>();
        let distance = self.chunk.len() - offset - size_of::<usize>();

        self.chunk.content[offset..offset+size].copy_from_slice(&usize::to_ne_bytes(distance));
    }

    pub fn compile(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        while !self.match_type(TokenType::EndOfFile)? {
            while let Ok(_) = self.consume(TokenType::EndOfLine) {}
            self.expression()?;
            match self.consume(TokenType::EndOfLine) {
                Ok(_) => {}
                Err(_) => self.consume(TokenType::EndOfFile)?,
            };
            if !self.check(TokenType::EndOfFile) {
                self.chunk.write_op(&Op::Pop);
            }
        }
        self.emit_return();

        if self.debug_output_chunk {
            println!("{}", self.chunk.display());
        }

        Ok(())
    }
}
