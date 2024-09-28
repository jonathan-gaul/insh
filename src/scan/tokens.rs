use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Unknown,

    EndOfFile,

    Let,
    Pin,
    LocalVariable,
    EnvironmentVariable,

    Dot,
    Minus,
    Plus,
    Slash,
    Star,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Comma,

    Bang,
    BangEqual,
    Equal,
    QuestionEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    EqualGreater,
    LessEqualGreater,
    MinusGreater,

    Question,
    Colon,
    Semicolon,
    Pipe,

    Is,

    EndOfLine,

    // Literal values
    Command,
    Identifier,
    String,
    Float,
    Int,
    IPv4,
    Path,
    False,
    True,
    EndCommand,

    // Keywords
    If,
    Then,
    Else,
    While,
    Until,
    Do,
    And,
    Or,
    For,
    Read,
    From,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub raw: Vec<char>,
    pub start: i64,
    pub end: i64,
    pub text: String,
}

impl Token {
    pub fn new(token_type: TokenType, text: String) -> Self {
        Token {
            token_type,
            raw: text.chars().collect(),
            text,
            start: 0,
            end: 0,
        }
    }

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::Unknown => "Unknown",
                TokenType::Command => "Command",
                TokenType::EndOfFile => "EndOfFile",
                TokenType::Dot => "Dot",
                TokenType::Minus => "Minus",
                TokenType::Plus => "Plus",
                TokenType::Slash => "Slash",
                TokenType::Star => "Star",
                TokenType::And => "And",
                TokenType::Or => "Or",
                TokenType::OpenBracket => "OpenBracket",
                TokenType::CloseBracket => "CloseBracket",
                TokenType::OpenBrace => "OpenBrace",
                TokenType::CloseBrace => "CloseBrace",
                TokenType::Comma => "Comma",
                TokenType::Bang => "Bang",
                TokenType::BangEqual => "BangEqual",
                TokenType::Equal => "Equal",
                TokenType::QuestionEqual => "QuestionEqual",
                TokenType::EqualEqual => "EqualEqual",
                TokenType::Greater => "Greater",
                TokenType::GreaterEqual => "GreaterEqual",
                TokenType::Less => "Less",
                TokenType::LessEqual => "LessEqual",
                TokenType::Identifier => "Identifier",
                TokenType::String => "String",
                TokenType::Float => "Float",
                TokenType::Int => "Int",
                TokenType::If => "If",
                TokenType::Then => "Then",
                TokenType::Else => "Else",
                TokenType::IPv4 => "IPv4",
                TokenType::LocalVariable => "Local",
                TokenType::EnvironmentVariable => "Env",
                TokenType::Path => "Path",
                TokenType::Question => "Question",
                TokenType::Colon => "Colon",
                TokenType::Pipe => "Pipe",
                TokenType::EqualGreater => "EqualGreater",
                TokenType::LessEqualGreater => "LessEqualGreater",
                TokenType::While => "While",
                TokenType::Until => "Until",
                TokenType::Do => "Do",
                TokenType::Is => "Is",
                TokenType::For => "For",
                TokenType::True => "True",
                TokenType::False => "False",
                TokenType::Read => "Read",
                TokenType::From => "From",
                TokenType::Let => "Let",
                TokenType::Pin => "Pin",
                TokenType::MinusGreater => "MinusGreater",
                TokenType::EndOfLine => "EndOfLine",
                _ => "UnknownToken",
            }
        )
    }
}
