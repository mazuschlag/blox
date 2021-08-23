use std::rc::Rc;

use super::token::Token;
use super::token_type::TokenType;

#[derive(Debug)]
pub struct Parser {
    pub current: Rc<Token>,
    pub previous: Rc<Token>,
    pub panic_mode: bool,
    pub had_error: bool,
}

impl Parser {
    pub fn new() -> Parser {
        let current = Rc::new(Token::new(TokenType::None, 0, 0, 0, String::new()));
        let previous = Rc::new(Token::new(TokenType::None, 0, 0, 0, String::new()));
        Parser {
            current,
            previous,
            panic_mode: false,
            had_error: false,
        }
    }
}
