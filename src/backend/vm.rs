use std::borrow::Borrow;
use std::fs;
use std::io::{self, BufRead, Write};
use std::rc::Rc;
use std::str;

use crate::error::codes::ErrCode;
use crate::frontend::compiler::Compiler;
use crate::DEBUG_TRACE;

use super::chunk::Chunk;
use super::obj::Obj;
use super::op_code::OpCode;
use super::value::Value;

pub struct Vm {
    ip: usize,
    stack: Vec<Rc<Value>>,
    objects: Option<Rc<Obj>>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            objects: None,
        }
    }

    pub fn repl(&mut self) -> bool {
        println!("=== Welcome to blox v1.0");
        println!("=== Enter 'q' or 'Q' to quit");
        print!("> ");
        io::stdout().flush().unwrap();
        for line in io::stdin().lock().lines() {
            match line {
                Ok(input) => {
                    if input.to_lowercase().trim() == "q" {
                        println!("=== Goodbye!");
                        return true;
                    }
                    self.interpret(input);
                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    Self::print_and_return_err(ErrCode::RuntimeError, &e.to_string());
                }
            }
        }

        true
    }

    pub fn run_file(&mut self, path: &String) -> bool {
        fs::read_to_string(path)
            .map_err(|e| Self::print_and_return_err(ErrCode::ScannerError, &e.to_string()))
            .map(|source| self.interpret(source))
            .is_err()
    }

    pub fn interpret(&mut self, source: String) -> bool {
        Compiler::new(source)
            .compile()
            .map(|compiler| {
                self.ip = 0;
                self.objects = compiler.objects;
                self.run(compiler.chunk)
            })
            .is_err()
    }

    fn run(&mut self, chunk: Chunk) -> Result<(), ErrCode> {
        while self.ip < chunk.code.len() {
            if DEBUG_TRACE {
                self.stack_trace();
                chunk.disassamble_instruction(self.ip, &chunk.code[self.ip])
            }

            let op_result = match chunk.code[self.ip] {
                OpCode::Return => Ok(()),
                OpCode::Constant(index) => Ok(self.stack.push(chunk.constants.get(index))),
                OpCode::Negate => {
                    let top = self.stack_top();
                    match self.stack[top].borrow() {
                        Value::Number(n) => Ok(self.stack[top] = Rc::new(Value::Number(-n))),
                        _ => Err(self.runtime_error("Operand must be a number", &chunk)),
                    }
                }
                OpCode::Add => self.stack.pop().zip(self.stack.pop()).map_or(
                    Err(String::from("Not enough values on the stack")),
                    |(a, b)| match (a.borrow(), b.borrow()) {
                        (Value::SourceStr(b_str), Value::SourceStr(a_str)) => {
                            self.concat_strings(&a_str.to_string(), &b_str.to_string());
                            Ok(())
                        }
                        (Value::SourceStr(src_str), Value::Str(str_obj)) => {
                            self.concat_strings(&str_obj, &src_str.to_string());
                            Ok(())
                        }
                        (Value::Str(str_obj), Value::SourceStr(src_str)) => {
                            self.concat_strings(&src_str.to_string(), &str_obj);
                            Ok(())
                        }
                        (Value::Str(b), Value::Str(a)) => {
                            self.concat_strings(&a, &b);
                            Ok(())
                        }
                        (Value::Number(b), Value::Number(a)) => {
                            Ok(self.stack.push(Rc::new(Value::Number(a + b))))
                        }
                        _ => Err(String::from("Operands must be two numbers or two strings")),
                    },
                ),
                OpCode::Subtract => self.binary_op(|left, right| Value::Number(left - right)),
                OpCode::Multiply => self.binary_op(|left, right| Value::Number(left * right)),
                OpCode::Divide => self.binary_op(|left, right| Value::Number(left / right)),
                OpCode::True => Ok(self.stack.push(Rc::new(Value::Bool(true)))),
                OpCode::False => Ok(self.stack.push(Rc::new(Value::Bool(false)))),
                OpCode::Nil => Ok(self.stack.push(Rc::new(Value::Nil))),
                OpCode::Not => self
                    .is_falsey()
                    .map(|value| self.stack.push(Rc::new(Value::Bool(value)))),
                OpCode::Equal => self.stack.pop().zip(self.stack.pop()).map_or(
                    Err(String::from("Not enough values on the stack")),
                    |(a, b)| match (a.borrow(), b.borrow()) {
                        (Value::Number(b), Value::Number(a)) => {
                            Ok(self.stack.push(Rc::new(Value::Bool(a == b))))
                        }
                        (Value::Str(b), Value::Str(a)) => {
                            Ok(self.stack.push(Rc::new(Value::Bool(a == b))))
                        }
                        (Value::Bool(b), Value::Bool(a)) => {
                            Ok(self.stack.push(Rc::new(Value::Bool(a == b))))
                        }
                        _ => Err(String::from(
                            "Operands must be two numbers, two strings, or two booleans",
                        )),
                    },
                ),
                OpCode::Greater => self.binary_op(|left, right| Value::Bool(left > right)),
                OpCode::Less => self.binary_op(|left, right| Value::Bool(left < right)),
                OpCode::Print => self.print_value(),
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
        F: FnMut(f64, f64) -> Value,
    {
        self.stack.pop().zip(self.stack.pop()).map_or(
            Err(String::from("Not enough values on the stack")),
            |(b, a)| match (a.borrow(), b.borrow()) {
                (Value::Number(b), Value::Number(a)) => Ok(self.stack.push(Rc::new(op(*a, *b)))),
                _ => Err(String::from("Operand must be a number")),
            },
        )
    }

    fn print_value(&mut self) -> Result<(), String> {
        match self.stack.pop() {
            Some(value) => {
                println!("{}", value);
                Ok(())
            }
            None => Err(String::from("Not enough values on the stack")),
        }
    }

    fn concat_strings(&mut self, a: &String, b: &String) {
        let next_obj = match &self.objects {
            Some(obj) => Some(Rc::clone(obj)),
            None => None,
        };

        let string = Rc::new(Value::Str(format!("{}{}", a, b)));
        self.stack.push(Rc::clone(&string));
        self.objects = Some(Rc::new(Obj::new(string, next_obj)));
    }

    fn is_falsey(&mut self) -> Result<bool, String> {
        let value = self.stack.pop();
        if let Some(v) = value {
            return Ok(v.is_falsey());
        }

        Err(String::from("Not enough values on stack"))
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
