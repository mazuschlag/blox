use std::rc::Rc;

use crate::backend::chunk::Chunk;
use crate::backend::obj::Obj;
use crate::backend::op_code::OpCode;
use crate::backend::source_str::SourceStr;
use crate::backend::value::Value;
use crate::error::codes::ErrCode;
use crate::DEBUG_PRINT_CODE;

use super::parser::Parser;
use super::precedence::Precedence;
use super::scanner::Scanner;
use super::token::Token;
use super::token_type::TokenType;

pub struct Compiler {
    pub scanner: Scanner,
    parser: Parser,
    pub chunk: Chunk,
    pub objects: Option<Rc<Obj>>,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self {
            scanner: Scanner::new(source),
            parser: Parser::new(),
            chunk: Chunk::new(),
            objects: None,
        }
    }

    pub fn compile(mut self) -> Result<Compiler, ErrCode> {
        self.advance();
        while !self.match_and_advance(TokenType::Eof) {
            self.declaration();
        }
        self.end_compiler()
    }

    pub fn consume(&mut self, typ: TokenType, msg: &str) {
        if self.parser.current_type() == typ {
            self.advance();
            return;
        }

        let token = Rc::clone(&self.parser.current);
        self.error(msg, &token);
    }

    fn end_compiler(mut self) -> Result<Compiler, ErrCode> {
        self.consume(TokenType::Eof, "Expect end of expression");
        if self.parser.had_error {
            return Err(ErrCode::CompileError);
        }

        self.emit_return();
        if DEBUG_PRINT_CODE {
            self.chunk.disassemble("code");
        }

        Ok(self)
    }

    fn advance(&mut self) {
        if self.parser.current_type() != TokenType::Eof {
            self.parser.previous = Rc::clone(&self.parser.current);
            loop {
                self.parser.current = Rc::new(self.scanner.scan_token());
                if self.parser.current_type() != TokenType::Error {
                    break;
                }
                let token = Rc::clone(&self.parser.current);
                self.error(&token.message, &token);
            }
        }
    }

    fn declaration(&mut self) {
        self.statement();
    }

    fn statement(&mut self) {
        if self.match_and_advance(TokenType::Print) {
            self.print_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.prefix_rule(self.parser.previous_type());
        match prefix_rule {
            Some(rule) => rule(self),
            None => {
                let token = Rc::clone(&self.parser.previous);
                self.error("Expect expression", &token)
            }
        }

        while precedence <= self.parser.current_precedence() {
            self.advance();
            if let Some(rule) = self.infix_rule(self.parser.previous_type()) {
                rule(self);
            }
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous_type();
        self.parse_precedence(self.parser.previous_precedence().next());
        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            _ => return,
        };
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous_type();
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => return,
        };
    }

    fn number(&mut self) {
        let number = self.previous_lexeme();
        let value = Rc::new(Value::Number(number.parse::<f64>().unwrap()));
        self.emit_constant(value);
    }

    fn string(&mut self) {
        let next_obj = match &self.objects {
            Some(obj) => Some(Rc::clone(obj)),
            None => None,
        };
        let string = Rc::new(Value::SourceStr(SourceStr::new(
            self.parser.previous.start,
            self.parser.previous.length,
            Rc::clone(&self.scanner.source),
        )));
        self.emit_constant(Rc::clone(&string));
        self.objects = Some(Rc::new(Obj::new(string, next_obj)));
    }

    fn literal(&mut self) {
        match self.parser.previous_type() {
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => {
                let token = Rc::clone(&self.parser.previous);
                let msg = format!("Literal op code should be unreachable for {}", &token.typ);
                self.error(&msg, &token)
            }
        }
    }

    fn prefix_rule(&mut self, typ: TokenType) -> Option<fn(&mut Compiler)> {
        match typ {
            TokenType::LeftParen => Some(|compiler| compiler.grouping()),
            TokenType::Minus => Some(|compiler| compiler.unary()),
            TokenType::Number => Some(|compiler| compiler.number()),
            TokenType::Str => Some(|compiler| compiler.string()),
            TokenType::True | TokenType::False | TokenType::Nil => {
                Some(|compiler| compiler.literal())
            }
            TokenType::Bang => Some(|compiler| compiler.unary()),
            _ => None,
        }
    }

    fn infix_rule(&mut self, typ: TokenType) -> Option<fn(&mut Compiler)> {
        match typ {
            TokenType::Minus
            | TokenType::Plus
            | TokenType::Slash
            | TokenType::Star
            | TokenType::BangEqual
            | TokenType::EqualEqual
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => Some(|compiler| compiler.binary()),
            _ => None,
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn emit_constant(&mut self, value: Rc<Value>) {
        let index = self.make_constant(value);
        self.emit_byte(OpCode::Constant(index));
    }

    fn emit_bytes(&mut self, first: OpCode, second: OpCode) {
        self.chunk.write(first, self.parser.previous.line);
        self.chunk.write(second, self.parser.previous.line);
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn make_constant(&mut self, value: Rc<Value>) -> usize {
        self.chunk.add_constant(value)
    }

    fn match_and_advance(&mut self, typ: TokenType) -> bool {
        if !self.check(typ) {
            return false;
        }

        self.advance();
        true
    }

    fn check(&self, typ: TokenType) -> bool {
        self.parser.current_type() == typ
    }

    fn error(&mut self, msg: &str, token: &Token) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic_mode = true;
        self.parser.had_error = true;

        let lexeme = self.current_lexeme();
        self.error_at(token, lexeme, msg)
    }

    fn error_at(&self, token: &Token, lexeme: String, msg: &str) {
        match token.typ {
            TokenType::Eof => println!("[line {}] Error at end: {}", token.line, msg),
            TokenType::Error => println!("[line {}] Error: {}", token.line, msg),
            _ => println!("[line {}] Error at '{}': {}", token.line, lexeme, msg),
        };
    }

    fn current_lexeme(&self) -> String {
        self.scanner
            .lexeme_at(self.parser.current.start, self.parser.current.length)
            .iter()
            .collect::<String>()
    }

    fn previous_lexeme(&self) -> String {
        self.scanner
            .lexeme_at(self.parser.previous.start, self.parser.previous.length)
            .iter()
            .collect::<String>()
    }
}
