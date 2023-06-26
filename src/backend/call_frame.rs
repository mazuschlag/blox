use super::function_obj::FunctionObj;

pub struct CallFrame {
    pub function: FunctionObj,
    pub ip: usize,
    pub slots_start: usize,
}

impl CallFrame {
    pub fn new(function: FunctionObj, ip: usize, slots_start: usize) -> CallFrame {
        CallFrame {
            function,
            ip,
            slots_start,
        }
    }
}
