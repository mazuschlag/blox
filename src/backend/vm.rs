use std::ops::{Add, Sub, Mul, Div};

use super::chunk::Chunk;
use super::chunk::OpCode;
use super::value::Value;

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    debug_trace: bool
}

impl VM {
    pub fn new(debug_trace: bool) -> VM {
        VM {
            ip: 0,
            stack: vec![],
            debug_trace
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.run(chunk)
    }

    fn run(&mut self, chunk: Chunk) -> InterpretResult {
        while self.ip < chunk.code.len() {
            if self.debug_trace {
                self.stack_trace();
                chunk.disassamble_instruction(self.ip, &chunk.code[self.ip])
            }
            match chunk.code[self.ip] {
                OpCode::Return => {
                    match self.stack.pop() {
                        Some(value) => println!("{}", value),
                        None => println!("void")
                    }
                },
                OpCode::Constant(index) => {
                    self.stack.push(chunk.constants.get(index))
                },
                OpCode::Negate => {
                    match self.stack.pop() {
                        Some(value) => self.stack.push(-value),
                        None => return InterpretResult::RuntimeError("Runtime Error: stack is empty".to_string())
                    }
                },
                OpCode::Add => if let Err(err) = self.binary_op(Add::add) {
                    return InterpretResult::RuntimeError(err)
                },
                OpCode::Subtract => if let Err(err) = self.binary_op(Sub::sub) {
                    return InterpretResult::RuntimeError(err)
                },
                OpCode::Multiply => if let Err(err) = self.binary_op(Mul::mul) {
                    return InterpretResult::RuntimeError(err)
                },
                OpCode::Divide => if let Err(err) = self.binary_op(Div::div) {
                    return InterpretResult::RuntimeError(err)
                },
            }
            self.ip += 1;
        }
        InterpretResult::NoCode
    }

    fn stack_trace(&self) {
        print!("          ");
        for index in 0..self.stack.len() {
            print!("[ {} ]", self.stack[index]);
        }
        println!();
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
        where F: FnMut(Value, Value) -> Value
    {
        let right = self.stack.pop();
        let left = self.stack.pop();
        if let Some(a) = left {
            if let Some(b) = right {
                self.stack.push(op(a, b));
                return Ok(())
            }
        }
        return Err("Runtime Error: stack is empty".to_string())
    }
}

#[allow(dead_code)]
pub enum InterpretResult {
    Ok,
    CompileError(String),
    RuntimeError(String),
    NoCode
}