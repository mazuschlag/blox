use std::rc::Rc;

use super::precedence::Precedence;
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
    pub fn new() -> Self {
        let current = Rc::new(Token::new(TokenType::None, 0, 0, 0, String::new()));
        let previous = Rc::new(Token::new(TokenType::None, 0, 0, 0, String::new()));
        Self {
            current,
            previous,
            panic_mode: false,
            had_error: false,
        }
    }

    pub fn previous_type(&self) -> TokenType {
        self.previous.typ
    }

    pub fn previous_precedence(&self) -> Precedence {
        self.previous_type().precedence()
    }

    pub fn current_type(&self) -> TokenType {
        self.current.typ
    }

    pub fn current_precedence(&self) -> Precedence {
        self.current.typ.precedence()
    }
}
