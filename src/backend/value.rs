use std::fmt;

use super::source_str::SourceStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(String),
    SourceStr(SourceStr),
    VarIdent(String),
    ValIdent(String),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        *self == Value::Nil || (self.is_bool() && !self.as_bool())
    }

    fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    fn as_bool(&self) -> bool {
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
            _ => write!(f, "nil"),
        }
    }
}
