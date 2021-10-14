use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(Rc<String>),
    Nil,
}

impl Value {
    pub fn is_falsey(self) -> bool {
        self == Value::Nil || (self.is_bool() && !self.as_bool())
    }

    pub fn num_op<F>(&self, other: Value, mut op: F) -> Result<Value, String>
    where
        F: FnMut(f64, f64) -> Value,
    {
        self.num_or_none()
            .zip(other.num_or_none())
            .ok_or(String::from("Operand must be a number"))
            .map(|(a, b)| op(a, b))
    }

    fn num_or_none(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Str(s) => write!(f, "{}", &s),
            _ => write!(f, "nil"),
        }
    }
}
