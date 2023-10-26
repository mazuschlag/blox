use std::fmt;

use super::{source_str::SourceStr, function_obj::FunctionObj};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(String),
    SourceStr(SourceStr),
    VarIdent(String),
    ValIdent(String),
    Function(FunctionObj),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Bool(false) | Value::Nil)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Str(s) => write!(f, "\"{}\"", s),
            Self::SourceStr(s) => write!(f, "\"{}\"", s),
            Self::VarIdent(s) => write!(f, "<var {}>", s),
            Self::ValIdent(s) => write!(f, "<val {}>", s),
            Self::Function(o) => write!(f, "{}", o.name),
            Self::Nil => write!(f, "nil"),
        }
    }
}
