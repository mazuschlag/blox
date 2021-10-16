use std::fmt;
use std::rc::Rc;

use super::obj::Obj;

pub struct StrObj {
    pub value: String,
    pub next: Option<Rc<dyn Obj>>,
}

impl StrObj {
    pub fn new(value: String, next: Option<Rc<dyn Obj>>) -> Self {
        Self { value, next }
    }
}

impl Obj for StrObj {
    fn debug_value(&self) -> &String {
        &self.value
    }

    fn next_debug_value(&self) -> Option<&String> {
        match &self.next {
            Some(obj) => Some(obj.debug_value()),
            _ => None,
        }
    }

    fn free(self) -> Option<Rc<dyn Obj>> {
        self.next
    }
}

impl fmt::Debug for StrObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StrObj")
            .field("value", &self.value)
            .field("next", &self.next_debug_value())
            .finish()
    }
}

impl PartialEq for StrObj {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
