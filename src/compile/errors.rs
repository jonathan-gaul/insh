use crate::scan::{
    errors::ScanError,
    tokens::{Token, TokenType},
};

#[derive(Debug)]
pub enum CompileError {
    Unknown,
    NotImplemented,
    MissingToken(Vec<TokenType>, Token),
    UnknownUnaryOperator,
    MissingFrom,
    ParseError(),
    ScanError(ScanError),
    TooManyLocals,
    InvalidAssignment,
}
