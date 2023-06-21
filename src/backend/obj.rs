use std::{
    borrow::Borrow,
    fmt,
    rc::Rc,
};

use super::value::Value;

pub struct Obj
    where  {
    pub value: Rc<Value>,
    pub next: Option<Rc<Obj>>,
}

impl Obj {
    pub fn new(value: Rc<Value>, next: Option<Rc<Obj>>) -> Self {
        Self { value, next }
    }

    pub fn peek(&self) -> &Value {
        self.value.borrow()
    }

    pub fn peek_next(&self) -> Option<&Value> {
        match &self.next {
            Some(obj) => Some(&obj.value),
            None => None,
        }
    }
}

impl fmt::Debug for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Obj")
            .field("value", &self.value)
            .field("next", &self.peek_next())
            .finish()
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
