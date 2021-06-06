use super::value::{Value, ValueArray};
use code::OpCode;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<OpCode>,
    constants: ValueArray,
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

    pub fn free(self) -> Chunk {
        return Chunk::new();
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (offset, byte) in self.code.iter().enumerate() {
            print!("{:04} ", offset);
            if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
                print!("    | ");
            } else {
                print!("{number:>width$} ", number = self.get_line(offset), width = 5);
            }
            self.disassamble_instruction(byte);
        }
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

    fn disassamble_instruction(&self, instruction: &OpCode) {
        match instruction {
            OpCode::Constant(index) => {
                println!("{} '{}'", instruction, self.constants.get(index.clone()))
            }
            OpCode::Return => println!("{}", instruction),
        };
    }
}

pub mod code {
    use std::fmt;
    #[derive(Debug, Copy, Clone)]
    pub enum OpCode {
        Constant(usize),
        Return,
    }

    impl fmt::Display for OpCode {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                OpCode::Constant(constant) => write!(
                    f,
                    "CONSTANT {number:>width$}",
                    number = constant,
                    width = 16
                ),
                OpCode::Return => write!(f, "RETURN"),
            }
        }
    }
}
