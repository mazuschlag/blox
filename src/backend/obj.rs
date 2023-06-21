use std::{
    borrow::Borrow,
    cell::RefCell,
    fmt,
    rc::Rc,
};

use super::{
    value::Value,
    value_array::ValueArray,
};

pub struct Obj {
    pub value_array: Rc<RefCell<ValueArray>>,
    pub value: usize,
    pub next: Option<Rc<Obj>>,
}

impl Obj {
    pub fn new(value_array: Rc<RefCell<ValueArray>>, value: usize, next: Option<Rc<Obj>>) -> Self {
        Self { value_array, value, next }
    }

    pub fn peek(&self) -> Value {
        (*self.value_array).borrow().get(self.value).clone()
    }

    pub fn peek_next(&self) -> Option<Rc<Obj>> {
        match &self.next {
            Some(obj) => Some(Rc::clone(&obj)),
            None => None,
        }
    }
}

impl fmt::Debug for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Obj")
            .field("value", &self.peek())
            .field("next", &self.peek_next())
            .finish()
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
