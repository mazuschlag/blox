use super::value::{Value, ValueArray};
use std::fmt;

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: ValueArray,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            constants: ValueArray::new(),
            lines: vec![],
        }
    }

    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.write_line(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.count() - 1
    }

    #[allow(dead_code)]
    pub fn free(self) -> Chunk {
        return Chunk::new();
    }

    #[allow(dead_code)]
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (offset, byte) in self.code.iter().enumerate() {
            self.disassamble_instruction(offset, byte);
        }
    }
    
    pub fn disassamble_instruction(&self, offset: usize, instruction: &OpCode) {
        print!("{:04} ", offset);
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("    | ");
        } else {
            print!("{number:>width$} ", number = self.get_line(offset), width = 5);
        }
        match instruction {
            OpCode::Constant(index) => {
                println!("{} '{}'", instruction, self.constants.get(*index))
            },
            _ => println!("{}", instruction),
        };
    }

    fn write_line(&mut self, line: usize) {
        let current_line = self.lines.len();
        if current_line == line {
            self.lines[current_line - 1] += 1;
        } else {
            self.lines.push(1);
        }
    }

    fn get_line(&self, offset: usize) -> usize {
        let mut current_index = 0;
        let mut line_counter = self.lines[current_index];
        while line_counter <= offset && current_index < self.lines.len() {
            line_counter += self.lines[line_counter];
            current_index += 1;
        }
        current_index + 1
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum OpCode {
    Constant(usize),
    Return,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpCode::Constant(index) => write!(
                f,
                "CONSTANT {number:>width$}",
                number = index,
                width = 16
            ),
            OpCode::Return => write!(f, "RETURN"),
            OpCode::Negate => write!(f, "NEGATE"),
            OpCode::Add => write!(f, "ADD"),
            OpCode::Subtract => write!(f, "SUBTRACT"),
            OpCode::Multiply => write!(f, "MULTIPLY"),
            OpCode::Divide => write!(f, "DIVIDE"),
        }
    }
}
