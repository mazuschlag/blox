use std::{
    borrow::Borrow,
    collections::HashMap,
    fs,
    io::{self, BufRead, Write},
    rc::Rc,
    str,
};

use crate::{
    error::codes::ErrCode,
    frontend::compiler::Compiler,
};

use super::{
    call_frame::CallFrame,
    chunk::Chunk,
    function_obj::{FunctionObj, FunctionType},
    obj::Obj,
    op_code::OpCode,
    value::Value,
};

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * u8::MAX as usize;

pub struct Vm {
    frame_count: usize,
    stack: Vec<Rc<Value>>,
    objects: Option<Rc<Obj>>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Rc<Value>>,
    debug_print_code: bool,
    debug_trace: bool,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            stack: Vec::with_capacity(STACK_MAX),
            objects: None,
            frames: Vec::with_capacity(FRAMES_MAX),
            globals: HashMap::new(),
            debug_print_code: false,
            debug_trace: false,
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

    pub fn run_file(&mut self, path: &str, debug_print_code: bool, debug_trace: bool) -> Result<(), ErrCode> {
        self.debug_print_code = debug_print_code;
        self.debug_trace = debug_trace;
        fs::read_to_string(path)
            .map_err(|e| ErrCode::Io(e.to_string()))
            .and_then(|source| self.interpret(source))
    }

    pub fn interpret(&mut self, source: String) -> Result<(), ErrCode> {
        let compiler = Compiler::new(source, FunctionType::Script, self.debug_print_code).compile()?;
        let frame = CallFrame::new(compiler.function, 0, 0);
        self.frames.push(frame);
        self.frame_count = self.frames.len();
        self.objects = compiler.objects;
        self.run()
    }

    fn run(&mut self) -> Result<(), ErrCode> {
        while self.frame().ip < self.frame().function.chunk.count() {
            if self.debug_trace {
                self.stack_trace();
                let ip = self.frame().ip;
                let op = self.frame().function.chunk.code[ip];
                self.frame().function.chunk.disassamble_instruction(ip, &op)
            }

            let op_result = self.match_op();
            if let Err(e) = op_result {
                return Err(ErrCode::Runtime(self.runtime_error(&e)));
            }

            self.frame().ip += 1;
        }

        Ok(())
    }

    fn match_op(&mut self) -> Result<(), String> {
        let ip = self.frame().ip;
        match self.frame().function.chunk.code[ip] {
            OpCode::Return => Ok(()),
            OpCode::Constant(index) => {
                let value =Rc::clone(&self.frame().function.chunk.constants.get(index));
                self.stack.push(value);
                Ok(())
            }
            OpCode::Negate => {
                let top = self.stack_top();
                if let Value::Number(n) = self.stack[top].borrow() {
                    self.stack[top] = Rc::new(Value::Number(-n));
                    return Ok(());
                }

                Err(self.runtime_error("Operand must be a number"))
            }
            OpCode::Add => {
                let (left, right) = self.get_left_right()?;
                
                match (right.borrow(), left.borrow()) {
                    (Value::SourceStr(r), Value::SourceStr(l)) => {
                        self.concat_strings(&l.to_string(), &r.to_string());
                    }
                    (Value::SourceStr(r), Value::Str(l)) => {
                        self.concat_strings(l, &r.to_string());
                    }
                    (Value::Str(r), Value::SourceStr(l)) => {
                        self.concat_strings(&l.to_string(), r);
                    }
                    (Value::Str(r), Value::Str(l)) => {
                        self.concat_strings(l, r);
                    }
                    (Value::Number(r), Value::Number(l)) => {
                        let value = Value::Number(l + r);
                        self.stack.push(Rc::new(value));
                    }
                    _ => return Err(String::from("Operands must be two numbers or two strings"))
                };

                Ok(())
            }
            OpCode::Subtract => self.binary_op(|right, left| Value::Number(left - right)),
            OpCode::Multiply => self.binary_op(|right, left| Value::Number(left * right)),
            OpCode::Divide => self.binary_op(|right, left| Value::Number(left / right)),
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
            OpCode::Not => {
                let value = self.is_falsey()?;
                self.stack.push(Rc::new(Value::Bool(value)));
                Ok(())
            }
            OpCode::Equal => { 
                let (left, right) = self.get_left_right()?;

                let value = match (right.borrow(), left.borrow()) {
                    (Value::Number(r), Value::Number(l)) => {
                        Value::Bool(l == r)
                    }
                    (Value::SourceStr(r), Value::SourceStr(l)) => {
                        Value::Bool(l.to_string() == r.to_string())
                    }
                    (Value::SourceStr(r), Value::Str(l)) => {
                        Value::Bool(l == &r.to_string())
                    }
                    (Value::Str(r), Value::SourceStr(l)) => {
                        Value::Bool(&l.to_string() == r)
                    }
                    (Value::Str(b), Value::Str(a)) => {
                        Value::Bool(a == b)
                    }
                    (Value::Bool(b), Value::Bool(a)) => {
                        Value::Bool(a == b)
                    }
                    _ => return Err(String::from(
                        "Operands must be two numbers, two strings, or two booleans",
                    )),
                };

                self.stack.push(Rc::new(value));
                Ok(())
            }
            OpCode::Greater => self.binary_op(|right, left| Value::Bool(left > right)),
            OpCode::Less => self.binary_op(|right, left| Value::Bool(left < right)),
            OpCode::Print => self.print_value(),
            OpCode::Pop => match self.stack.pop() {
                Some(_) => Ok(()),
                None => Err(String::from("Not enough values on the stack")),
            },
            OpCode::DefGlobal(index) => {
                let name = Rc::clone(&self.frame().function.chunk.constants.get(index));
                match name.borrow() {
                    Value::VarIdent(n) | Value::ValIdent(n) => {
                        let top = self.stack_top();
                        self.globals.insert(n.clone(), self.stack[top].clone());
                        self.stack.pop();
                        Ok(())
                    }
                    _ => Err(String::from("Not a valid identifier")),
                }
            }
            OpCode::GetGlobal(index) => {
                let name = Rc::clone(&self.frame().function.chunk.constants.get(index));
                match name.borrow() {
                    Value::VarIdent(n) | Value::ValIdent(n) => match self.globals.get(n) {
                        Some(value) => {
                            self.stack.push(value.clone());
                            Ok(())
                        }
                        None => Err(format!("Undefined variable {}", n)),
                    },
                    _ => Err(String::from("Not a valid identifier")),
                }
            }
            OpCode::SetGlobal(index) => {
                let name_ref = Rc::clone(&self.frame().function.chunk.constants.get(index));
                let name = match name_ref.borrow() {
                    Value::VarIdent(name) | Value::ValIdent(name) => name,
                    _ => return Err(String::from("Not a valid identifier")),
                };
                
                let top = self.stack_top();
                if self.globals.contains_key(name) {
                    self.globals.insert(name.clone(), Rc::clone(&self.stack[top]));
                    return Ok(());
                }
                    
                Err(format!("Undefined variable {}", name_ref))
            }
            OpCode::GetLocal(slot) => {
                let offset = self.frame().slots_start + slot;
                let value = Rc::clone(&self.stack[offset]);
                self.stack.push(value);
                Ok(())
            }
            OpCode::SetLocal(slot) => {
                let top = self.stack_top();
                let offset = self.frame().slots_start + slot;
                self.stack[offset] = Rc::clone(&self.stack[top]);
                Ok(())
            }
            OpCode::JumpIfFalse(offset) => {
                let top = self.stack_top();
                if self.stack[top].is_falsey() {
                    self.frame().ip += offset;
                }

                Ok(())
            }
            OpCode::Jump(offset) => {
                self.frame().ip += offset;
                Ok(())
            }
            OpCode::Loop(offset) => {
                self.frame().ip -= offset + 1;
                Ok(())
            }
            OpCode::Case(offset) => {
                let top = self.stack_top();
                let below = top - 1;
                if self.stack.len() < 2 {
                    return Err(String::from("Not enough values on the stack"))
                }

                match (self.stack[top].borrow(), self.stack[below].borrow()) {
                    (Value::Number(r), Value::Number(l)) => {
                        if r != l {
                            self.frame().ip += offset;
                            self.stack.pop();
                        }
                    }
                    (Value::SourceStr(r), Value::SourceStr(l)) => {
                        if l.to_string() != r.to_string() {
                            self.frame().ip += offset;
                            self.stack.pop();
                        }
                    }
                    (Value::SourceStr(r), Value::Str(l)) => {
                        if l != &r.to_string() {
                            self.frame().ip += offset;
                            self.stack.pop();
                        }
                    }
                    (Value::Str(r), Value::SourceStr(l)) => {
                        if &l.to_string() != r {
                            self.frame().ip += offset;
                            self.stack.pop();
                        }
                    }
                    (Value::Str(r), Value::Str(l)) => {
                        if l != r {
                            self.frame().ip += offset;
                            self.stack.pop();
                        }
                    }
                    (Value::Bool(r), Value::Bool(l)) => {
                        if l != r {
                            self.frame().ip += offset;
                            self.stack.pop();
                        }
                    }
                    _ => return Err(String::from("Mismatched types in case statement"))
                }

                Ok(())
            }
        }
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
        self.objects = Some(Rc::new(Obj::new(string, next_obj)));
    }

    fn is_falsey(&mut self) -> Result<bool, String> {
        let value = self.stack.pop();
        if let Some(v) = value {
            return Ok(v.is_falsey());
        }

        Err(String::from("Not enough values on stack"))
    }

    fn get_left_right(&mut self) -> Result<(Rc<Value>, Rc<Value>), String> {
        let right_ref = self.stack.pop();
        let left_ref = self.stack.pop();
        match (right_ref, left_ref) {
            (None, _) | (_, None) => return Err(String::from("Not enough values on the stack")),
            (Some(right), Some(left)) => Ok((left, right)),
        }
    }

    fn frame(&mut self) -> &mut CallFrame {
        &mut self.frames[self.frame_count - 1]
    }

    fn stack_top(&self) -> usize {
        self.stack.len() - 1
    }

    fn runtime_error(&mut self, msg: &str) -> String {
        let frame = self.frame();
        format!("{}\n[line {}] in script\n", msg, frame.function.chunk.get_line(frame.ip))
    }

    fn stack_trace(&self) {
        print!("           ");
        for index in 0..self.stack.len() {
            print!("[ {} ]", self.stack[index]);
        }

        println!();
    }
}
