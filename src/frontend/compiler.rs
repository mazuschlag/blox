use super::scanner::Scanner;
use super::scanner::TokenType;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Compiler {
        Compiler
    }

    pub fn compile(&self, source: String) {
        let mut line = 0;
        let mut scanner = Scanner::new(source);
        loop {
            let token = scanner.scan_token();
            if token.line != line {
                print!("{number:>width$} ", number = token.line, width = 5);
                line = token.line;
            } else {
                print!("    | ");
            }
            println!("{} '{}.{}'", token.typ, token.length, token.start);

            if token.typ == TokenType::Eof {
                return;
            }
        }
    }
}
