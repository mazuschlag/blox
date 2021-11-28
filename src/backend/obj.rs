use std::fmt;
use std::rc::Rc;

use super::value::Value;

pub struct Obj {
    pub value: Value,
    pub next: Option<Rc<Obj>>,
}

impl Obj {
    pub fn new(value: Value, next: Option<Rc<Obj>>) -> Self {
        Self { value, next }
    }

    pub fn debug_value(&self) -> &Value {
        &self.value
    }

    pub fn next_debug_value(&self) -> Option<&Value> {
        match &self.next {
            Some(obj) => Some(obj.debug_value()),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn free(self) -> Option<Rc<Obj>> {
        self.next
    }
}

impl fmt::Debug for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Obj")
            .field("value", &self.value)
            .field("next", &self.next_debug_value())
            .finish()
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
