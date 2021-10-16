use std::rc::Rc;

pub trait Obj {
    fn debug_value(&self) -> &String;
    fn next_debug_value(&self) -> Option<&String>;
    fn free(self) -> Option<Rc<dyn Obj>>;
}
