use crate::scan::tokens::TokenType;

use super::{compiler::Compiler, errors::CompileError};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,

    Invalid,
}

impl Precedence {
    pub fn from(v: u8) -> Self {
        match v {
            x if x == Precedence::None as u8 => Precedence::None,
            x if x == Precedence::Assignment as u8 => Precedence::Assignment,
            x if x == Precedence::Or as u8 => Precedence::Or,
            x if x == Precedence::And as u8 => Precedence::And,
            x if x == Precedence::Equality as u8 => Precedence::Equality,
            x if x == Precedence::Comparison as u8 => Precedence::Comparison,
            x if x == Precedence::Term as u8 => Precedence::Term,
            x if x == Precedence::Factor as u8 => Precedence::Factor,
            x if x == Precedence::Unary as u8 => Precedence::Unary,
            x if x == Precedence::Call as u8 => Precedence::Call,
            x if x == Precedence::Primary as u8 => Precedence::Primary,
            _ => Precedence::Invalid,
        }
    }
}

pub type ParseFn = fn(&mut Compiler, bool) -> Result<(), CompileError>;

pub struct ParseRule {}

impl ParseRule {
    pub(super) fn prefix_for(token_type: TokenType) -> Option<ParseFn> {
        match token_type {
            TokenType::OpenBracket => Some(Compiler::grouping),
            TokenType::Minus => Some(Compiler::unary),
            TokenType::Identifier | TokenType::String => Some(Compiler::string_constant),
            TokenType::Int => Some(Compiler::int_constant),
            TokenType::Float => Some(Compiler::float_constant),
            TokenType::Command => Some(Compiler::command),
            TokenType::True => Some(Compiler::true_literal),
            TokenType::False => Some(Compiler::false_literal),
            TokenType::Parse => Some(Compiler::parse),
            TokenType::EnvironmentVariable => Some(Compiler::env_var),
            TokenType::LocalVariable => Some(Compiler::local_var),
            TokenType::OpenBrace => Some(Compiler::block),
            TokenType::Let => Some(Compiler::let_var),
            TokenType::Pin => Some(Compiler::pin_var),
            TokenType::If => Some(Compiler::if_),
            TokenType::While => Some(Compiler::while_),
            _ => None,
        }
    }

    pub(super) fn infix_for(token_type: TokenType) -> Option<ParseFn> {
        match token_type {
            TokenType::Minus => Some(Compiler::binary),
            TokenType::Plus => Some(Compiler::binary),
            TokenType::Slash => Some(Compiler::binary),
            TokenType::Star => Some(Compiler::binary),
            TokenType::Pipe => Some(Compiler::binary),            
            TokenType::And => Some(Compiler::and),
            TokenType::Or => Some(Compiler::or),
            _ => None,
        }
    }

    pub(super) fn precedence_for(token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::Minus | TokenType::Plus => Precedence::Term,
            TokenType::Slash | TokenType::Star => Precedence::Factor,
            TokenType::Equal | TokenType::QuestionEqual => Precedence::Assignment,
            TokenType::Pipe => Precedence::Primary,
            TokenType::And => Precedence::And,
            _ => Precedence::None,
        }
    }
}

impl Compiler {
    pub(super) fn parse_precedence(&mut self, precedence: u8) -> Result<(), CompileError> {
        self.advance()?;

        let can_assign = precedence <= Precedence::Assignment as u8;

        if let Some(prefix) = ParseRule::prefix_for(self.previous.token_type) {
            prefix(self, can_assign)?;
        } else {
            return Ok(());
        }

        loop {
            let infix_precedence = ParseRule::precedence_for(self.current.token_type);
            if precedence > infix_precedence as u8 {
                break;
            }

            self.advance()?;
            if let Some(infix) = ParseRule::infix_for(self.previous.token_type) {
                infix(self, can_assign)?;
            }
        }

        if can_assign && self.match_type(TokenType::Equal)? {
            Err(CompileError::InvalidAssignment)
        } else {
            Ok(())
        }
    }
}
