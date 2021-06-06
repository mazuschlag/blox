use std::ops::{Add, Sub, Mul, Div};

use super::chunk::Chunk;
use super::chunk::OpCode;
use super::value::Value;

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Stack,
    debug_trace: bool
}

impl VM {
    pub fn new(debug_trace: bool) -> VM {
        VM {
            chunk: None,
            ip: 0,
            stack: Stack::new(),
            debug_trace
        }
    }

    pub fn interpret(mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = Some(chunk);
        self.run()
    }

    fn run(mut self) -> InterpretResult {
        if let Some(chunk) = self.chunk {
            while self.ip < chunk.code.len() {
                if self.debug_trace {
                    self.stack.trace();
                    chunk.disassamble_instruction(self.ip, &chunk.code[self.ip])
                }
                match chunk.code[self.ip] {
                    OpCode::Return => {
                        println!("{}", self.stack.pop())
                    },
                    OpCode::Constant(index) => {
                        self.stack.push(chunk.constants.get(index))
                    },
                    OpCode::Negate => {
                        let value = -self.stack.pop();
                        self.stack.push(value);
                    },
                    OpCode::Add => self.stack.binary_op(Add::add),
                    OpCode::Subtract => self.stack.binary_op(Sub::sub),
                    OpCode::Multiply => self.stack.binary_op(Mul::mul),
                    OpCode::Divide => self.stack.binary_op(Div::div)
                }
                self.ip += 1;
            }
        }
        InterpretResult::NoCode
    }
}

struct Stack {
    stack: [Value; STACK_MAX],
    top: usize
}

impl Stack {
    fn new() -> Stack {
        Stack {
            stack: [0 as Value; STACK_MAX],
            top: 0
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.top = 0;
    }

    fn push(&mut self, value: Value) {
        self.stack[self.top] = value;
        self.top += 1;
    }

    fn pop(&mut self) -> Value {
        self.top -= 1;
        self.stack[self.top]
    }

    fn trace(&self) {
        print!("          ");
        for index in 0..self.top {
            print!("[ {} ]", self.stack[index]);
        }
        println!();
    }

    fn binary_op<F>(&mut self, mut op: F)
        where F: FnMut(Value, Value) -> Value
    {
        let right = self.pop();
        let left = self.pop();
        self.push(op(left, right));
    }
}

#[allow(dead_code)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
    NoCode
}