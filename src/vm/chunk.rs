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
        self.lines.push(line);
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
            if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
                print!("    | ");
            } else {
                print!("{number:>width$} ", number = self.lines[offset], width = 5);
            }
            self.disassamble_instruction(byte);
        }
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
