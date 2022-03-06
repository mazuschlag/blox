#[derive(Debug, Clone)]
pub enum ErrCode {
    Compile,
    Runtime(String),
    Io(String),
}
