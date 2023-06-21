use std::{borrow::Borrow, rc::Rc};
use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: vec![] }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> Value {
        self.values[index].clone()
    }

    pub fn find_identifier(&self, query: &str) -> Option<(usize, Value)> {
        for (index, constant) in self.values.iter().enumerate() {
            match (*constant).borrow() {
                Value::ValIdent(name) => {
                    if name == query {
                        return Some((index, constant.clone()));
                    }
                }
                Value::VarIdent(name) => {
                    if name == query {
                        return Some((index, constant.clone()));
                    }
                }
                _ => (),
            }
        }

        None
    }
}
