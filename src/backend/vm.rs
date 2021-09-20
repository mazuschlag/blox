use std::fs;
use std::io::{self, BufRead, Write};
use std::ops::{Add, Div, Mul, Sub};

use crate::error::codes::ErrCode;
use crate::frontend::compiler::Compiler;
use crate::DEBUG_TRACE;

use super::chunk::Chunk;
use super::op_code::OpCode;
use super::value::Value;

pub struct Vm {
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            ip: 0,
            stack: vec![],
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
                Err(e) => {
                    return Err(Self::print_and_return_err(
                        ErrCode::RuntimeError,
                        &e.to_string(),
                    ))
                }
            }
        }

        Ok(())
    }

    pub fn run_file(&mut self, path: &String) -> Result<(), ErrCode> {
        let source = fs::read_to_string(path)
            .map_err(|e| Self::print_and_return_err(ErrCode::ScannerError, &e.to_string()))?;
        self.interpret(source)
    }

    pub fn interpret(&mut self, source: String) -> Result<(), ErrCode> {
        let compiled = Compiler::new(source).compile()?;
        self.ip = 0;
        self.run(compiled)
    }

    fn run(&mut self, chunk: Chunk) -> Result<(), ErrCode> {
        while self.ip < chunk.code.len() {
            if DEBUG_TRACE {
                self.stack_trace();
                chunk.disassamble_instruction(self.ip, &chunk.code[self.ip])
            }

            let op_result = match chunk.code[self.ip] {
                OpCode::Return => {
                    match self.stack.pop() {
                        Some(value) => println!("{}", value),
                        None => println!("void"),
                    };
                    Ok(())
                }
                OpCode::Constant(index) => Ok(self.stack.push(chunk.constants.get(index))),
                OpCode::Negate => {
                    let top = self.stack_top();
                    if let Value::Number(n) = self.stack[top] {
                        self.stack[top] = Value::Number(-n);
                        return Ok(());
                    }
                    Err(self.runtime_error("Operand must be a number.", &chunk))
                }
                OpCode::Add => self.binary_op(Add::add),
                OpCode::Subtract => self.binary_op(Sub::sub),
                OpCode::Multiply => self.binary_op(Mul::mul),
                OpCode::Divide => self.binary_op(Div::div),
                OpCode::True => Ok(self.stack.push(Value::Bool(true))),
                OpCode::False => Ok(self.stack.push(Value::Bool(false))),
                OpCode::Nil => Ok(self.stack.push(Value::Nil)),
            };

            if let Err(e) = op_result {
                return Err(Self::print_and_return_err(
                    ErrCode::RuntimeError,
                    &self.runtime_error(&e, &chunk),
                ));
            }

            self.ip += 1;
        }

        Ok(())
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
    where
        F: FnMut(f64, f64) -> f64,
    {
        let right = self.stack.pop();
        let left = self.stack.pop();
        if right.is_none() || left.is_none() {
            return Err(String::from("Not enough values on stack"));
        }

        if let Some(Value::Number(a)) = left {
            if let Some(Value::Number(b)) = right {
                self.stack.push(Value::Number(op(a, b)));
                return Ok(());
            }
        }

        return Err(String::from("Operands must be numbers"));
    }

    fn stack_top(&self) -> usize {
        self.stack.len() - 1
    }

    fn runtime_error(&self, msg: &str, chunk: &Chunk) -> String {
        format!("{}\n[line {}] in script\n", msg, chunk.get_line(self.ip))
    }

    fn stack_trace(&self) {
        print!("           ");
        for index in 0..self.stack.len() {
            print!("[ {} ]", self.stack[index]);
        }

        println!();
    }

    fn print_and_return_err(code: ErrCode, e: &str) -> ErrCode {
        eprintln!("{}", e);
        code
    }
}
