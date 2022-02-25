use std::borrow::Borrow;
use std::rc::Rc;

use super::value::Value;

#[derive(Debug)]
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

    pub fn get(&self, index: usize) -> Rc<Value> {
        Rc::clone(&self.values[index])
    }

    pub fn find_identifier(&self, query: &String) -> Option<usize> {
        for (index, constant) in self.values.iter().enumerate() {
            let value = Rc::clone(constant);
            match value.borrow() {
                Value::Ident(name) => {
                    if name == query {
                        return Some(index);
                    }
                }
                _ => (),
            }
        }

        None
    }
}
