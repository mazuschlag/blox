mod vm;

use crate::vm::chunk::Chunk;
use crate::vm::chunk::code::OpCode;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return);
    chunk.disassemble("text chunk");
    chunk.free();
}
