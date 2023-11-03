use failure::Fail;
use std::{io, string::FromUtf8Error};

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
    UnexceptedCommandType,
    ///
    #[fail(display = "{}",_0)]
    StringError(String),
     ///
     #[fail(display = "UTF-8 error: {}",_0)]
     Utf8(#[cause] FromUtf8Error),
     ///
     #[fail(display = "sled error: {}", _0)]
     Sled(#[cause] sled::Error),
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

impl From<FromUtf8Error> for KvsError {
    fn from(value: FromUtf8Error) -> Self {
        KvsError::Utf8(value)
    }
}

impl From<sled::Error> for KvsError {
    fn from(err: sled::Error) -> KvsError {
        KvsError::Sled(err)
    }
}

/// Result type for kvs 
pub type Result<T> = std::result::Result<T,KvsError>;