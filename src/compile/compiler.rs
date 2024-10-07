use std::mem::{self, take};

use crate::{
    scan::{
        scanner::Scanner,
        tokens::{Token, TokenType},
    },
    vm::{
        chunk::bytecode_chunk::ByteCodeChunk,
        op::Op,
        value::{fvalue, ivalue, Value},
    },
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
    pub(super) chunk: ByteCodeChunk,

    pub(super) scanner: Scanner,

    pub(super) previous: Token,
    pub(super) current: Token,

    pub(super) debug_output_chunk: bool,

    pub(super) locals: Vec<LocalVariable>,
    pub(super) local_count: usize,
    pub(super) scope_depth: i64,
}

impl Compiler {
    pub fn new(scanner: Scanner, chunk: ByteCodeChunk) -> Self {
        let local = LocalVariable {
            name: Token::new(TokenType::String, "".to_string()),
            depth: 0,
        };
        Compiler {
            chunk,
            scanner,
            previous: Token::new(TokenType::Unknown, String::new()),
            current: Token::new(TokenType::Unknown, String::new()),

            debug_output_chunk: true,

            locals: Vec::from([local]),
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

    pub(super) fn consume(&mut self, token_types: &[TokenType]) -> Result<(), CompileError> {
        if token_types.contains(&self.current.token_type()) {
            self.advance()
        } else {
            Err(CompileError::MissingToken(
                token_types.to_vec(),
                self.current.clone(),
            ))
        }
    }

    pub(super) fn expression(&mut self) -> Result<(), CompileError> {
        self.parse_precedence(Precedence::Assignment as u8)
    }

    pub(super) fn grouping(&mut self, _: bool) -> Result<(), CompileError> {
        self.expression()?;
        self.consume(&[TokenType::CloseBracket])?;
        Ok(())
    }

    pub(super) fn unary(&mut self, _: bool) -> Result<(), CompileError> {
        let operator_type = self.previous.token_type;

        self.parse_precedence(Precedence::Unary as u8)?;

        match operator_type {
            TokenType::Minus => self.emit_op(Op::Negate),
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
            TokenType::Plus => self.emit_op(Op::Add),
            TokenType::Minus => self.emit_op(Op::Subtract),
            TokenType::Star => self.emit_op(Op::Multiply),
            TokenType::Slash => self.emit_op(Op::Divide),
            TokenType::Pipe => self.emit_op(Op::Pipe),
            TokenType::EqualEqual => self.emit_op(Op::Equal),
            _ => {}
        }

        Ok(())
    }

    pub(super) fn block(&mut self, _: bool) -> Result<(), CompileError> {
        self.emit_begin_scope();
        self.begin_scope();
        while let Ok(_) = self.consume(&[TokenType::EndOfLine]) {}
        while !self.check(TokenType::CloseBrace) && !self.check(TokenType::EndOfFile) {
            self.expression()?;
        }
        self.consume(&[TokenType::CloseBrace])?;
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
            self.emit_op(Op::Pop);
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

    pub(super) fn read(&mut self, _: bool) -> Result<(), CompileError> {
        // read <type>  (e.g. number, json, text)
        self.expression()?;

        // from
        self.consume(&[TokenType::From])?;

        // <expr>  (e.g. 'file <x> or "str" etc')
        self.expression()?;

        self.emit_sys_call("read".to_owned());

        Ok(())
    }

    pub(super) fn string_constant(&mut self, _: bool) -> Result<(), CompileError> {
        self.emit_string_constant(self.previous.text.to_owned());
        Ok(())
    }

    pub(super) fn true_literal(&mut self, _: bool) -> Result<(), CompileError> {
        self.emit_bool_constant(true);
        Ok(())
    }

    pub(super) fn false_literal(&mut self, _: bool) -> Result<(), CompileError> {
        self.emit_bool_constant(false);
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
        let env_name = self.previous.text.to_owned();
        let op = if self.match_type(TokenType::Equal)? {
            self.expression()?;
            Op::SetEnv
        } else {
            Op::GetEnv
        };

        self.emit_var(op, &env_name);
        Ok(())
    }

    pub(super) fn function_definition(
        &mut self,
        params: Vec<String>,
        _: bool,
    ) -> Result<(), CompileError> {
        // @param1 @param2 => block
        // or:
        // => block
        self.match_type(TokenType::EqualGreater)?;

        let offset = self.emit_branch(Op::Branch);
        let arity = params.len() as u8;

        // Pull arguments from stack
        for identifier in params {
            self.emit_var(Op::DefineLocal, &identifier);
        }

        // Function body
        self.expression()?;

        // Patch our branch.
        self.patch_branch(offset);

        // Now make the function
        self.emit_function(Value::Function(arity, offset));

        Ok(())
    }

    pub(super) fn let_var(&mut self, can_assign: bool) -> Result<(), CompileError> {
        // let @identifier = <expr>
        self.advance()?;
        let identifier = self.previous.text.to_owned();

        // might be a function definition...
        if self.check(TokenType::EqualGreater) || self.check(TokenType::Identifier) {
            let mut params = Vec::new();
            while self.check(TokenType::Identifier) {
                self.advance()?;
                params.push(self.previous.text.clone());
            }
            self.function_definition(params, can_assign)?;
        } else {
            self.match_type(TokenType::Equal)?;
            self.expression()?;
        }
        self.emit_var(Op::DefineLocal, &identifier);
        Ok(())
    }

    pub(super) fn pin_var(&mut self, _: bool) -> Result<(), CompileError> {
        // pin @identifier = <expr>
        self.advance()?;

        let identifier = self.previous.text.to_owned();

        if self.check(TokenType::Equal) {
            // Set var and pin
            self.match_type(TokenType::Equal)?;
            self.expression()?;
        } else {
            // Pin current value
            self.emit_var(Op::GetLocal, &identifier);
        }

        self.emit_var(Op::PinLocal, &identifier);
        Ok(())
    }

    pub(super) fn local_var(&mut self, can_assign: bool) -> Result<(), CompileError> {
        let identifier = self.previous.text.to_owned();

        let is_set: bool = can_assign && self.match_type(TokenType::Equal)?;
        let op = if is_set {
            self.expression()?;
            Op::SetLocal
        } else {
            Op::GetLocal
        };

        self.emit_var(op, &identifier);
        Ok(())
    }

    pub(super) fn if_(&mut self, _: bool) -> Result<(), CompileError> {
        // if <expr>
        self.expression()?;

        // then
        self.consume(&[TokenType::Then])?;

        let offset = self.emit_branch(Op::BranchIfFalse);
        self.emit_op(Op::Pop);

        // <expr>
        self.expression()?;

        let else_offset = self.emit_branch(Op::Branch);

        self.patch_branch(offset);
        self.emit_op(Op::Pop);

        while self.match_type(TokenType::EndOfLine)? {}

        // else
        if self.match_type(TokenType::Else)? {
            self.expression()?;
        } else {
            self.emit_none();
        }

        self.patch_branch(else_offset);
        Ok(())
    }

    pub(super) fn and(&mut self, _: bool) -> Result<(), CompileError> {
        let offset = self.emit_branch(Op::BranchIfFalse);

        self.emit_op(Op::Pop);
        self.parse_precedence(Precedence::And as u8)?;

        self.patch_branch(offset);
        Ok(())
    }

    pub(super) fn or(&mut self, _: bool) -> Result<(), CompileError> {
        let else_offset = self.emit_branch(Op::BranchIfFalse);
        let end_offset = self.emit_branch(Op::Branch);

        self.patch_branch(else_offset);
        self.emit_op(Op::Pop);

        self.parse_precedence(Precedence::Or as u8)?;

        self.patch_branch(end_offset);

        Ok(())
    }

    pub(super) fn current_offset(&self) -> usize {
        self.chunk.len()
    }

    pub(super) fn while_(&mut self, _: bool) -> Result<(), CompileError> {
        let start_offset = self.current_offset();

        self.expression()?;

        let end_offset = self.emit_branch(Op::BranchIfFalse);
        self.emit_op(Op::Pop);

        self.expression()?;

        self.emit_loop(start_offset);

        self.patch_branch(end_offset);

        self.emit_op(Op::Pop);
        Ok(())
    }

    pub(super) fn patch_branch(&mut self, offset: usize) {
        let current_offset = self.current_offset();
        let size = size_of::<usize>();
        let distance = current_offset - offset - size;
        self.chunk.content[offset..offset + size].copy_from_slice(&usize::to_ne_bytes(distance));
    }

    pub fn compile_to(&mut self, tt: TokenType) -> Result<(), CompileError> {
        self.advance()?;

        while !self.match_type(tt)? {
            while let Ok(_) = self.consume(&[TokenType::EndOfLine]) {}
            self.expression()?;
            self.consume(&[
                TokenType::EndOfLine,
                TokenType::EndOfFile,
                TokenType::Semicolon,
                tt,
            ])?;
            if !self.check(tt) {
                self.emit_op(Op::Pop);
            }
        }
        self.emit_return();

        if self.debug_output_chunk {
            println!("{}", self.chunk.display());
        }

        Ok(())
    }
}

impl Into<Scanner> for Compiler {
    fn into(self) -> Scanner {
        self.scanner
    }
}
