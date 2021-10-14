use super::op_code::OpCode;
use super::value::Value;
use super::value_array::ValueArray;

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: ValueArray,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
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
            print!(
                "{number:>width$} ",
                number = self.get_line(offset),
                width = 5
            );
        }
        match instruction {
            OpCode::Constant(index) => {
                println!("{} '{}'", instruction, self.constants.get(*index))
            }
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

    pub fn get_line(&self, offset: usize) -> usize {
        let mut current_index = 0;
        let mut line_counter = self.lines[current_index];
        while line_counter <= offset && current_index < self.lines.len() {
            line_counter += self.lines[line_counter];
            current_index += 1;
        }
        current_index + 1
    }
}
