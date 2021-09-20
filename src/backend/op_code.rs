use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpCode {
    Constant(usize),
    Return,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    True,
    False,
    Nil,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(index) => {
                write!(f, "CONSTANT {number:>width$}", number = index, width = 16)
            }
            Self::Return => write!(f, "RETURN"),
            Self::Negate => write!(f, "NEGATE"),
            Self::Add => write!(f, "ADD"),
            Self::Subtract => write!(f, "SUBTRACT"),
            Self::Multiply => write!(f, "MULTIPLY"),
            Self::Divide => write!(f, "DIVIDE"),
            Self::True => write!(f, "TRUE"),
            Self::False => write!(f, "FALSE"),
            Self::Nil => write!(f, "NIL"),
        }
    }
}
