use std::{borrow::Borrow, rc::Rc};
use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ValueArray {
    values: Vec<Rc<Value>>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray { values: vec![] }
    }

    pub fn write(&mut self, value: Rc<Value>) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> &Rc<Value> {
        &self.values[index]
    }

    pub fn find_identifier(&self, query: &str) -> Option<(usize, &Rc<Value>)> {
        for (index, constant) in self.values.iter().enumerate() {
            match (*constant).borrow() {
                Value::ValIdent(name) => {
                    if name == query {
                        return Some((index, constant));
                    }
                }
                Value::VarIdent(name) => {
                    if name == query {
                        return Some((index, constant));
                    }
                }
                _ => (),
            }
        }

        None
    }
}
