#![deny(missing_docs)]
//! A simple kv store

pub use error::{KvsError,Result};
pub use kv::KvStore;

mod error;
mod kv;