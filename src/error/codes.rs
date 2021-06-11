use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrCode {
    CompileError,
    RuntimeError,
    ScannerError,
}

impl fmt::Display for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrCode::CompileError => write!(f, "Compile error"),
            ErrCode::RuntimeError => write!(f, "Runtime error"),
            ErrCode::ScannerError => write!(f, "Scanner error"),
        }
    }
}
