use super::token_type::TokenType;

#[derive(Debug)]
pub struct Token {
    pub typ: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
    pub message: String,
}

impl Token {
    pub fn new(typ: TokenType, start: usize, length: usize, line: usize, message: String) -> Self {
        Self {
            typ,
            start,
            length,
            line,
            message,
        }
    }
}
