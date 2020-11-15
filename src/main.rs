mod vm;

use crate::vm::chunk::code::OpCode;
use crate::vm::chunk::Chunk;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant(constant), 1);
    chunk.write(OpCode::Return, 1);
    chunk.disassemble("text chunk");
    chunk.free();
}
