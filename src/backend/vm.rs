use std::borrow::Borrow;
use std::collections::HashMap;
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
    objects: Option<Box<Obj>>,
    globals: HashMap<String, Rc<Value>>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            objects: None,
            globals: HashMap::new(),
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
                    if input.to_lowercase().trim() == "q" {
                        println!("=== Goodbye!");
                        return Ok(());
                    }

                    if let Err(ErrCode::Runtime(e)) = self.interpret(input) {
                        println!("{}", e);
                    }

                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    return Err(ErrCode::Io(e.to_string()));
                }
            }
        }

        Ok(())
    }

    pub fn run_file(&mut self, path: &str) -> Result<(), ErrCode> {
        fs::read_to_string(path)
            .map_err(|e| ErrCode::Io(e.to_string()))
            .and_then(|source| self.interpret(source))
    }

    pub fn interpret(&mut self, source: String) -> Result<(), ErrCode> {
        Compiler::new(source).compile().and_then(|compiler| {
            self.ip = 0;
            self.objects = compiler.objects;
            self.run(compiler.chunk)
        })
    }

    fn run(&mut self, chunk: Chunk) -> Result<(), ErrCode> {
        while self.ip < chunk.code.len() {
            if DEBUG_TRACE {
                self.stack_trace();
                chunk.disassamble_instruction(self.ip, &chunk.code[self.ip])
            }

            let op_result = match chunk.code[self.ip] {
                OpCode::Return => Ok(()),
                OpCode::Constant(index) => {
                    self.stack.push(chunk.constants.get(index));
                    Ok(())
                }
                OpCode::Negate => {
                    let top = self.stack_top();
                    match self.stack[top].borrow() {
                        Value::Number(n) => {
                            self.stack[top] = Rc::new(Value::Number(-n));
                            Ok(())
                        }
                        _ => Err(self.runtime_error("Operand must be a number", &chunk)),
                    }
                }
                OpCode::Add => self.stack.pop().zip(self.stack.pop()).map_or(
                    Err(String::from("Not enough values on the stack")),
                    |(right, left)| match (right.borrow(), left.borrow()) {
                        (Value::SourceStr(r), Value::SourceStr(l)) => {
                            self.concat_strings(&l.to_string(), &r.to_string());
                            Ok(())
                        }
                        (Value::SourceStr(r), Value::Str(l)) => {
                            self.concat_strings(l, &r.to_string());
                            Ok(())
                        }
                        (Value::Str(r), Value::SourceStr(l)) => {
                            self.concat_strings(&l.to_string(), r);
                            Ok(())
                        }
                        (Value::Str(r), Value::Str(l)) => {
                            self.concat_strings(l, r);
                            Ok(())
                        }
                        (Value::Number(r), Value::Number(l)) => {
                            self.stack.push(Rc::new(Value::Number(l + r)));
                            Ok(())
                        }
                        _ => Err(String::from("Operands must be two numbers or two strings")),
                    },
                ),
                OpCode::Subtract => self.binary_op(|left, right| Value::Number(left - right)),
                OpCode::Multiply => self.binary_op(|left, right| Value::Number(left * right)),
                OpCode::Divide => self.binary_op(|left, right| Value::Number(left / right)),
                OpCode::True => {
                    self.stack.push(Rc::new(Value::Bool(true)));
                    Ok(())
                }
                OpCode::False => {
                    self.stack.push(Rc::new(Value::Bool(false)));
                    Ok(())
                }
                OpCode::Nil => {
                    self.stack.push(Rc::new(Value::Nil));
                    Ok(())
                }
                OpCode::Not => self
                    .is_falsey()
                    .map(|value| self.stack.push(Rc::new(Value::Bool(value)))),
                OpCode::Equal => self.stack.pop().zip(self.stack.pop()).map_or(
                    Err(String::from("Not enough values on the stack")),
                    |(right, left)| match (right.borrow(), left.borrow()) {
                        (Value::Number(r), Value::Number(l)) => {
                            self.stack.push(Rc::new(Value::Bool(l == r)));
                            Ok(())
                        }
                        (Value::SourceStr(r), Value::SourceStr(l)) => {
                            self.stack
                                .push(Rc::new(Value::Bool(l.to_string() == r.to_string())));
                            Ok(())
                        }
                        (Value::SourceStr(r), Value::Str(l)) => {
                            self.stack.push(Rc::new(Value::Bool(l == &r.to_string())));
                            Ok(())
                        }
                        (Value::Str(r), Value::SourceStr(l)) => {
                            self.stack.push(Rc::new(Value::Bool(&l.to_string() == r)));
                            Ok(())
                        }
                        (Value::Str(b), Value::Str(a)) => {
                            self.stack.push(Rc::new(Value::Bool(a == b)));
                            Ok(())
                        }
                        (Value::Bool(b), Value::Bool(a)) => {
                            self.stack.push(Rc::new(Value::Bool(a == b)));
                            Ok(())
                        }
                        _ => Err(String::from(
                            "Operands must be two numbers, two strings, or two booleans",
                        )),
                    },
                ),
                OpCode::Greater => self.binary_op(|left, right| Value::Bool(left > right)),
                OpCode::Less => self.binary_op(|left, right| Value::Bool(left < right)),
                OpCode::Print => self.print_value(),
                OpCode::Pop => match self.stack.pop() {
                    Some(_) => Ok(()),
                    None => Err(String::from("Not enough values on the stack")),
                },
                OpCode::DefGlobal(index) => {
                    let name = chunk.constants.get(index);
                    match name.borrow() {
                        Value::VarIdent(n) | Value::ValIdent(n) => {
                            let top = self.stack_top();
                            self.globals.insert(n.clone(), Rc::clone(&self.stack[top]));
                            self.stack.pop();
                            Ok(())
                        }
                        _ => Err(String::from("Not a valid identifier")),
                    }
                }
                OpCode::GetGlobal(index) => {
                    let name = chunk.constants.get(index);
                    match name.borrow() {
                        Value::VarIdent(n) | Value::ValIdent(n) => match self.globals.get(n) {
                            Some(value) => {
                                self.stack.push(Rc::clone(value));
                                Ok(())
                            }
                            None => Err(format!("Undefined variable {}", n)),
                        },
                        _ => Err(String::from("Not a valid identifier")),
                    }
                }
                OpCode::SetGlobal(index) => {
                    let name = chunk.constants.get(index);
                    match name.borrow() {
                        Value::VarIdent(n) | Value::ValIdent(n) => {
                            let top = self.stack_top();
                            if self.globals.contains_key(n) {
                                self.globals.insert(n.clone(), Rc::clone(&self.stack[top]));
                                Ok(())
                            } else {
                                Err(format!("Undefined variable {}", n))
                            }
                        }
                        _ => Err(String::from("Not a valid identifier")),
                    }
                }
                OpCode::GetLocal(slot) => {
                    self.stack.push(Rc::clone(&self.stack[slot]));
                    Ok(())
                }
                OpCode::SetLocal(slot) => {
                    let top = self.stack_top();
                    self.stack[slot] = Rc::clone(&self.stack[top]);
                    Ok(())
                }
                OpCode::JumpIfFalse(offset) => {
                    let top = self.stack_top();
                    if self.stack[top].is_falsey() {
                        self.ip += offset;
                    }

                    Ok(())
                }
                OpCode::Jump(offset) => {
                    self.ip += offset;
                    Ok(())
                }
            };

            if let Err(e) = op_result {
                return Err(ErrCode::Runtime(self.runtime_error(&e, &chunk)));
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
                (Value::Number(b), Value::Number(a)) => {
                    self.stack.push(Rc::new(op(*a, *b)));
                    Ok(())
                }
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

    fn concat_strings(&mut self, a: &str, b: &str) {
        let next_obj = self.objects.take();

        let string = Rc::new(Value::Str(format!("{}{}", a, b)));
        self.stack.push(Rc::clone(&string));
        self.objects = Some(Box::new(Obj::new(string, next_obj)));
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
}
