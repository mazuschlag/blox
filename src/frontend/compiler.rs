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
        if self.match_and_advance(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.match_and_advance(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration",
        );
        self.define_variable(global);
    }

    fn statement(&mut self) {
        if self.match_and_advance(TokenType::Print) {
            self.print_statement();
            return;
        }

        self.expression_statement();
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let can_assign = precedence <= Precedence::Assignment;
        let prefix_rule = self.prefix_rule(self.parser.previous_type(), can_assign);
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

        if !can_assign && self.match_and_advance(TokenType::Equal) {
            let token = Rc::clone(&self.parser.previous);
            self.error("Invalid assignment target", &token)
        }
    }

    fn parse_variable(&mut self, error_msg: &str) -> usize {
        self.consume(TokenType::Identifier, error_msg);
        self.identifier_constant(self.previous_lexeme())
    }

    fn define_variable(&mut self, global: usize) {
        self.emit_byte(OpCode::DefGlobal(global));
    }

    fn identifier_constant(&mut self, name: String) -> usize {
        match self.chunk.find_identifier(&name) {
            Some(index) => index,
            None => self.make_constant(Rc::new(Value::Ident(name))),
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

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous_lexeme(), can_assign);
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        let arg = self.identifier_constant(name);
        if can_assign && self.match_and_advance(TokenType::Equal) {
            self.expression();
            self.emit_byte(OpCode::SetGlobal(arg));
            return;
        }

        self.emit_byte(OpCode::GetGlobal(arg));
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

    fn prefix_rule(
        &mut self,
        typ: TokenType,
        can_assign: bool,
    ) -> Option<Box<dyn Fn(&mut Compiler)>> {
        match typ {
            TokenType::LeftParen => Some(Box::new(|compiler: &mut Compiler| compiler.grouping())),
            TokenType::Minus => Some(Box::new(|compiler: &mut Compiler| compiler.unary())),
            TokenType::Number => Some(Box::new(|compiler: &mut Compiler| compiler.number())),
            TokenType::Str => Some(Box::new(|compiler: &mut Compiler| compiler.string())),
            TokenType::True | TokenType::False | TokenType::Nil => {
                Some(Box::new(|compiler: &mut Compiler| compiler.literal()))
            }
            TokenType::Bang => Some(Box::new(|compiler: &mut Compiler| compiler.unary())),
            TokenType::Identifier => Some(Box::new(move |compiler: &mut Compiler| {
                compiler.variable(can_assign)
            })),
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

    #[allow(dead_code)]
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

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;
        while self.parser.current_type() != TokenType::Eof {
            if self.parser.previous_type() == TokenType::SemiColon {
                return;
            }

            match self.parser.current_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
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
