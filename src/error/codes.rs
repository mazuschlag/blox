use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrCode {
    CompileError(String),
    RuntimeError(String),
    ScannerError(String),
}

impl fmt::Display for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrCode::CompileError(e) => write!(f, "Compile error: {}", e),
            ErrCode::RuntimeError(e) => write!(f, "Runtime error: {}", e),
            ErrCode::ScannerError(e) => write!(f, "Scanner error: {}", e),
        }
    }
}
