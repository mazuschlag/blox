use std::fmt;
#[allow(dead_code)]
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
    DefGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    GetLocal(usize),
    SetLocal(usize),
    JumpIfFalse(usize),
    Jump(usize),
    Loop(usize),
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(index) => {
                write!(f, "CONSTANT {number:>width$}", number = index, width = 16)
            }
            Self::DefGlobal(index) => {
                write!(f, "DEF_GLOBAL {number:>width$}", number = index, width = 14)
            }
            Self::GetGlobal(index) => {
                write!(f, "GET_GLOBAL {number:>width$}", number = index, width = 14)
            }
            Self::SetGlobal(index) => {
                write!(f, "SET_GLOBAL {number:>width$}", number = index, width = 14)
            }
            Self::GetLocal(index) => {
                write!(f, "GET_LOCAL {number:>width$}", number = index, width = 15)
            }
            Self::SetLocal(index) => {
                write!(f, "SET_LOCAL {number:>width$}", number = index, width = 15)
            }
            Self::JumpIfFalse(index) => {
                write!(
                    f,
                    "JUMP_IF_FALSE {number:>width$}",
                    number = index,
                    width = 11
                )
            }
            Self::Jump(index) => {
                write!(f, "JUMP {number:>width$}", number = index, width = 20)
            }
            Self::Loop(index) => {
                write!(f, "LOOP {number:>width$}", number = index, width = 20)
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
