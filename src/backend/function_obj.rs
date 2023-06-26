use super::chunk::Chunk;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionObj {
    pub arity: u32,
    pub chunk: Chunk,
    pub name: String,
}

impl FunctionObj {
    pub fn new(name: String) -> FunctionObj {
        FunctionObj {
            arity: 0,
            chunk: Chunk::new(),
            name: name,
        }
    }
}

impl fmt::Display for FunctionObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }
    }
}

pub enum FunctionType {
    Function,
    Script,
}
