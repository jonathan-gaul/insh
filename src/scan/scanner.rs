use super::{
    errors::ScanError,
    tokens::{Token, TokenType},
};

#[derive(Copy, Clone, Debug)]
pub enum ScannerMode {
    Command,
    Expression,
    Argument,
}

pub struct Scanner {
    start_offset: usize,
    current_offset: usize,
    current_column: i64,
    current_line: i64,
    chars: Vec<char>,
    mode: ScannerMode,
    mode_stack: Vec<ScannerMode>,

    debug_output_tokens: bool,
}

impl Scanner {
    pub fn new(text: String) -> Self {
        Scanner {
            start_offset: 0,
            current_offset: 0,
            current_column: 0,
            current_line: 1,
            chars: text.chars().collect(),
            mode: ScannerMode::Command,
            mode_stack: Vec::new(),

            debug_output_tokens: true,
        }
    }

    fn push_mode(&mut self, mode: ScannerMode) {
        // Don't push "Command" mode to stack, as we never
        // want to return to it.
        match self.mode {
            ScannerMode::Command => {}
            _ => {
                self.mode_stack.push(self.mode);
            }
        }
        self.mode = mode;
    }

    fn pop_mode(&mut self) {
        self.mode = self.mode_stack.pop().unwrap_or(ScannerMode::Command);
    }

    fn read_number(&mut self) -> Result<Token, ScanError> {
        let mut dots = 0;

        while !self.is_at_end() && (self.current_char().is_digit(10) || self.current_char() == '.')
        {
            if self.current_char() == '.' {
                dots += 1;
            }
            self.next_char();
        }

        let token_type = match dots {
            0 => TokenType::Int,
            1 => TokenType::Float,
            3 => TokenType::IPv4,
            _ => return Err(ScanError::InvalidNumber),
        };

        Ok(self.new_token(token_type, None, None))
    }

    fn read_string(&mut self) -> Result<Token, ScanError> {
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\n' {
                self.current_line += 1;
            }
            self.next_char();
        }

        if self.is_at_end() {
            Err(ScanError::MissingStringDelimiter)
        } else {
            self.next_char();
            Ok(self.new_token(TokenType::String, Some(1), Some(-1)))
        }
    }

    fn identifier_type(&self) -> TokenType {
        match self.chars[self.start_offset] {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'd' => self.check_keyword(1, "o", TokenType::Do),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if self.current_offset - self.start_offset > 1 {
                    match self.chars[self.start_offset + 1] {
                        'a' => self.check_keyword(2, "lse", TokenType::False),
                        'o' => self.check_keyword(2, "r", TokenType::For),
                        'r' => self.check_keyword(2, "om", TokenType::From),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => {
                if self.current_offset - self.start_offset > 1 {
                    match self.chars[self.start_offset + 1] {
                        'f' => TokenType::If,
                        's' => TokenType::Is,
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'l' => self.check_keyword(1, "et", TokenType::Let),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            'p' => {
                if self.current_offset - self.start_offset > 1 {
                    match self.chars[self.start_offset + 1] {
                        'a' => self.check_keyword(2, "rse", TokenType::Parse),
                        'i' => self.check_keyword(2, "n", TokenType::Pin),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            't' => {
                if self.current_offset - self.start_offset > 1 {
                    match self.chars[self.start_offset + 1] {
                        'h' => self.check_keyword(2, "en", TokenType::Then),
                        'r' => self.check_keyword(2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'u' => self.check_keyword(1, "ntil", TokenType::Until),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn read_identifier(&mut self) -> Result<Token, ScanError> {
        while !self.is_at_end() && self.current_char().is_alphanumeric()
            || self.current_char() == '_'
        {
            self.next_char();
        }

        let token_type = self.identifier_type();
        if matches!(
            token_type,
            TokenType::Do | TokenType::If | TokenType::Then | TokenType::Else
        ) {
            self.push_mode(ScannerMode::Command);
        }

        Ok(self.new_token(token_type, None, None))
    }

    fn read_command(&mut self) -> Result<Token, ScanError> {
        while !self.is_at_end() && !self.current_char().is_whitespace() {
            self.next_char();
        }

        let (mode, token_type) = match self.identifier_type() {
            TokenType::Identifier => (ScannerMode::Argument, TokenType::Command),
            other => (ScannerMode::Expression, other),
        };

        self.push_mode(mode);
        Ok(self.new_token(token_type, None, None))
    }

    fn read_argument(&mut self) -> Result<Token, ScanError> {
        while !self.is_at_end()
            && !self.current_char().is_whitespace()
            && self.current_char() != ')'
        {
            self.next_char();
        }

        let token_type = match self.identifier_type() {
            TokenType::Identifier => TokenType::String,
            TokenType::Is | TokenType::And | TokenType::Or | TokenType::For | TokenType::From | TokenType::Do | TokenType::If | TokenType::Else => TokenType::String,
            other => other,
        };

        if matches!(
            token_type,
            TokenType::Do | TokenType::Then | TokenType::Else
        ) {
            self.push_mode(ScannerMode::Command)
        }

        Ok(self.new_token(token_type, None, None))
    }

    fn read_variable(&mut self) -> Result<Token, ScanError> {
        let token_type = match self.chars[self.current_offset - 1] {
            '$' => TokenType::EnvironmentVariable,
            '@' => TokenType::LocalVariable,
            _ => return Err(ScanError::UnknownVariableType),
        };

        self.next_char();

        while !self.is_at_end()
            && (self.current_char().is_alphabetic()
                || self.current_char().is_alphabetic()
                || self.current_char() == '_')
        {
            self.next_char();
        }

        Ok(self.new_token(token_type, Some(1), Some(0)))
    }

    fn is_at_end(&self) -> bool {
        self.current_offset >= self.chars.len()
    }

    fn next_char(&mut self) -> char {
        let c = self.chars[self.current_offset];
        self.current_offset += 1;
        self.current_column += 1;
        c
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current_offset]
        }
    }

    fn check_keyword(&self, from: isize, rest: &str, token_type: TokenType) -> TokenType {
        let len = rest.len();
        if self.current_offset.wrapping_sub(self.start_offset) == len.wrapping_add_signed(from) {
            let slice = self.chars[self.start_offset.wrapping_add_signed(from)
                ..self.start_offset + len.wrapping_add_signed(from)]
                .iter()
                .collect::<String>();

            if slice == rest {
                return token_type;
            }
        }

        TokenType::Identifier
    }

    fn skip_whitespace(&mut self) {
        while " \r\t#".contains(self.current_char()) {
            if self.current_char() == '#' {
                while !self.is_at_end() && self.current_char() != '\n' {
                    self.next_char();
                }
            }
            self.next_char();
        }

        if self.current_char() == '\n' {
            self.current_line += 1;
            self.current_column += 1;
            self.mode = ScannerMode::Command
        }
    }

    fn new_token(
        &self,
        token_type: TokenType,
        start_offset: Option<isize>,
        end_offset: Option<isize>,
    ) -> Token {
        let start = start_offset.unwrap_or(0);
        let end = end_offset.unwrap_or(0);
        let raw = (&self.chars[self.start_offset.wrapping_add_signed(start)
            ..self.current_offset.wrapping_add_signed(end)])
            .iter()
            .collect();

        if self.debug_output_tokens {
            println!(
                "L{:04} C{:02} ({:04}) {:12} [{}]",
                self.current_line,
                self.current_column,
                self.start_offset,
                format!("[{:02}] {}", token_type as u8, token_type),
                raw
            );
        }

        Token::new(token_type, raw)
    }

    fn token_if_match(
        &mut self,
        c: char,
        match_token: TokenType,
        non_match_token: TokenType,
    ) -> TokenType {
        if !self.is_at_end() && self.chars[self.current_offset] == c {
            self.next_char();
            match_token
        } else {
            non_match_token
        }
    }

    pub fn read_token(&mut self) -> Result<Token, ScanError> {
        self.skip_whitespace();

        self.start_offset = self.current_offset;

        if self.is_at_end() {
            Ok(self.new_token(TokenType::EndOfFile, None, None))
        } else {
            let c = self.next_char();

            loop {
                match (self.mode, c) {
                    (_, '\n') => {
                        let tt = if let ScannerMode::Argument = self.mode {
                            TokenType::EndCommand
                        } else {
                            TokenType::EndOfLine
                        };

                        self.current_line += 1;
                        self.current_column = 0;
                        self.push_mode(ScannerMode::Command);

                        return Ok(self.new_token(tt, None, None));
                    }

                    (_, '#') => while !self.is_at_end() && self.next_char() != '\n' {},

                    (ScannerMode::Command, '(' | '"' | '@' | '$' | '-') => {
                        self.push_mode(ScannerMode::Expression)
                    }
                    (ScannerMode::Command, '{') => {
                        self.push_mode(ScannerMode::Command);
                        return Ok(self.new_token(TokenType::OpenBrace, None, None));
                    }
                    (ScannerMode::Command, '}') => {
                        self.pop_mode();
                        return Ok(self.new_token(TokenType::CloseBrace, None, None));
                    }
                    (ScannerMode::Command, _) => {
                        if c.is_numeric() {
                            self.mode = ScannerMode::Expression;
                        } else {
                            return self.read_command();
                        }
                    }

                    (ScannerMode::Argument, '(') => self.push_mode(ScannerMode::Expression),
                    (ScannerMode::Argument, '"') => return self.read_string(),
                    (ScannerMode::Argument, '|') => self.push_mode(ScannerMode::Expression),
                    (ScannerMode::Argument, '@' | '$') => return self.read_variable(),
                    (ScannerMode::Argument, ')') => {
                        self.pop_mode();
                        return Ok(self.new_token(TokenType::CloseBracket, None, None));
                    }
                    (ScannerMode::Argument, '}') => {
                        self.pop_mode();
                        return Ok(self.new_token(TokenType::CloseBrace, None, None));
                    }
                    (ScannerMode::Argument, _) => return self.read_argument(),

                    (ScannerMode::Expression, '(') => {
                        self.push_mode(ScannerMode::Command);
                        return Ok(self.new_token(TokenType::OpenBracket, None, None));
                    }
                    (ScannerMode::Expression, ')') => {
                        self.pop_mode();
                        return Ok(self.new_token(TokenType::CloseBracket, None, None));
                    }
                    (ScannerMode::Expression, '{') => {
                        self.push_mode(ScannerMode::Command);
                        return Ok(self.new_token(TokenType::OpenBrace, None, None));
                    }
                    (ScannerMode::Expression, '}') => {
                        self.pop_mode();
                        return Ok(self.new_token(TokenType::CloseBrace, None, None));
                    }
                    (ScannerMode::Expression, ',') => {
                        return Ok(self.new_token(TokenType::Comma, None, None))
                    }
                    (ScannerMode::Expression, '-') => {
                        let ttype =
                            self.token_if_match('>', TokenType::MinusGreater, TokenType::Minus);
                        return Ok(self.new_token(ttype, None, None));
                    }
                    (ScannerMode::Expression, '+') => {
                        return Ok(self.new_token(TokenType::Plus, None, None))
                    }
                    (ScannerMode::Expression, '*') => {
                        return Ok(self.new_token(TokenType::Star, None, None))
                    }
                    (ScannerMode::Expression, '/') => {
                        return Ok(self.new_token(TokenType::Slash, None, None))
                    }
                    (ScannerMode::Expression, '?') => {
                        self.push_mode(ScannerMode::Command);
                        let ttype =
                            self.token_if_match('=', TokenType::QuestionEqual, TokenType::Question);
                        return Ok(self.new_token(ttype, None, None));
                    }
                    (ScannerMode::Expression, ':') => {
                        self.push_mode(ScannerMode::Command);
                        return Ok(self.new_token(TokenType::Colon, None, None));
                    }
                    (ScannerMode::Expression, '!') => {
                        let ttype = self.token_if_match('=', TokenType::BangEqual, TokenType::Bang);
                        return Ok(self.new_token(ttype, None, None));
                    }
                    (ScannerMode::Expression, '=') => {
                        self.push_mode(ScannerMode::Command);
                        let ttype = match self.token_if_match(
                            '=',
                            TokenType::EqualEqual,
                            TokenType::Identifier,
                        ) {
                            TokenType::Identifier => {
                                self.token_if_match('>', TokenType::EqualGreater, TokenType::Equal)
                            }
                            other => other,
                        };
                        return Ok(self.new_token(ttype, None, None));
                    }
                    (ScannerMode::Expression, '<') => {
                        let ttype =
                            match self.token_if_match('=', TokenType::LessEqual, TokenType::Less) {
                                TokenType::LessEqual => self.token_if_match(
                                    '>',
                                    TokenType::LessEqualGreater,
                                    TokenType::LessEqual,
                                ),
                                other => other,
                            };
                        return Ok(self.new_token(ttype, None, None));
                    }
                    (ScannerMode::Expression, '>') => {
                        let ttype =
                            self.token_if_match('=', TokenType::GreaterEqual, TokenType::Greater);
                        return Ok(self.new_token(ttype, None, None));
                    }
                    (ScannerMode::Expression, '|') => {
                        self.push_mode(ScannerMode::Command);
                        return Ok(self.new_token(TokenType::Pipe, None, None));
                    }
                    (ScannerMode::Expression, '@' | '$') => return self.read_variable(),
                    (ScannerMode::Expression, '"') => return self.read_string(),
                    (ScannerMode::Expression, _) => {
                        return if c.is_digit(10) {
                            self.read_number()
                        } else if c.is_alphabetic() {
                            self.read_identifier()
                        } else {
                            Err(ScanError::UnrecognisedCharacter)
                        }
                    }
                }
            }
        }
    }
}
