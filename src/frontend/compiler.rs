use std::rc::Rc;

use crate::backend::chunk::Chunk;
use crate::error::codes::ErrCode;

use super::scanner::{Scanner, Token, TokenType};

pub struct Compiler {
    scanner: Scanner,
    parser: Parser,
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            scanner: Scanner::new(source),
            parser: Parser::new(),
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk) -> Result<(), ErrCode> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression");
        if self.parser.had_error {
            return Err(ErrCode::RuntimeError);
        }
        Ok(())
    }

    fn advance(&mut self) {
        self.parser.previous = Rc::clone(&self.parser.current);
        loop {
            self.parser.current = Rc::new(self.scanner.scan_token());
            if self.parser.current.typ != TokenType::Error {
                break;
            }
            self.error_at_current();
        }
    }

    fn expression(&mut self) {}

    fn consume(&mut self, typ: TokenType, msg: &str) {}

    fn error_at_current(&mut self) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic_mode = true;
        self.parser.had_error = true;

        let lexeme = self
            .scanner
            .lexeme_at(self.parser.current.start, self.parser.current.length)
            .iter().collect::<String>();
        self.error_at(&self.parser.current, lexeme)
    }

    fn error(&mut self) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic_mode = true;
        self.parser.had_error = true;

        let lexeme = self
            .scanner
            .lexeme_at(self.parser.previous.start, self.parser.previous.length)
            .iter().collect::<String>();
        self.error_at(&self.parser.previous, lexeme)
    }

    fn error_at(& self, token: &Token, lexeme: String) {

        println!(
            "{}",
            match token.typ {
                TokenType::Eof => format!("[line {}] Error at end: {}", token.line, token.message),
                TokenType::Error => format!("[line {}] Error: {}", token.line, token.message),
                _ => format!(
                    "[line {}] Error at '{}': {}",
                    token.line,
                    lexeme,
                    token.message
                ),
            }
        );
        
    }
}

struct Parser {
    current: Rc<Token>,
    previous: Rc<Token>,
    panic_mode: bool,
    had_error: bool,
}

impl Parser {
    pub fn new() -> Parser {
        let current = Rc::new(Token::new(TokenType::Eof, 0, 0, 0, String::new()));
        let previous = Rc::new(Token::new(TokenType::Eof, 0, 0, 0, String::new()));
        Parser {
            current,
            previous,
            panic_mode: false,
            had_error: true,
        }
    }
}
