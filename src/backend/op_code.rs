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
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal(usize),
    GetGlobal(usize),
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(index) => {
                write!(f, "CONSTANT {number:>width$}", number = index, width = 16)
            }
            Self::DefineGlobal(index) => {
                write!(
                    f,
                    "DEFINE_GLOBAL {number:>width$}",
                    number = index,
                    width = 16
                )
            }
            Self::GetGlobal(index) => {
                write!(f, "GET_GLOBAL {number:>width$}", number = index, width = 16)
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
            Self::Not => write!(f, "NOT"),
            Self::Equal => write!(f, "EQUAL"),
            Self::Greater => write!(f, "GREATER"),
            Self::Less => write!(f, "LESS"),
            Self::Print => write!(f, "PRINT"),
            Self::Pop => write!(f, "POP"),
        }
    }
}
