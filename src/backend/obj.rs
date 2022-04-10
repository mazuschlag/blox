use std::fmt;
use std::rc::Rc;

use super::value::Value;

pub struct Obj {
    pub value: Rc<Value>,
    pub next: Option<Box<Obj>>,
}

impl Obj {
    pub fn new(value: Rc<Value>, next: Option<Box<Obj>>) -> Self {
        Self { value, next }
    }

    pub fn peek(&self) -> &Value {
        &self.value
    }

    pub fn peek_next(&self) -> Option<&Value> {
        match &self.next {
            Some(obj) => Some(obj.peek()),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn free(self) -> Option<Box<Obj>> {
        self.next
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
