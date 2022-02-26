use std::rc::Rc;

use super::token::Token;
use super::token_type::TokenType;

#[derive(Clone)]
pub struct Local {
    pub name: Rc<Token>,
    pub depth: i32,
    pub dec_type: TokenType,
}

impl Local {
    pub fn new(name: Rc<Token>, dec_type: TokenType) -> Local {
        Local {
            name: name,
            depth: -1,
            dec_type: dec_type,
        }
    }
}
