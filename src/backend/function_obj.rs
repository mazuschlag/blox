use super::{chunk::Chunk, source_str::SourceStr};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionObj {
    pub arity: u32,
    pub chunk: Chunk,
    pub name: SourceStr,
}

impl FunctionObj {
    fn new(name: SourceStr) -> FunctionObj {
        FunctionObj { arity: 0, chunk: Chunk::new(), name: name }
    }
}