use code::OpCode;
use memory::reallocate;
#[derive(Debug)]
pub struct Chunk {
    capacity: usize,
    count: usize,
    code: Vec<OpCode>
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            capacity: 0,
            count: 0,
            code: vec![]
        }
    }

    pub fn write(&mut self, byte: OpCode) {
        let new_count = self.count + 1;
        if self.capacity < new_count {
            self.capacity = match self.capacity {
                0 => 8,
                _ => self.capacity * 2
            };
            self.code = reallocate(&self.code, self.capacity);
        }
        self.code.push(byte);
        self.count = new_count;
    }

    pub fn free(self) -> Chunk {
        return Chunk::new();
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        while offset < self.count {
            offset = self.disassamble_instruction(offset);
        }
    }

    fn disassamble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        let instruction = self.code[offset];
        return match instruction {
            OpCode::Return => Chunk::simple_instruction(instruction, offset)
        };
    }

    fn simple_instruction(name: OpCode, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
}

pub mod memory {
    use super::code::OpCode;

    pub fn reallocate(chunk: &Vec<OpCode>, new_size: usize) -> Vec<OpCode> {
        if new_size == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(new_size);
        for byte in chunk {
            result.push(byte.clone());
        }
        return result;
    }
}

pub mod code {
    use std::fmt;
    #[derive(Debug, Copy, Clone)]
    pub enum OpCode {
        Return
    }
    
    impl fmt::Display for OpCode {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}
