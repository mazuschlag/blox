#[derive(Debug, Clone)]
pub enum ErrCode {
    CompileError,
    RuntimeError(String),
    IoError(String),
}

