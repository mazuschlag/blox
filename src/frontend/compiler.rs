use std::rc::Rc;

use crate::backend::chunk::Chunk;
use crate::error::codes::ErrCode;

use super::scanner::{Scanner, TokenType, Token};

pub struct Compiler {
    scanner: Scanner,
    parser: Parser
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            scanner: Scanner::new(source),
            parser: Parser::new()
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk) -> Result<(), ErrCode> {
        self.advance()?;
        self.expression()?;
        self.consume(TokenType::Eof, "Expect end of expression")
    }

    fn advance(&mut self) -> Result<(), ErrCode> {
        self.parser.previous = Rc::clone(&self.parser.current);
        loop {
            self.parser.current = Rc::new(self.scanner.scan_token());
            if self.parser.current.typ != TokenType::Error {
                break;
            }
            return Err(self.error_at_current());
        }
        
        Ok(())
    }

    fn expression(&mut self) -> Result<(), ErrCode> {
        Ok(())
    }

    fn consume(&mut self, typ: TokenType, msg: &str) -> Result<(), ErrCode> {
        Ok(())
    }

    fn error_at_current(&self) -> ErrCode {
        let lexeme = self.scanner.lexeme_at(self.parser.current.start, self.parser.current.length);
        Compiler::error_at(&self.parser.current, lexeme)
    }

    fn error(&self) -> ErrCode {
        let lexeme = self.scanner.lexeme_at(self.parser.previous.start, self.parser.previous.length);
        Compiler::error_at(&self.parser.previous, lexeme)
    }

    fn error_at(token: &Token, lexeme: &[char]) -> ErrCode {
        ErrCode::CompileError(
            match token.typ { 
                TokenType::Eof => format!("[line {}] Error at end: {}", token.line, token.message),
                TokenType::Error => format!("[line {}] Error: {}", token.line, token.message),
                _ => format!("[line {}] Error at '{}': {}", token.line, lexeme.iter().collect::<String>(), token.message)
            }
        )
    }
}

struct Parser {
    current: Rc<Token>,
    previous: Rc<Token>
}

impl Parser {
    pub fn new() -> Parser {
        let current = Rc::new(Token::new(TokenType::Eof, 0, 0, 0, String::new()));
        let previous = Rc::new(Token::new(TokenType::Eof, 0, 0, 0, String::new()));
        Parser { current, previous }
    }
}