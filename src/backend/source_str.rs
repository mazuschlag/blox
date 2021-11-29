use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceStr {
    start: usize,
    len: usize,
    source: Rc<Vec<char>>,
}

impl SourceStr {
    pub fn new(start: usize, len: usize, source: Rc<Vec<char>>) -> Self {
        SourceStr { start, len, source }
    }
}

impl fmt::Display for SourceStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.source[self.start..self.start + self.len]
                .iter()
                .collect::<String>()
        )
    }
}
