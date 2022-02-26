use std::rc::Rc;

use super::token::Token;

#[derive(Clone)]
pub struct Local {
    pub name: Rc<Token>,
    pub depth: i32,
}

impl Local {
    pub fn new(name: Rc<Token>) -> Local {
        Local {
            name: name,
            depth: -1,
        }
    }
}
