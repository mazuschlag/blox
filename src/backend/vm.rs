use std::fs;
use std::io::{self, BufRead, Write};
use std::ops::{Add, Div, Mul, Sub};

use crate::error::codes::ErrCode;
use crate::frontend::compiler::Compiler;

use super::chunk::Chunk;
use super::chunk::OpCode;
use super::value::Value;

pub struct Vm {
    ip: usize,
    stack: Vec<Value>,
    debug_trace: bool,
}

impl Vm {
    pub fn new(debug_trace: bool) -> Vm {
        Vm {
            ip: 0,
            stack: vec![],
            debug_trace,
        }
    }

    pub fn repl(&mut self) -> Result<(), ErrCode> {
        println!("=== Welcome to blox v1.0");
        println!("=== Enter 'q' or 'Q' to quit");
        print!("> ");
        io::stdout().flush().unwrap();
        for line in io::stdin().lock().lines() {
            match line {
                Ok(input) => {
                    if input.trim() == "Q" || input.trim() == "q" {
                        println!("=== Goodbye!");
                        return Ok(());
                    }
                    self.interpret(input)?;
                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => return Err(Vm::print_and_return_err(ErrCode::RuntimeError, &e.to_string())),
            }
        }
        Ok(())
    }

    pub fn run_file(&mut self, path: &String) -> Result<(), ErrCode> {
        let source = fs::read_to_string(path)
            .map_err(|e| Vm::print_and_return_err(ErrCode::ScannerError, &e.to_string()))?;
        self.interpret(source)
    }

    pub fn interpret(&mut self, source: String) -> Result<(), ErrCode> {
        let mut chunk = Chunk::new();
        let compiled = Compiler::new(source).compile(&mut chunk)?;
        self.ip = 0;
        self.run(chunk)
    }

    fn run(&mut self, chunk: Chunk) -> Result<(), ErrCode> {
        while self.ip < chunk.code.len() {
            if self.debug_trace {
                self.stack_trace();
                chunk.disassamble_instruction(self.ip, &chunk.code[self.ip])
            }
            match chunk.code[self.ip] {
                OpCode::Return => match self.stack.pop() {
                    Some(value) => println!("{}", value),
                    None => println!("void"),
                },
                OpCode::Constant(index) => self.stack.push(chunk.constants.get(index)),
                OpCode::Negate => {
                    let top = self.stack.len() - 1;
                    self.stack[top] = -self.stack[top];
                }
                OpCode::Add => {
                    if let Err(e) = self.binary_op(Add::add) {
                        return Err(Vm::print_and_return_err(ErrCode::RuntimeError, &e));
                    }
                }
                OpCode::Subtract => {
                    if let Err(e) = self.binary_op(Sub::sub) {
                        return Err(Vm::print_and_return_err(ErrCode::RuntimeError, &e));
                    }
                }
                OpCode::Multiply => {
                    if let Err(e) = self.binary_op(Mul::mul) {
                        return Err(Vm::print_and_return_err(ErrCode::RuntimeError, &e));
                    }
                }
                OpCode::Divide => {
                    if let Err(e) = self.binary_op(Div::div) {
                        return Err(Vm::print_and_return_err(ErrCode::RuntimeError, &e));
                    }
                }
            }
            self.ip += 1;
        }
        Ok(())
    }

    fn stack_trace(&self) {
        print!("          ");
        for index in 0..self.stack.len() {
            print!("[ {} ]", self.stack[index]);
        }
        println!();
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
    where
        F: FnMut(Value, Value) -> Value,
    {
        let right = self.stack.pop();
        let left = self.stack.pop();
        if let Some(a) = left {
            if let Some(b) = right {
                self.stack.push(op(a, b));
                return Ok(());
            }
        }
        return Err(String::from("Not enough values on stack"));
    }

    fn print_and_return_err(code: ErrCode, e: &str) -> ErrCode {
        eprintln!("{}", e);
        code
    }
}
