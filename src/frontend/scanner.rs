use super::token::Token;
use super::token_type::TokenType;

#[derive(Debug)]
pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        if c.is_alphabetic() {
            return self.identifier_token();
        }

        if c.is_digit(10) {
            return self.number_token();
        }

        match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::SemiColon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                let typ = if self.check('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(typ)
            }
            '=' => {
                let typ = if self.check('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(typ)
            }
            '<' => {
                let typ = if self.check('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(typ)
            }
            '>' => {
                let typ = if self.check('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(typ)
            }
            '"' => self.string_token(),
            _ => self.error_token("Unexpected character."),
        }
    }

    pub fn lexeme_at(&self, start: usize, length: usize) -> &[char] {
        return &self.source[start..(start + length)];
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                '\t' | ' ' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                }
                '/' => {
                    if self.peek_next() != '/' {
                        return;
                    }
                    self.skip_comments();
                }
                _ => return,
            }
        }
    }

    fn skip_comments(&mut self) {
        while self.check_comment() {
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        }
    }

    fn identifier_token(&mut self) -> Token {
        while !self.is_at_end() && self.peek().is_alphanumeric() {
            self.advance();
        }
        let typ = self.identifier_type();
        self.make_token(typ)
    }

    fn number_token(&mut self) -> Token {
        while !self.is_at_end() && self.peek().is_digit(10) {
            self.advance();
        }

        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance(); // consume the '.'.
            while !self.is_at_end() && self.peek().is_digit(10) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string_token(&mut self) -> Token {
        self.start += 1;
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // Closing quote
        self.advance();
        self.make_token(TokenType::Str)
    }

    fn error_token(&self, message: &str) -> Token {
        Token::new(
            TokenType::Error,
            self.start,
            self.current - self.start,
            self.line,
            message.to_string(),
        )
    }

    fn make_token(&self, typ: TokenType) -> Token {
        let length = match typ {
            TokenType::Str => self.current - self.start - 1,
            _ => self.current - self.start,
        };
        Token::new(typ, self.start, length, self.line, String::new())
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self.source[self.current - 1];
    }

    fn check(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.advance();
        true
    }

    fn check_comment(&mut self) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek() != '/' {
            return false;
        }

        if self.peek_next() != '/' {
            return false;
        }

        self.current += 2;
        true
    }

    fn peek(&self) -> char {
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn identifier_type(&self) -> TokenType {
        match self.source[self.start] {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            'f' => {
                if self.current - self.start == 1 {
                    return TokenType::Identifier;
                }
                match self.source[self.start + 1] {
                    'a' => self.check_keyword(1, 4, "alse", TokenType::False),
                    'o' => self.check_keyword(1, 2, "or", TokenType::For),
                    'u' => self.check_keyword(1, 2, "un", TokenType::Fun),
                    _ => TokenType::Identifier,
                }
            }
            't' => {
                if self.current - self.start == 1 {
                    return TokenType::Identifier;
                }
                match self.source[self.start + 1] {
                    'h' => self.check_keyword(1, 3, "his", TokenType::This),
                    'r' => self.check_keyword(1, 3, "rue", TokenType::True),
                    _ => TokenType::Identifier,
                }
            }
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, typ: TokenType) -> TokenType {
        if self.current - self.start == start + length {
            let source_value = &self.source[self.start + 1..self.current];
            let rest_value: Vec<char> = rest.chars().collect();
            if source_value == &rest_value {
                return typ;
            }
        }

        TokenType::Identifier
    }
}
