use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::chunk::Chunk;
use crate::backend::obj::Obj;
use crate::backend::op_code::OpCode;
use crate::backend::source_str::SourceStr;
use crate::backend::value::Value;
use crate::error::codes::ErrCode;
use crate::DEBUG_PRINT_CODE;

use super::local::Local;
use super::precedence::Precedence;
use super::scanner::Scanner;
use super::token::Token;
use super::token_type::TokenType;

pub struct Compiler {
    pub scanner: Scanner,
    pub chunk: Chunk,
    pub objects: Option<Rc<Obj>>,
    locals: Vec<Rc<RefCell<Local>>>,
    local_count: usize,
    scope_depth: usize,
    panic_mode: bool,
    had_error: bool,
    declaration_start: TokenType,
    current: Rc<Token>,
    previous: Rc<Token>,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self {
            scanner: Scanner::new(source),
            chunk: Chunk::new(),
            objects: None,
            locals: Vec::new(),
            local_count: 0,
            scope_depth: 0,
            panic_mode: false,
            had_error: false,
            declaration_start: TokenType::None,
            current: Rc::new(Token::new(TokenType::None, 0, 0, 0, String::new())),
            previous: Rc::new(Token::new(TokenType::None, 0, 0, 0, String::new())),
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
        if self.current.typ == typ {
            self.advance();
            return;
        }

        let token = Rc::clone(&self.current);
        self.error(msg, &token);
    }

    fn end_compiler(mut self) -> Result<Compiler, ErrCode> {
        self.consume(TokenType::Eof, "Expect end of expression");
        if self.had_error {
            return Err(ErrCode::CompileError);
        }

        if DEBUG_PRINT_CODE {
            self.chunk.disassemble("code");
        }

        Ok(self)
    }

    fn advance(&mut self) {
        if self.current.typ != TokenType::Eof {
            self.previous = Rc::clone(&self.current);
            loop {
                self.current = Rc::new(self.scanner.scan_token());
                if self.current.typ != TokenType::Error {
                    break;
                }
                let token = Rc::clone(&self.current);
                self.error(&token.message, &token);
            }
        }
    }

    fn declaration(&mut self) {
        self.declaration_start = self.current.typ;
        if self.match_and_advance(TokenType::Var) {
            self.var_declaration();
        } else if self.match_and_advance(TokenType::Val) {
            self.val_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }
    
    fn val_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.", TokenType::Val);
        self.parse_variable_expression(global);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.", TokenType::Var);
        self.parse_variable_expression(global);
    }

    fn parse_variable_expression(&mut self, global: usize) {
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

        if self.match_and_advance(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
            return;
        }

        self.expression_statement();
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while self.local_count > 0
            && self.locals[self.local_count - 1].borrow().depth > self.scope_depth as i32
        {
            self.emit_byte(OpCode::Pop);
            self.local_count -= 1;
        }
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
        let prefix_rule = self.prefix_rule(self.previous.typ, can_assign);
        match prefix_rule {
            Some(rule) => rule(self),
            None => {
                let token = Rc::clone(&self.previous);
                self.error("Expect expression", &token)
            }
        }

        while precedence <= self.current.typ.precedence() {
            self.advance();
            if let Some(rule) = self.infix_rule(self.previous.typ) {
                rule(self);
            }
        }

        if !can_assign && self.match_and_advance(TokenType::Equal) {
            let token = Rc::clone(&self.previous);
            self.error("Invalid assignment target", &token)
        }
    }

    fn parse_variable(&mut self, error_msg: &str, variable_type: TokenType) -> usize {
        self.consume(TokenType::Identifier, error_msg);
        self.declare_variable(variable_type);
        if self.scope_depth > 0 {
            return 0;
        }

        let (index, _) = self.identifier_constant(Rc::clone(&self.previous));
        index
    }

    fn declare_variable(&mut self, variable_type: TokenType) {
        if self.scope_depth == 0 {
            return;
        }

        let name = Rc::clone(&self.previous);
        for i in (0..self.local_count).rev() {
            let local = Rc::clone(&self.locals[i]);
            if local.borrow().depth != -1 && local.borrow().depth < self.scope_depth as i32 {
                break;
            }

            if self.identifiers_equal(&local.borrow().name, &name) {
                self.error(
                    "Already a variable with this name in this scope.",
                    &local.borrow().name,
                );
            }
        }

        self.add_local(name, variable_type);
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        if a.length != b.length {
            return false;
        }

        self.scanner.lexeme(a.start, a.length) == self.scanner.lexeme(b.start, b.length)
    }

    fn add_local(&mut self, name: Rc<Token>, variable_type: TokenType) {
        let local = Rc::new(RefCell::new(Local::new(name, variable_type)));
        self.locals.push(local);
        self.local_count += 1;
    }

    fn define_variable(&mut self, global: usize) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_byte(OpCode::DefGlobal(global));
    }

    fn mark_initialized(&mut self) {
        let local = Rc::clone(&self.locals[self.local_count - 1]);
        local.borrow_mut().depth = self.scope_depth as i32;
    }

    fn identifier_constant(&mut self, name: Rc<Token>) -> (usize, TokenType) {
        let lexeme = self.scanner.lexeme(name.start, name.length);
        match self.chunk.find_identifier(&lexeme) {
            Some((index, value)) => match *value {
                Value::ValIdent(_) => (index, TokenType::Val),
                _ => (index, TokenType::Var)
            },
            None => {
                let value = match self.declaration_start {
                    TokenType::Val => Rc::new(Value::ValIdent(lexeme)),
                    _ => Rc::new(Value::VarIdent(lexeme)),
                };
                (self.make_constant(value), self.declaration_start)
            },
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn binary(&mut self) {
        let operator_type = self.previous.typ;
        self.parse_precedence(self.previous.typ.precedence().next());
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
        let operator_type = self.previous.typ;
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => return,
        };
    }

    fn number(&mut self) {
        let number = self
            .scanner
            .lexeme(self.previous.start, self.previous.length);
        let value = Rc::new(Value::Number(number.parse::<f64>().unwrap()));
        self.emit_constant(value);
    }

    fn string(&mut self) {
        let next_obj = match &self.objects {
            Some(obj) => Some(Rc::clone(obj)),
            None => None,
        };
        let string = Rc::new(Value::SourceStr(SourceStr::new(
            self.previous.start,
            self.previous.length,
            Rc::clone(&self.scanner.source),
        )));
        self.emit_constant(Rc::clone(&string));
        self.objects = Some(Rc::new(Obj::new(string, next_obj)));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(Rc::clone(&self.previous), can_assign);
    }

    fn named_variable(&mut self, name: Rc<Token>, can_assign: bool) {
        let variable = Rc::clone(&name);
        let arg = self.resolve_local(&variable);
        let (get_op, set_op, dec_type) = match arg {
            Some((index, dec_type)) => (OpCode::GetLocal(index), OpCode::SetLocal(index), dec_type),
            None => {
                let (index, dec_type) = self.identifier_constant(name);
                (OpCode::GetGlobal(index), OpCode::SetGlobal(index), dec_type)
            }
        };

        if dec_type == TokenType::Val && self.declaration_start != TokenType::Val && self.check(TokenType::Equal) {
            self.error("Cannot reassign to value.", &variable);
        }

        if can_assign && self.match_and_advance(TokenType::Equal) {
            self.expression();
            self.emit_byte(set_op);
            return;
        }

        self.emit_byte(get_op);
    }

    fn resolve_local(&mut self, name: &Token) -> Option<(usize, TokenType)> {
        for i in (0..self.local_count).rev() {
            let local = Rc::clone(&self.locals[i]);
            if self.identifiers_equal(&local.borrow().name, name) {
                if local.borrow().depth == -1 {
                    self.error("Can't read local variable in its own initializer.", name);
                }

                return Some((i, (local.borrow().dec_type)));
            }
        }

        None
    }

    fn literal(&mut self) {
        match self.previous.typ {
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => {
                let token = Rc::clone(&self.previous);
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
        self.chunk.write(first, self.previous.line);
        self.chunk.write(second, self.previous.line);
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.chunk.write(byte, self.previous.line);
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
        self.current.typ == typ
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current.typ != TokenType::Eof {
            if self.previous.typ == TokenType::SemiColon {
                return;
            }

            match self.current.typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::Val
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
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        self.had_error = true;

        let lexeme = self.scanner.lexeme(token.start, token.length);
        self.error_at(token, lexeme, msg)
    }

    fn error_at(&self, token: &Token, lexeme: String, msg: &str) {
        match token.typ {
            TokenType::Eof => println!("[line {}] Error at end: {}", token.line, msg),
            TokenType::Error => println!("[line {}] Error: {}", token.line, msg),
            _ => println!("[line {}] Error at '{}': {}", token.line, lexeme, msg),
        };
    }
}
