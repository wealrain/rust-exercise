use failure::Fail;
use std::io;

///error types
#[derive(Fail,Debug)]
pub enum KvsError {
    /// IO Error
    #[fail(display = "{}",_0)]
    Io(#[cause] io::Error),
    /// 序列化和反序列化异常
    #[fail(display = "{}",_0)]
    Serde(#[cause] serde_json::Error),
    /// 键不存在
    #[fail(display = "Key not found")]
    KeyNotFound,
    /// 命令不存在
    #[fail(display = "Unexcepted command type")]
    UnexceptedCommandType
}

impl From<io::Error> for KvsError {
    fn from(value: io::Error) -> Self {
        KvsError::Io(value)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(value: serde_json::Error) -> Self {
        KvsError::Serde(value)
    }
}

/// Result type for kvs 
pub type Result<T> = std::result::Result<T,KvsError>;