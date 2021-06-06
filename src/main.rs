mod backend;

use crate::backend::chunk::OpCode;
use crate::backend::chunk::Chunk;
use crate::backend::vm::VM;

fn main() {
    let mut vm = VM::new(true);

    // -((1.2 + 3.4) / 5.6)
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant(constant), 1);

    let constant = chunk.add_constant(3.9);
    chunk.write(OpCode::Constant(constant), 1);

    chunk.write(OpCode::Add, 1);

    let constant = chunk.add_constant(5.1);
    chunk.write(OpCode::Constant(constant), 1);

    chunk.write(OpCode::Divide, 1);
    chunk.write(OpCode::Negate, 1);
    chunk.write(OpCode::Return, 1);
    println!("{}", vm.interpret(chunk));
}
